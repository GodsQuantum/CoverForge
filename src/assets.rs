use anyhow::{Context, Result, anyhow};
use image::{DynamicImage, ImageFormat, ImageReader};
use serde::Serialize;
use std::{
    fs,
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
};

pub const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;
pub const MAX_DECODED_PIXELS: u64 = 50_000_000;
const MAX_SVG_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct UploadedAsset {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub url: String,
    pub mime: String,
    pub width: u32,
    pub height: u32,
    pub kind: String,
}

pub struct AssetStore {
    pub(crate) root: PathBuf,
}

impl AssetStore {
    pub fn new(output_dir: &Path) -> Result<Self> {
        let root = output_dir.join("uploads");
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn save_upload(
        &self,
        filename: &str,
        _mime: Option<&str>,
        bytes: &[u8],
    ) -> Result<UploadedAsset> {
        if bytes.len() > MAX_UPLOAD_BYTES {
            return Err(anyhow!("asset exceeds upload byte limit"));
        }
        let safe = safe_upload_filename(filename)?;
        let extension = validate_extension(&safe)?;
        let id = uuid::Uuid::new_v4().simple().to_string();
        let stored_name = format!("{id}-{safe}");
        let path = self.root.join(&stored_name);

        if extension == "svg" {
            let (width, height) = validate_svg_logo(bytes)?;
            fs::write(&path, bytes)?;
            return Ok(UploadedAsset {
                id,
                filename: stored_name.clone(),
                path: path.to_string_lossy().into_owned(),
                url: format!("/uploads/{stored_name}"),
                mime: "image/svg+xml".into(),
                width,
                height,
                kind: "logo".into(),
            });
        }

        let (format, mime) = sniff_raster_format(bytes)?;
        let decoded = decode_raster_bytes_safely(bytes, format)?;
        use image::GenericImageView;
        let (width, height) = decoded.dimensions();
        fs::write(&path, bytes)?;

        Ok(UploadedAsset {
            id,
            filename: stored_name.clone(),
            path: path.to_string_lossy().into_owned(),
            url: format!("/uploads/{stored_name}"),
            mime: mime.into(),
            width,
            height,
            kind: "image".into(),
        })
    }
}

pub fn load_raster_safely(path: &Path) -> Result<DynamicImage> {
    let bytes = fs::read(path)?;
    let (format, _) = sniff_raster_format(&bytes)?;
    decode_raster_bytes_safely(&bytes, format)
}

fn sniff_raster_format(bytes: &[u8]) -> Result<(ImageFormat, &'static str)> {
    let format = image::guess_format(bytes).context("unsupported or invalid raster")?;
    match format {
        ImageFormat::Png => Ok((format, "image/png")),
        ImageFormat::Jpeg => Ok((format, "image/jpeg")),
        ImageFormat::WebP => Ok((format, "image/webp")),
        _ => Err(anyhow!("unsupported raster format")),
    }
}

fn raster_dimensions(bytes: &[u8], format: ImageFormat) -> Result<(u32, u32)> {
    let reader = ImageReader::with_format(Cursor::new(bytes), format);
    let (width, height) = reader.into_dimensions()?;
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_DECODED_PIXELS {
        return Err(anyhow!(
            "decoded pixel limit exceeded: {pixels} > {MAX_DECODED_PIXELS}"
        ));
    }
    Ok((width, height))
}

fn decode_raster_bytes_safely(bytes: &[u8], format: ImageFormat) -> Result<DynamicImage> {
    raster_dimensions(bytes, format)?;
    let mut image = image::load_from_memory_with_format(bytes, format)?;
    if format == ImageFormat::Jpeg {
        let orientation = exif::Reader::new()
            .read_from_container(&mut BufReader::new(Cursor::new(bytes)))
            .ok()
            .and_then(|metadata| {
                metadata
                    .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
                    .and_then(|field| field.value.get_uint(0))
            })
            .unwrap_or(1);
        image = apply_exif_orientation(image, orientation);
    }
    Ok(image)
}

fn apply_exif_orientation(image: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => image.fliph(),
        3 => image.rotate180(),
        4 => image.flipv(),
        5 => image.rotate90().fliph(),
        6 => image.rotate90(),
        7 => image.rotate270().fliph(),
        8 => image.rotate270(),
        _ => image,
    }
}

