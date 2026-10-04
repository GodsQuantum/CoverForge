use crate::{
    model::{
        InlineRenderRequest, PackageManifest, PackageManifestAsset, PackageRenderRequest,
        PackageRenderResponse, RenderRequest, RenderedAsset,
    },
    render::Renderer,
};
use anyhow::{Context, Result, anyhow};
use std::collections::BTreeMap;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

enum PackageSource<'a> {
    Saved(&'a str),
    Inline,
}

fn validate_package_source(request: &PackageRenderRequest) -> Result<PackageSource<'_>> {
    match (&request.template, &request.inline_template) {
        (Some(id), None) if !id.trim().is_empty() => Ok(PackageSource::Saved(id)),
        (None, Some(_)) => Ok(PackageSource::Inline),
        _ => Err(anyhow!(
            "exactly one of template or inline_template is required"
        )),
    }
}

fn safe_component(value: &str) -> String {
    let mut out = String::new();
    let mut separator = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            if separator && !out.is_empty() {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
            separator = false;
        } else {
            separator = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() { "asset".into() } else { out }
}

fn unique_archive_name(filename: &str, used: &mut std::collections::BTreeSet<String>) -> String {
    let path = std::path::Path::new(filename);
    let stem = safe_component(path.file_stem().and_then(|v| v.to_str()).unwrap_or("asset"));
    let extension = path
        .extension()
        .and_then(|v| v.to_str())
        .filter(|v| v.chars().all(|c| c.is_ascii_alphanumeric()))
        .map(|v| format!(".{}", v.to_ascii_lowercase()))
        .unwrap_or_default();
    let mut candidate = format!("{stem}{extension}");
    let mut index = 2usize;
    while used.contains(&candidate) {
        candidate = format!("{stem}-{index}{extension}");
        index += 1;
    }
    used.insert(candidate.clone());
    candidate
}

fn write_package_zip(
    zip_path: &std::path::Path,
    manifest: &PackageManifest,
    assets: &[RenderedAsset],
) -> Result<()> {
    use std::io::Write;
    let file = std::fs::File::create(zip_path)?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut used = std::collections::BTreeSet::new();
    for asset in assets {
        let name = unique_archive_name(&asset.filename, &mut used);
        writer.start_file(name, options)?;
        let bytes = std::fs::read(&asset.path)?;
        writer.write_all(&bytes)?;
    }
    writer.start_file("manifest.json", options)?;
    writer.write_all(&serde_json::to_vec_pretty(manifest)?)?;
    writer.finish()?;
    Ok(())
}

fn build_manifest(
    template: &str,
    variables: &BTreeMap<String, String>,
    assets: &[RenderedAsset],
) -> Result<PackageManifest> {
    let generated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("format package timestamp")?;
    Ok(PackageManifest {
        coverforge_version: env!("CARGO_PKG_VERSION").into(),
        generated_at,
        template: template.into(),
        source_image: variables
            .get("background")
            .filter(|value| !value.trim().is_empty())
            .cloned(),
        variables: variables.clone(),
        assets: assets
            .iter()
            .map(|asset| PackageManifestAsset {
                variant: asset.variant.clone(),
                width: asset.width,
                height: asset.height,
                filename: asset.filename.clone(),
            })
            .collect(),
    })
}

pub fn render_package(
    renderer: &Renderer,
    request: &PackageRenderRequest,
) -> Result<PackageRenderResponse> {
    let source = validate_package_source(request)?;
    let mut seen = std::collections::BTreeSet::new();
    let variants: Vec<String> = request
        .variants
        .iter()
        .filter(|variant| seen.insert((*variant).clone()))
        .cloned()
        .collect();

    let rendered = match source {
        PackageSource::Saved(id) => renderer.render(&RenderRequest {
            template: id.to_string(),
            variables: request.variables.clone(),
            variants,
            output_stem: request.output_stem.clone(),
        })?,
        PackageSource::Inline => renderer.render_inline(&InlineRenderRequest {
            template: request
                .inline_template
                .clone()
                .ok_or_else(|| anyhow!("inline template missing after validation"))?,
            variables: request.variables.clone(),
            variants,
            output_stem: request.output_stem.clone(),
        })?,
    };

    let manifest = build_manifest(&rendered.template, &request.variables, &rendered.assets)?;
    let package_dir = renderer.output_dir().join("packages");
    std::fs::create_dir_all(&package_dir)?;
    let stem = request
        .output_stem
        .as_deref()
        .map(safe_component)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| safe_component(&rendered.template));
    let package_filename = format!("{}-{}.zip", stem, uuid::Uuid::new_v4().simple());
    let package_path = package_dir.join(&package_filename);
    write_package_zip(&package_path, &manifest, &rendered.assets)?;

    Ok(PackageRenderResponse {
        ok: true,
        template: rendered.template,
        assets: rendered.assets,
        package_filename: package_filename.clone(),
        package_path: package_path.to_string_lossy().into_owned(),
        package_url: format!("/outputs/packages/{package_filename}"),
        manifest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PackageRenderRequest, Template};
    use std::collections::BTreeMap;

    fn request(template: Option<&str>, inline_template: Option<Template>) -> PackageRenderRequest {
        PackageRenderRequest {
            template: template.map(str::to_string),
            inline_template,
            variables: BTreeMap::new(),
            variants: vec!["youtube".into()],
            output_stem: None,
        }
    }

    #[test]
    fn manifest_describes_rendered_assets() -> anyhow::Result<()> {
        let mut variables = BTreeMap::new();
        variables.insert("background".into(), "/srv/source.jpg".into());
        variables.insert("title".into(), "Hello".into());
        let assets = vec![
            crate::model::RenderedAsset {
                variant: "youtube".into(),
                width: 1920,
                height: 1080,
                filename: "demo-youtube.png".into(),
                path: "/tmp/demo-youtube.png".into(),
                url: "/outputs/demo-youtube.png".into(),
            },
            crate::model::RenderedAsset {
                variant: "square".into(),
                width: 1080,
                height: 1080,
                filename: "demo-square.png".into(),
                path: "/tmp/demo-square.png".into(),
                url: "/outputs/demo-square.png".into(),
            },
        ];
        let manifest = build_manifest("starter-brand", &variables, &assets)?;
        assert_eq!(manifest.coverforge_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(manifest.template, "starter-brand");
        assert_eq!(manifest.source_image.as_deref(), Some("/srv/source.jpg"));
        assert_eq!(manifest.variables, variables);
        assert_eq!(manifest.assets.len(), 2);
        assert_eq!(manifest.assets[0].variant, "youtube");
        assert_eq!(
            (manifest.assets[0].width, manifest.assets[0].height),
            (1920, 1080)
        );
        assert_eq!(manifest.assets[1].filename, "demo-square.png");
        assert!(manifest.generated_at.contains('T'));
        assert!(manifest.generated_at.ends_with('Z'));
        Ok(())
    }

    #[test]
    fn package_names_are_safe_and_unique() -> anyhow::Result<()> {
        let base =
            std::env::temp_dir().join(format!("coverforge-package-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base)?;
        let a = base.join("a.png");
        let b = base.join("b.png");
        let c = base.join("c.png");
        std::fs::write(&a, b"a")?;
        std::fs::write(&b, b"b")?;
        std::fs::write(&c, b"c")?;
        let assets = vec![
            crate::model::RenderedAsset {
                variant: "../../escape".into(),
                width: 1,
                height: 1,
                filename: "../../escape.png".into(),
                path: a.to_string_lossy().into_owned(),
                url: "".into(),
            },
            crate::model::RenderedAsset {
                variant: "youtube".into(),
                width: 1,
                height: 1,
                filename: "youtube.png".into(),
                path: b.to_string_lossy().into_owned(),
                url: "".into(),
            },
            crate::model::RenderedAsset {
                variant: "youtube".into(),
                width: 1,
                height: 1,
                filename: "youtube.png".into(),
                path: c.to_string_lossy().into_owned(),
                url: "".into(),
            },
        ];
        let manifest = build_manifest("starter-brand", &BTreeMap::new(), &assets)?;
        let zip_path = base.join("package.zip");
        write_package_zip(&zip_path, &manifest, &assets)?;
        let file = std::fs::File::open(&zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut names = Vec::new();
        for index in 0..archive.len() {
            names.push(archive.by_index(index)?.name().to_string());
        }
        assert!(names.contains(&"manifest.json".to_string()));
        assert_eq!(names.len(), 4);
        let image_names: Vec<_> = names
            .iter()
            .filter(|name| *name != "manifest.json")
            .collect();
        let unique: std::collections::BTreeSet<_> = image_names.iter().copied().collect();
        assert_eq!(image_names.len(), unique.len());
        for name in names {
            assert!(!name.contains(".."));
            assert!(!name.starts_with('/'));
        }
        std::fs::remove_dir_all(base)?;
        Ok(())
    }

    #[test]
    fn validates_exactly_one_template_source() {
        assert!(validate_package_source(&request(None, None)).is_err());
        let inline = Template {
            version: 1,
            id: "inline".into(),
            canvas: crate::model::Canvas {
                width: 10,
                height: 10,
                background: "#000000".into(),
            },
            variants: BTreeMap::new(),
            layers: Vec::new(),
            brand: Default::default(),
            formats: BTreeMap::new(),
        };
        assert!(validate_package_source(&request(Some("saved"), Some(inline.clone()))).is_err());
        assert!(validate_package_source(&request(Some("saved"), None)).is_ok());
        assert!(validate_package_source(&request(None, Some(inline))).is_ok());
    }
}
