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
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatTemplate {
    pub canvas: Canvas,
    #[serde(default)]
    pub layers: Vec<Layer>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
        frame: Frame,
        text: String,
        #[serde(default = "default_font_family")]
        font_family: String,
        #[serde(default = "default_font_weight")]
        font_weight: u16,
        font_size: f32,
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
    },
    Rect {
        id: String,
        frame: Frame,
        fill: String,
        #[serde(default = "one")]
        opacity: f32,
        #[serde(default)]
        radius: f32,
    },
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
fn default_font_family() -> String {
    "League Spartan".into()
}
fn default_font_weight() -> u16 {
    700
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