pub(crate) fn validate_svg_logo(bytes: &[u8]) -> Result<(u32, u32)> {
    if bytes.len() > MAX_SVG_BYTES {
        return Err(anyhow!("SVG exceeds 2 MiB limit"));
    }
    let svg = std::str::from_utf8(bytes).context("SVG must be UTF-8")?;
    let lower = svg.to_ascii_lowercase();
    for forbidden in [
        "<script",
        "<foreignobject",
        "<iframe",
        "<object",
        "<embed",
        "<image",
        "javascript:",
        "file://",
    ] {
        if lower.contains(forbidden) {
            return Err(anyhow!("unsafe SVG content"));
        }
    }
    if has_svg_event_handler(&lower) || has_external_svg_href(&lower) {
        return Err(anyhow!("unsafe SVG reference or event handler"));
    }

    let root_end = lower.find('>').ok_or_else(|| anyhow!("invalid SVG root"))?;
    let root = &lower[..=root_end];
    let has_view_box = root.contains("viewbox=");
    let has_dimensions = root.contains("width=") && root.contains("height=");
    if !has_view_box && !has_dimensions {
        return Err(anyhow!("SVG requires viewBox or explicit dimensions"));
    }

    let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default())
        .context("invalid SVG")?;
    let size = tree.size();
    let width = size.width().ceil().max(1.0) as u32;
    let height = size.height().ceil().max(1.0) as u32;
    if width == 0 || height == 0 {
        return Err(anyhow!("invalid SVG dimensions"));
    }
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_DECODED_PIXELS {
        return Err(anyhow!("SVG pixel limit exceeded"));
    }
    Ok((width, height))
}

