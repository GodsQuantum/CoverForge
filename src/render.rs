use crate::model::{Canvas, Frame, Layer, RenderRequest, RenderResponse, RenderedAsset, Template};
use anyhow::{Context, Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD};
use image::{
    DynamicImage, GenericImageView, ImageFormat, codecs::jpeg::JpegEncoder, imageops::FilterType,
};
use resvg::{tiny_skia, usvg};
use std::{
    collections::BTreeMap,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct Renderer {
    template_dir: PathBuf,
    output_dir: PathBuf,
    asset_roots: Vec<PathBuf>,
    font_dir: Option<PathBuf>,
}

impl Renderer {
    pub fn from_env() -> Result<Self> {
        let template_dir =
            std::env::var("COVERFORGE_TEMPLATE_DIR").unwrap_or_else(|_| "./templates".into());
        let output_dir =
            std::env::var("COVERFORGE_OUTPUT_DIR").unwrap_or_else(|_| "./output".into());
        let roots =
            std::env::var("COVERFORGE_ASSET_ROOTS").unwrap_or_else(|_| "./examples/assets".into());
        let font_dir = std::env::var("COVERFORGE_FONT_DIR").ok().map(PathBuf::from);
        fs::create_dir_all(&output_dir)?;
        Ok(Self {
            template_dir: PathBuf::from(template_dir),
            output_dir: PathBuf::from(output_dir),
            asset_roots: roots
                .split(';')
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .collect(),
            font_dir,
        })
    }

    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    pub fn list_templates(&self) -> Result<Vec<String>> {
        let mut out = Vec::new();
        if !self.template_dir.exists() {
            return Ok(out);
        }
        for entry in fs::read_dir(&self.template_dir)? {
            let p = entry?.path();
            if p.extension().and_then(|x| x.to_str()) == Some("json") {
                if let Some(stem) = p.file_stem().and_then(|x| x.to_str()) {
                    out.push(stem.to_string());
                }
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn save_template(&self, id: &str, template: &Template) -> Result<()> {
        if !valid_template_id(id) {
            return Err(anyhow!("invalid template id"));
        }
        if template.id != id {
            return Err(anyhow!(
                "template id mismatch: path={id} body={}",
                template.id
            ));
        }
        fs::create_dir_all(&self.template_dir)?;
        let target = self.template_dir.join(format!("{id}.json"));
        let tmp = self
            .template_dir
            .join(format!(".{id}.json.tmp-{}", Uuid::new_v4().simple()));
        fs::write(&tmp, serde_json::to_vec_pretty(template)?)?;
        fs::rename(&tmp, &target)?;
        Ok(())
    }

    pub fn load_template(&self, id: &str) -> Result<Template> {
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        {
            return Err(anyhow!("invalid template id"));
        }
        let p = self.template_dir.join(format!("{id}.json"));
        let raw = fs::read_to_string(&p).with_context(|| format!("template not found: {id}"))?;
        let t: Template = serde_json::from_str(&raw)?;
        Ok(t)
    }

    pub fn render_legacy(
        &self,
        template_id: &str,
        variant: &str,
        variables: &BTreeMap<String, String>,
        requested_filename: &str,
    ) -> Result<RenderedAsset> {
        let template = self.load_template(template_id)?;
        let (canvas, layers) = resolve_format(&template, variant)?;
        let filename = safe_output_filename(requested_filename)?;
        let path = self.output_dir.join(&filename);
        let svg = self.compose_svg(canvas, layers, variables)?;
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("png")
            .to_lowercase();

        if ext == "jpg" || ext == "jpeg" {
            let tmp = self
                .output_dir
                .join(format!(".{}.png", Uuid::new_v4().simple()));
            self.rasterize(&svg, canvas, &tmp)?;
            encode_jpeg_limited(&tmp, &path, 512 * 1024)?;
            let _ = fs::remove_file(tmp);
        } else {
            self.rasterize(&svg, canvas, &path)?;
        }

        Ok(RenderedAsset {
            variant: variant.into(),
            width: canvas.width,
            height: canvas.height,
            filename: filename.clone(),
            path: path.to_string_lossy().into_owned(),
            url: format!("/outputs/{filename}"),
        })
    }

    pub fn render(&self, req: &RenderRequest) -> Result<RenderResponse> {
        let template = self.load_template(&req.template)?;
        let variants = if req.variants.is_empty() {
            if !template.formats.is_empty() {
                template.formats.keys().cloned().collect()
            } else if template.variants.is_empty() {
                vec!["default".to_string()]
            } else {
                template.variants.keys().cloned().collect()
            }
        } else {
            req.variants.clone()
        };

        let stem = req
            .output_stem
            .as_deref()
            .map(safe_name)
            .unwrap_or_else(|| format!("{}-{}", safe_name(&template.id), Uuid::new_v4().simple()));
        let mut assets = Vec::new();

        for variant in variants {
            let (canvas, layers) = resolve_format(&template, &variant)?;
            let svg = self.compose_svg(canvas, layers, &req.variables)?;
            let filename = format!("{stem}-{}.png", safe_name(&variant));
            let path = self.output_dir.join(&filename);
            self.rasterize(&svg, canvas, &path)?;
            assets.push(RenderedAsset {
                variant,
                width: canvas.width,
                height: canvas.height,
                filename: filename.clone(),
                path: path.to_string_lossy().into_owned(),
                url: format!("/outputs/{filename}"),
            });
        }
        Ok(RenderResponse {
            ok: true,
            template: template.id,
            assets,
        })
    }

    fn compose_svg(
        &self,
        canvas: &Canvas,
        layers: &[Layer],
        vars: &BTreeMap<String, String>,
    ) -> Result<String> {
        let mut body = String::new();
        body.push_str(&format!(
            r#"<rect width="100%" height="100%" fill="{}"/>"#,
            xml(&canvas.background)
        ));
        for layer in layers {
            match layer {
                Layer::Rect {
                    frame,
                    fill,
                    opacity,
                    radius,
                    ..
                } => {
                    let (x, y, w, h) = px(*frame, canvas);
                    let r = radius * canvas.width.min(canvas.height) as f32;
                    body.push_str(&format!(r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{}" opacity="{}"/>"#, xml(fill), clamp(*opacity)));
                }
                Layer::Image {
                    frame,
                    source,
                    fit,
                    focal_x,
                    focal_y,
                    opacity,
                    ..
                } => {
                    let resolved = substitute(source, vars);
                    if resolved.is_empty() {
                        continue;
                    }
                    let path = self.resolve_asset(&resolved)?;
                    let (x, y, w, h) = px(*frame, canvas);
                    let data = self.prepared_image(
                        &path,
                        w.max(1.0) as u32,
                        h.max(1.0) as u32,
                        fit,
                        *focal_x,
                        *focal_y,
                    )?;
                    body.push_str(&format!(r#"<image x="{x}" y="{y}" width="{w}" height="{h}" opacity="{}" href="data:image/png;base64,{}"/>"#, clamp(*opacity), STANDARD.encode(data)));
                }
                Layer::Text {
                    frame,
                    text,
                    font_family,
                    font_weight,
                    font_size,
                    color,
                    stroke_color,
                    stroke_width,
                    align,
                    max_lines,
                    line_height,
                    uppercase,
                    rotation_deg,
                    ..
                } => {
                    let mut value = substitute(text, vars);
                    if *uppercase {
                        value = value.to_uppercase();
                    }
                    if value.is_empty() {
                        continue;
                    }
                    let (x, y, w, h) = px(*frame, canvas);
                    let size = font_size * canvas.height as f32;
                    let lines = wrap_text(&value, w, size, max_lines.unwrap_or(6));
                    let anchor = match align.as_str() {
                        "center" => "middle",
                        "right" => "end",
                        _ => "start",
                    };
                    let tx = match align.as_str() {
                        "center" => x + w / 2.0,
                        "right" => x + w,
                        _ => x,
                    };
                    let total_h = size * line_height * lines.len() as f32;
                    let start_y = y + ((h - total_h).max(0.0) / 2.0) + size;
                    let stroke = stroke_color
                        .as_ref()
                        .map(|c| {
                            format!(
                                r#" stroke="{}" stroke-width="{}" paint-order="stroke fill""#,
                                xml(c),
                                stroke_width * canvas.height as f32
                            )
                        })
                        .unwrap_or_default();
                    let rotation = if rotation_deg.abs() > f32::EPSILON {
                        let cx = x + w / 2.0;
                        let cy = y + h / 2.0;
                        format!(r#" transform="rotate({rotation_deg} {cx} {cy})""#)
                    } else {
                        String::new()
                    };
                    body.push_str(&format!(r#"<text x="{tx}" y="{start_y}" fill="{}" font-family="{}" font-weight="{}" font-size="{size}" text-anchor="{anchor}"{stroke}{rotation}>"#, xml(color), xml(font_family), font_weight));
                    for (i, line) in lines.iter().enumerate() {
                        let dy = if i == 0 { 0.0 } else { size * line_height };
                        body.push_str(&format!(
                            r#"<tspan x="{tx}" dy="{dy}">{}</tspan>"#,
                            xml(line)
                        ));
                    }
                    body.push_str("</text>");
                }
            }
        }
        Ok(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">{body}</svg>"#,
            canvas.width, canvas.height, canvas.width, canvas.height
        ))
    }

    fn resolve_asset(&self, raw: &str) -> Result<PathBuf> {
        // n8n sees the Cloud9 storage bind under /host, while CoverForge runs
        // directly in CT400 where the same tree is mounted at /srv/storage.
        let translated = raw
            .strip_prefix("/host/srv/storage/")
            .map(|tail| format!("/srv/storage/{tail}"))
            .unwrap_or_else(|| raw.to_string());
        let p = PathBuf::from(&translated);
        let candidates: Vec<PathBuf> = if p.is_absolute() {
            vec![p]
        } else {
            self.asset_roots.iter().map(|r| r.join(&p)).collect()
        };
        for candidate in candidates {
            if !candidate.exists() {
                continue;
            }
            let canonical = candidate.canonicalize()?;
            for root in &self.asset_roots {
                let r = root.canonicalize().unwrap_or_else(|_| root.clone());
                if canonical.starts_with(&r) {
                    return Ok(canonical);
                }
            }
        }
        Err(anyhow!("asset not found or outside allowed roots: {raw}"))
    }

    fn prepared_image(
        &self,
        path: &Path,
        w: u32,
        h: u32,
        fit: &str,
        fx: f32,
        fy: f32,
    ) -> Result<Vec<u8>> {
        let img = image::open(path)?;
        let out = if fit == "contain" {
            let mut bg = DynamicImage::new_rgba8(w, h);
            let resized = img.resize(w, h, FilterType::Lanczos3);
            let (rw, rh) = resized.dimensions();
            image::imageops::overlay(
                &mut bg,
                &resized,
                ((w - rw) / 2) as i64,
                ((h - rh) / 2) as i64,
            );
            bg
        } else {
            crop_cover(img, w, h, fx, fy)
        };
        let mut cur = Cursor::new(Vec::new());
        out.write_to(&mut cur, ImageFormat::Png)?;
        Ok(cur.into_inner())
    }

    fn rasterize(&self, svg: &str, canvas: &Canvas, path: &Path) -> Result<()> {
        let mut options = usvg::Options::default();
        if let Some(dir) = &self.font_dir {
            options.fontdb_mut().load_fonts_dir(dir);
        }
        options.fontdb_mut().load_system_fonts();
        let tree = usvg::Tree::from_str(svg, &options)?;
        let mut pixmap = tiny_skia::Pixmap::new(canvas.width, canvas.height)
            .ok_or_else(|| anyhow!("invalid canvas"))?;
        resvg::render(
            &tree,
            tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );
        pixmap.save_png(path)?;
        Ok(())
    }
}

fn resolve_format<'a>(template: &'a Template, variant: &str) -> Result<(&'a Canvas, &'a [Layer])> {
    if let Some(format) = template.formats.get(variant) {
        return Ok((&format.canvas, &format.layers));
    }
    if !template.formats.is_empty() {
        return Err(anyhow!("unknown format: {variant}"));
    }
    if variant == "default" {
        return Ok((&template.canvas, &template.layers));
    }
    let canvas = template
        .variants
        .get(variant)
        .ok_or_else(|| anyhow!("unknown variant: {variant}"))?;
    Ok((canvas, &template.layers))
}

fn crop_cover(img: DynamicImage, w: u32, h: u32, fx: f32, fy: f32) -> DynamicImage {
    let (iw, ih) = img.dimensions();
    let scale = (w as f32 / iw as f32).max(h as f32 / ih as f32);
    let rw = (iw as f32 * scale).ceil() as u32;
    let rh = (ih as f32 * scale).ceil() as u32;
    let resized = img.resize_exact(rw, rh, FilterType::Lanczos3);
    let max_x = rw.saturating_sub(w);
    let max_y = rh.saturating_sub(h);
    let x = (clamp(fx) * max_x as f32).round() as u32;
    let y = (clamp(fy) * max_y as f32).round() as u32;
    resized.crop_imm(x, y, w, h)
}

fn px(f: Frame, c: &Canvas) -> (f32, f32, f32, f32) {
    (
        f.x * c.width as f32,
        f.y * c.height as f32,
        f.width * c.width as f32,
        f.height * c.height as f32,
    )
}
fn clamp(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}
fn safe_name(v: &str) -> String {
    v.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}
fn substitute(s: &str, vars: &BTreeMap<String, String>) -> String {
    let mut out = s.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{{{k}}}}}"), v);
    }
    out
}
fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn wrap_text(s: &str, width: f32, size: f32, max_lines: usize) -> Vec<String> {
    let avg = (size * 0.54).max(1.0);
    let max_chars = (width / avg).floor().max(4.0) as usize;
    let mut lines = Vec::new();
    for explicit in s.lines() {
        let mut cur = String::new();
        for word in explicit.split_whitespace() {
            let next = if cur.is_empty() {
                word.to_string()
            } else {
                format!("{cur} {word}")
            };
            if next.chars().count() > max_chars && !cur.is_empty() {
                lines.push(cur);
                cur = word.to_string();
            } else {
                cur = next;
            }
            if lines.len() >= max_lines {
                break;
            }
        }
        if !cur.is_empty() && lines.len() < max_lines {
            lines.push(cur);
        }
        if lines.len() >= max_lines {
            break;
        }
    }
    lines
}

fn valid_template_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}

fn safe_output_filename(raw: &str) -> Result<String> {
    let name = Path::new(raw)
        .file_name()
        .and_then(|x| x.to_str())
        .ok_or_else(|| anyhow!("invalid output filename"))?;
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(anyhow!("invalid output filename"));
    }
    let ext = Path::new(name)
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
        return Err(anyhow!("unsupported output extension"));
    }
    Ok(name.to_string())
}

fn encode_jpeg_limited(src_png: &Path, target: &Path, max_bytes: usize) -> Result<()> {
    let image = image::open(src_png)?.to_rgb8();
    let mut best = Vec::new();
    let mut selected_quality = None;
    for quality in [90u8, 85, 80, 75, 70, 65, 60, 55, 50, 45, 40, 35, 30, 25, 20] {
        let mut buf = Vec::new();
        JpegEncoder::new_with_quality(&mut buf, quality).encode_image(&image)?;
        if buf.len() <= max_bytes {
            best = buf;
            selected_quality = Some(quality);
            break;
        }
        best = buf;
    }
    let quality = selected_quality.ok_or_else(|| {
        anyhow!(
            "jpeg_size_limit_unreachable: {} bytes > {} bytes at minimum quality",
            best.len(),
            max_bytes
        )
    })?;
    fs::write(target, &best)?;
    tracing::debug!(
        path = %target.display(),
        bytes = best.len(),
        quality,
        max_bytes,
        "JPEG written within size limit"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_filename_is_deterministic() {
        assert_eq!(safe_name("CF 73 / Rebut!"), "CF_73___Rebut_");
    }

    #[test]
    fn variables_are_substituted_without_touching_unknowns() {
        let mut vars = BTreeMap::new();
        vars.insert("title".into(), "Hello".into());
        assert_eq!(
            substitute("{{title}} — {{guest}}", &vars),
            "Hello — {{guest}}"
        );
    }

    #[test]
    fn wrapping_respects_line_cap() {
        let lines = wrap_text("one two three four five six seven eight", 130.0, 30.0, 2);
        assert!(lines.len() <= 2);
        assert!(!lines.is_empty());
    }

    #[test]
    fn cover_crop_returns_exact_target_size() {
        let img = DynamicImage::new_rgb8(640, 480);
        let out = crop_cover(img, 320, 180, 0.5, 0.5);
        assert_eq!(out.dimensions(), (320, 180));
    }
}
