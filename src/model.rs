use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub version: u32,
    pub id: String,
    pub canvas: Canvas,
    #[serde(default)]
    pub variants: BTreeMap<String, Canvas>,
    #[serde(default)]
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub brand: BrandStyle,
    #[serde(default)]
    pub formats: BTreeMap<String, FormatTemplate>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrandStyle {
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub visual_summary: String,
    #[serde(default)]
    pub palette: BTreeMap<String, String>,
    #[serde(default)]
    pub fonts: BTreeMap<String, String>,
    #[serde(default)]
    pub logos: BTreeMap<String, String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatTemplate {
    pub canvas: Canvas,
    #[serde(default)]
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub platform: String,
    #[serde(default)]
    pub suffix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    #[serde(default = "default_background")]
    pub background: String,
}

fn default_background() -> String {
    "#000000".into()
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Layer {
    Image {
        id: String,
        #[serde(default)]
        name: String,
        #[serde(default = "yes")]
        visible: bool,
        #[serde(default)]
        locked: bool,
        frame: Frame,
        source: String,
        #[serde(default = "default_fit")]
        fit: String,
        #[serde(default = "center")]
        focal_x: f32,
        #[serde(default = "center")]
        focal_y: f32,
        #[serde(default = "one")]
        opacity: f32,
    },
    Text {
        id: String,
        #[serde(default)]
        name: String,
        #[serde(default = "yes")]
        visible: bool,
        #[serde(default)]
        locked: bool,
        frame: Frame,
        text: String,
        #[serde(default = "default_font_family")]
        font_family: String,
        #[serde(default = "default_font_weight")]
        font_weight: u16,
        #[serde(default = "default_font_style")]
        font_style: String,
        font_size: f32,
        #[serde(default)]
        auto_fit: bool,
        #[serde(default)]
        min_font_size: Option<f32>,
        #[serde(default = "default_text_color")]
        color: String,
        #[serde(default)]
        stroke_color: Option<String>,
        #[serde(default)]
        stroke_width: f32,
        #[serde(default = "default_align")]
        align: String,
        #[serde(default)]
        max_lines: Option<usize>,
        #[serde(default = "default_line_height")]
        line_height: f32,
        #[serde(default)]
        uppercase: bool,
        #[serde(default)]
        rotation_deg: f32,
        #[serde(default = "one")]
        opacity: f32,
    },
    Rect {
        id: String,
        #[serde(default)]
        name: String,
        #[serde(default = "yes")]
        visible: bool,
        #[serde(default)]
        locked: bool,
        frame: Frame,
        fill: String,
        #[serde(default = "one")]
        opacity: f32,
        #[serde(default)]
        radius: f32,
    },
}

impl Layer {
    pub fn visible(&self) -> bool {
        match self {
            Layer::Image { visible, .. }
            | Layer::Text { visible, .. }
            | Layer::Rect { visible, .. } => *visible,
        }
    }
}

fn default_fit() -> String {
    "cover".into()
}
fn center() -> f32 {
    0.5
}
fn one() -> f32 {
    1.0
}
fn yes() -> bool {
    true
}
fn default_font_family() -> String {
    "League Spartan".into()
}
fn default_font_weight() -> u16 {
    700
}
fn default_font_style() -> String {
    "normal".into()
}
fn default_text_color() -> String {
    "#ffffff".into()
}
fn default_align() -> String {
    "left".into()
}
fn default_line_height() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    pub template: String,
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
    #[serde(default)]
    pub variants: Vec<String>,
    #[serde(default)]
    pub output_stem: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineRenderRequest {
    pub template: Template,
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
    #[serde(default)]
    pub variants: Vec<String>,
    #[serde(default)]
    pub output_stem: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReframeTarget {
    pub id: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReframeRequest {
    pub asset: String,
    pub formats: Vec<ReframeTarget>,
    #[serde(default)]
    pub focal_override: Option<Point>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReframeResult {
    pub format: String,
    pub crop: Frame,
    pub focal: Point,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyGenerateRequest {
    pub template_name: String,
    pub output_filename: String,
    pub bg_image: String,
    #[serde(default)]
    pub fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderedAsset {
    pub variant: String,
    pub width: u32,
    pub height: u32,
    pub filename: String,
    pub path: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderResponse {
    pub ok: bool,
    pub template: String,
    pub assets: Vec<RenderedAsset>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v05_templates_load_without_v06_metadata() {
        let raw = r##"{
          "version":1,
          "id":"legacy",
          "canvas":{"width":1080,"height":1080,"background":"#000000"},
          "variants":{},
          "layers":[],
          "brand":{"display_name":"Legacy"},
          "formats":{"square":{"canvas":{"width":1080,"height":1080,"background":"#000000"},"layers":[]}}
        }"##;
        let template: Template = serde_json::from_str(raw).expect("v0.5 template");
        assert!(template.brand.logos.is_empty());
        let format = template.formats.get("square").unwrap();
        assert!(format.label.is_empty());
        assert!(format.category.is_empty());
        assert!(format.platform.is_empty());
        assert!(format.suffix.is_empty());
    }

    #[test]
    fn starter_brand_contract() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/starter-brand.json");
        let raw = std::fs::read_to_string(path).expect("starter-brand template must exist");
        let template: Template = serde_json::from_str(&raw).expect("valid starter-brand template");
        assert_eq!(template.id, "starter-brand");
        let expected = [
            ("youtube", 1920, 1080),
            ("square", 1080, 1080),
            ("feed", 1080, 1350),
            ("vertical", 1080, 1920),
            ("landscape", 1200, 628),
        ];
        for (id, width, height) in expected {
            let format = template.formats.get(id).expect("required starter format");
            assert_eq!((format.canvas.width, format.canvas.height), (width, height));
            assert!(format.layers.iter().any(
                |layer| matches!(layer, Layer::Image { source, .. } if source == "{{background}}")
            ));
            assert!(format.layers.iter().any(|layer| matches!(layer, Layer::Text { text, auto_fit: true, .. } if text.contains("{{title}}"))));
        }
        let serialized = serde_json::to_string(&template).unwrap();
        for variable in [
            "{{background}}",
            "{{logo}}",
            "{{title}}",
            "{{subtitle}}",
            "{{badge}}",
        ] {
            assert!(serialized.contains(variable), "missing {variable}");
        }
    }

    #[test]
    fn legacy_text_layer_defaults_are_backward_compatible() {
        let raw = r##"{
          "type":"text",
          "id":"title",
          "frame":{"x":0.1,"y":0.2,"width":0.7,"height":0.2},
          "text":"{{title}}",
          "font_size":0.08
        }"##;
        let layer: Layer = serde_json::from_str(raw).expect("legacy layer should deserialize");
        assert!(layer.visible());
        match layer {
            Layer::Text {
                name,
                locked,
                auto_fit,
                font_style,
                opacity,
                ..
            } => {
                assert!(name.is_empty());
                assert!(!locked);
                assert!(!auto_fit);
                assert_eq!(font_style, "normal");
                assert_eq!(opacity, 1.0);
            }
            _ => panic!("expected text layer"),
        }
    }
}