fn has_svg_event_handler(lower: &str) -> bool {
    let bytes = lower.as_bytes();
    let mut i = 0;
    while i + 3 < bytes.len() {
        if bytes[i].is_ascii_whitespace() && bytes[i + 1] == b'o' && bytes[i + 2] == b'n' {
            let mut j = i + 3;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-') {
                j += 1;
            }
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'=' {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn has_external_svg_href(lower: &str) -> bool {
    let mut rest = lower;
    while let Some(pos) = rest.find("href") {
        rest = &rest[pos + 4..];
        let trimmed = rest.trim_start();
        if !trimmed.starts_with('=') {
            continue;
        }
        let value = trimmed[1..].trim_start();
        let quote = value.as_bytes().first().copied();
        if !matches!(quote, Some(b'\'' | b'"')) {
            return true;
        }
        let q = quote.unwrap() as char;
        let after = &value[1..];
        let Some(end) = after.find(q) else {
            return true;
        };
        let target = after[..end].trim();
        if !target.starts_with('#') {
            return true;
        }
        rest = &after[end + 1..];
    }
    false
}

fn validate_extension(filename: &str) -> Result<String> {
    let extension = Path::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| anyhow!("missing asset extension"))?;
    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "svg" => Ok(extension),
        _ => Err(anyhow!("unsupported asset extension")),
    }
}

fn safe_upload_filename(filename: &str) -> Result<String> {
    let file_name = Path::new(filename)
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("invalid asset filename"))?;
    let extension = validate_extension(file_name)?;
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("invalid asset filename"))?;
    let mut slug = String::new();
    let mut separator = false;
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            if separator && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(ch.to_ascii_lowercase());
            separator = false;
        } else {
            separator = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        return Err(anyhow!("invalid asset filename"));
    }
    Ok(format!("{slug}.{extension}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_upload_names() -> anyhow::Result<()> {
        assert_eq!(
            safe_upload_filename("../../brand photo.png")?,
            "brand-photo.png"
        );
        assert!(safe_upload_filename("../..").is_err());
        assert!(validate_extension("photo.exe").is_err());
        assert!(!safe_upload_filename("été photo.png")?.contains('/'));
        Ok(())
    }

    #[test]
    fn asset_store_creates_upload_root() -> anyhow::Result<()> {
        let base = std::env::temp_dir().join(format!("coverforge-assets-{}", uuid::Uuid::new_v4()));
        let store = AssetStore::new(&base)?;
        assert!(store.root.exists());
        assert_eq!(store.root, base.join("uploads"));
        std::fs::remove_dir_all(base)?;
        Ok(())
    }

    fn png_bytes(width: u32, height: u32) -> anyhow::Result<Vec<u8>> {
        let image = image::DynamicImage::new_rgb8(width, height);
        let mut cursor = std::io::Cursor::new(Vec::new());
        image.write_to(&mut cursor, image::ImageFormat::Png)?;
        Ok(cursor.into_inner())
    }

    fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = 0xffff_ffffu32;
        for &byte in bytes {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                let mask = 0u32.wrapping_sub(crc & 1);
                crc = (crc >> 1) ^ (0xedb8_8320 & mask);
            }
        }
        !crc
    }

    fn oversized_png_header(width: u32, height: u32) -> Vec<u8> {
        let mut png = png_bytes(1, 1).expect("seed png");
        png[16..20].copy_from_slice(&width.to_be_bytes());
        png[20..24].copy_from_slice(&height.to_be_bytes());
        let crc = crc32(&png[12..29]);
        png[29..33].copy_from_slice(&crc.to_be_bytes());
        png
    }

    #[test]
    fn reads_raster_metadata() -> anyhow::Result<()> {
        let base = std::env::temp_dir().join(format!("coverforge-assets-{}", uuid::Uuid::new_v4()));
        let store = AssetStore::new(&base)?;
        let asset = store.save_upload("sample.png", Some("image/png"), &png_bytes(3, 2)?)?;
        assert_eq!((asset.width, asset.height), (3, 2));
        assert_eq!(asset.mime, "image/png");
        assert_eq!(asset.kind, "image");
        assert!(std::path::Path::new(&asset.path).exists());
        std::fs::remove_dir_all(base)?;
        Ok(())
    }

    #[test]
    fn rejects_raster_over_pixel_limit() -> anyhow::Result<()> {
        let base = std::env::temp_dir().join(format!("coverforge-assets-{}", uuid::Uuid::new_v4()));
        let store = AssetStore::new(&base)?;
        let bytes = oversized_png_header(10_000, 6_000);
        let err = store
            .save_upload("huge.png", Some("image/png"), &bytes)
            .unwrap_err();
        assert!(err.to_string().contains("pixel"));
        std::fs::remove_dir_all(base)?;
        Ok(())
    }

    #[test]
    fn rejects_unsupported_raster() -> anyhow::Result<()> {
        let base = std::env::temp_dir().join(format!("coverforge-assets-{}", uuid::Uuid::new_v4()));
        let store = AssetStore::new(&base)?;
        assert!(
            store
                .save_upload("fake.png", Some("image/png"), b"not an image")
                .is_err()
        );
        std::fs::remove_dir_all(base)?;
        Ok(())
    }

    #[test]
    fn normalizes_exif_orientation() -> anyhow::Result<()> {
        use image::GenericImageView;
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/orientation-6.jpg");
        let image = load_raster_safely(&fixture)?;
        assert_eq!(image.dimensions(), (3, 2));
        Ok(())
    }

    #[test]
    fn rejects_active_or_external_svg() {
        for svg in [
            r#"<svg viewBox="0 0 10 10" onload="alert(1)"></svg>"#,
            r#"<svg viewBox="0 0 10 10"><script>alert(1)</script></svg>"#,
            r#"<svg viewBox="0 0 10 10"><image href="https://example.com/a.png"/></svg>"#,
            r#"<svg viewBox="0 0 10 10"><use href="file:///etc/passwd#x"/></svg>"#,
            r#"<svg viewBox="0 0 10 10"><a href="javascript:alert(1)"/></svg>"#,
        ] {
            assert!(validate_svg_logo(svg.as_bytes()).is_err(), "{svg}");
        }
    }
}
