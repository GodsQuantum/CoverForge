mod model;
mod render;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use model::{InlineRenderRequest, Layer, LegacyGenerateRequest, RenderRequest, Template};
use render::Renderer;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    process::Command,
    sync::Arc,
};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    renderer: Arc<Renderer>,
    font_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "coverforge=info".into()),
        )
        .init();

    let renderer = Arc::new(Renderer::from_env()?);
    let output_dir = renderer.output_dir().to_path_buf();
    let font_dir =
        PathBuf::from(std::env::var("COVERFORGE_FONT_DIR").unwrap_or_else(|_| "./fonts".into()));
    fs::create_dir_all(&font_dir)?;
    let state = AppState {
        renderer,
        font_dir: font_dir.clone(),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/openapi.json", get(openapi))
        .route("/v1/templates", get(list_templates))
        .route("/v1/templates/{id}", get(get_template).put(put_template))
        .route("/v1/templates/{id}/dataset", get(get_template_dataset))
        .route("/v1/fonts", get(list_fonts).post(upload_font))
        .route("/v1/fonts/{name}", axum::routing::delete(delete_font))
        .route("/v1/asset", get(get_asset))
        .route("/v1/render", post(render))
        .route("/v1/render/preview", post(render_preview))
        .route("/v1/render/batch", post(render_batch))
        .route("/api/generate", post(legacy_generate))
        .nest_service("/outputs", ServeDir::new(output_dir))
        .nest_service("/font-files/custom", ServeDir::new(font_dir))
        .nest_service("/font-files/system", ServeDir::new("/usr/share/fonts"))
        .fallback_service(ServeDir::new("web/build").append_index_html_on_directories(true))
        .layer(DefaultBodyLimit::max(24 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let bind = std::env::var("COVERFORGE_BIND").unwrap_or_else(|_| "127.0.0.1:3099".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "CoverForge listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<Value> {
    Json(json!({
        "ok": true,
        "service": "CoverForge",
        "version": env!("CARGO_PKG_VERSION"),
        "renderer": "rust/resvg",
        "templates_editable": true,
        "fonts_editable": true
    }))
}

async fn openapi() -> Json<Value> {
    Json(json!({
        "openapi": "3.1.0",
        "info": {
            "title": "CoverForge API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "API-first visual cover editor and deterministic Rust renderer"
        },
        "paths": {
            "/health": {
                "get": { "summary": "Service health and version", "responses": { "200": { "description": "Healthy" } } }
            },
            "/v1/templates": {
                "get": { "summary": "List show templates", "responses": { "200": { "description": "Template ids" } } }
            },
            "/v1/templates/{id}": {
                "get": {
                    "summary": "Get the canonical JSON design",
                    "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "responses": { "200": { "description": "Template JSON" } }
                },
                "put": {
                    "summary": "Atomically replace a design",
                    "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Template" } } } },
                    "responses": { "200": { "description": "Saved" } }
                }
            },
            "/v1/templates/{id}/dataset": {
                "get": {
                    "summary": "Discover autofill variables, types and usages",
                    "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "responses": { "200": { "description": "Autofill dataset" } }
                }
            },
            "/v1/fonts": {
                "get": { "summary": "List renderer fonts", "responses": { "200": { "description": "Fonts" } } },
                "post": { "summary": "Upload a custom font", "responses": { "200": { "description": "Font added" } } }
            },
            "/v1/fonts/{name}": {
                "delete": {
                    "summary": "Delete a custom font",
                    "parameters": [{ "name": "name", "in": "path", "required": true, "schema": { "type": "string" } }],
                    "responses": { "200": { "description": "Font deleted" } }
                }
            },
            "/v1/asset": {
                "get": {
                    "summary": "Preview an asset inside configured safe roots",
                    "parameters": [{ "name": "path", "in": "query", "required": true, "schema": { "type": "string" } }],
                    "responses": { "200": { "description": "Asset bytes" } }
                }
            },
            "/v1/render": {
                "post": {
                    "summary": "Render a saved template with autofill variables",
                    "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/RenderRequest" } } } },
                    "responses": { "200": { "description": "Rendered assets" } }
                }
            },
            "/v1/render/preview": {
                "post": {
                    "summary": "Render unsaved inline template JSON through the production Rust engine",
                    "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/InlineRenderRequest" } } } },
                    "responses": { "200": { "description": "Rendered preview assets" } }
                }
            },
            "/v1/render/batch": {
                "post": {
                    "summary": "Render up to 100 saved-template jobs",
                    "responses": { "200": { "description": "Batch render results" } }
                }
            },
            "/api/generate": {
                "post": {
                    "summary": "Legacy AutoPublisher compatibility endpoint",
                    "responses": { "200": { "description": "Rendered asset" } }
                }
            }
        },
        "components": {
            "schemas": {
                "Frame": {
                    "type": "object",
                    "required": ["x", "y", "width", "height"],
                    "properties": {
                        "x": { "type": "number", "minimum": 0, "maximum": 1 },
                        "y": { "type": "number", "minimum": 0, "maximum": 1 },
                        "width": { "type": "number", "minimum": 0, "maximum": 1 },
                        "height": { "type": "number", "minimum": 0, "maximum": 1 }
                    }
                },
                "Template": {
                    "type": "object",
                    "required": ["version", "id", "canvas"],
                    "description": "Canonical CoverForge JSON. Layer types: text, image, rect. Formats may own independent layer stacks.",
                    "additionalProperties": true
                },
                "RenderRequest": {
                    "type": "object",
                    "required": ["template"],
                    "properties": {
                        "template": { "type": "string" },
                        "variables": { "type": "object", "additionalProperties": { "type": "string" } },
                        "variants": { "type": "array", "items": { "type": "string" } },
                        "output_stem": { "type": ["string", "null"] }
                    }
                },
                "InlineRenderRequest": {
                    "type": "object",
                    "required": ["template"],
                    "properties": {
                        "template": { "$ref": "#/components/schemas/Template" },
                        "variables": { "type": "object", "additionalProperties": { "type": "string" } },
                        "variants": { "type": "array", "items": { "type": "string" } },
                        "output_stem": { "type": ["string", "null"] }
                    }
                }
            }
        }
    }))
}

async fn list_templates(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "templates": s.renderer.list_templates()? })))
}

async fn get_template(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Template>, ApiError> {
    Ok(Json(s.renderer.load_template(&id)?))
}

async fn put_template(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(template): Json<Template>,
) -> Result<Json<Value>, ApiError> {
    s.renderer.save_template(&id, &template)?;
    Ok(Json(json!({"ok": true, "template": id})))
}

async fn get_template_dataset(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let template = s.renderer.load_template(&id)?;
    Ok(Json(template_dataset(&template)))
}

async fn list_fonts(State(s): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "fonts": font_catalog(&s.font_dir)? })))
}

#[derive(serde::Deserialize)]
struct AssetQuery {
    path: String,
}

async fn get_asset(
    State(s): State<AppState>,
    Query(query): Query<AssetQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let path = s.renderer.resolve_asset_path(&query.path)?;
    let bytes = tokio::fs::read(&path).await?;
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(mime.as_ref())?);
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=300"),
    );
    Ok((headers, bytes))
}

async fn upload_font(
    State(s): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() != Some("font") {
            continue;
        }
        let original = field.file_name().unwrap_or("font.ttf").to_string();
        let filename = safe_font_filename(&original)?;
        let bytes = field.bytes().await?;
        if bytes.is_empty() || bytes.len() > 20 * 1024 * 1024 {
            return Err(ApiError(anyhow::anyhow!(
                "font must be between 1 byte and 20 MiB"
            )));
        }

        let tmp = s
            .font_dir
            .join(format!(".upload-{}", uuid::Uuid::new_v4().simple()));
        fs::write(&tmp, &bytes)?;
        let scan = Command::new("fc-scan")
            .arg("--format=%{family[0]}\t%{style[0]}")
            .arg(&tmp)
            .output()?;
        if !scan.status.success() || scan.stdout.is_empty() {
            let _ = fs::remove_file(&tmp);
            return Err(ApiError(anyhow::anyhow!(
                "unsupported or invalid font file"
            )));
        }
        let target = s.font_dir.join(&filename);
        fs::rename(&tmp, &target)?;
        let _ = Command::new("fc-cache").arg("-f").arg(&s.font_dir).status();

        let meta = String::from_utf8_lossy(&scan.stdout);
        let mut parts = meta.splitn(2, '\t');
        return Ok(Json(json!({
            "ok": true,
            "filename": filename,
            "family": parts.next().unwrap_or("").trim(),
            "style": parts.next().unwrap_or("").trim(),
            "source": "custom"
        })));
    }
    Err(ApiError(anyhow::anyhow!(
        "multipart field 'font' is required"
    )))
}

async fn delete_font(
    State(s): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let filename = safe_font_filename(&name)?;
    let path = s.font_dir.join(&filename);
    if !path.is_file() {
        return Err(ApiError(anyhow::anyhow!("custom font not found")));
    }
    fs::remove_file(path)?;
    let _ = Command::new("fc-cache").arg("-f").arg(&s.font_dir).status();
    Ok(Json(json!({ "ok": true, "deleted": filename })))
}

async fn render(
    State(s): State<AppState>,
    Json(req): Json<RenderRequest>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::to_value(s.renderer.render(&req)?)?))
}

async fn render_preview(
    State(s): State<AppState>,
    Json(req): Json<InlineRenderRequest>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::to_value(s.renderer.render_inline(&req)?)?))
}

async fn render_batch(
    State(s): State<AppState>,
    Json(reqs): Json<Vec<RenderRequest>>,
) -> Result<Json<Value>, ApiError> {
    if reqs.len() > 100 {
        return Err(ApiError(anyhow::anyhow!("batch limit is 100")));
    }
    let mut results = Vec::with_capacity(reqs.len());
    for req in reqs {
        results.push(s.renderer.render(&req)?);
    }
    Ok(Json(serde_json::to_value(results)?))
}

async fn legacy_generate(
    State(s): State<AppState>,
    Json(req): Json<LegacyGenerateRequest>,
) -> Result<Json<Value>, ApiError> {
    let (template, variant) = legacy_template_parts(&req.template_name)?;
    let mut variables = BTreeMap::new();
    variables.insert("background".into(), req.bg_image);
    for (k, v) in req.fields {
        variables.insert(k, value_to_string(v));
    }
    if let Some(v) = variables.get("PodLogo").cloned() {
        variables.insert("logo".into(), v);
    }
    if let Some(v) = variables
        .get("title_copy")
        .cloned()
        .or_else(|| variables.get("title").cloned())
    {
        variables.insert("title".into(), v);
    }

    let asset = s
        .renderer
        .render_legacy(&template, &variant, &variables, &req.output_filename)?;

    Ok(Json(json!({
        "ok": true,
        "template_name": req.template_name,
        "template_key": template,
        "format_key": variant,
        "width": asset.width,
        "height": asset.height,
        "path": asset.path,
        "n8n_path": if asset.path.starts_with("/srv/storage/") {
            format!("/host{}", asset.path)
        } else {
            asset.path.clone()
        },
        "url": asset.url,
        "filename": asset.filename
    })))
}

fn legacy_template_parts(value: &str) -> anyhow::Result<(String, String)> {
    let slug = template_slug(value);
    for (suffix, format) in [
        ("-tiktok", "vertical"),
        ("-portrait", "feed"),
        ("-square", "square"),
        ("-acast", "acast"),
        ("-ig", "feed"),
        ("-yt", "youtube"),
    ] {
        if let Some(show) = slug.strip_suffix(suffix)
            && !show.is_empty()
        {
            return Ok((show.to_string(), format.to_string()));
        }
    }
    Err(anyhow::anyhow!("unknown legacy template name: {value}"))
}

fn template_slug(value: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for ch in value.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn value_to_string(v: Value) -> String {
    match v {
        Value::String(v) => v,
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn template_dataset(template: &Template) -> Value {
    let mut fields: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    let mut formats = Vec::new();

    if !template.formats.is_empty() {
        for (format, definition) in &template.formats {
            formats.push(json!({
                "id": format,
                "width": definition.canvas.width,
                "height": definition.canvas.height,
                "layers": definition.layers.len()
            }));
            collect_layer_variables(format, &definition.layers, &mut fields);
        }
    } else {
        let variants: Vec<String> = if template.variants.is_empty() {
            vec!["default".into()]
        } else {
            template.variants.keys().cloned().collect()
        };
        for format in variants {
            let canvas = if format == "default" {
                &template.canvas
            } else {
                template.variants.get(&format).unwrap_or(&template.canvas)
            };
            formats.push(json!({
                "id": format,
                "width": canvas.width,
                "height": canvas.height,
                "layers": template.layers.len()
            }));
            collect_layer_variables(&format, &template.layers, &mut fields);
        }
    }

    let fields: Vec<Value> = fields
        .into_iter()
        .map(|(key, (types, usages))| {
            json!({
                "key": key,
                "types": types.into_iter().collect::<Vec<_>>(),
                "required": true,
                "usages": usages.into_iter().collect::<Vec<_>>()
            })
        })
        .collect();

    json!({
        "template": template.id,
        "formats": formats,
        "fields": fields
    })
}

fn collect_layer_variables(
    format: &str,
    layers: &[Layer],
    fields: &mut BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)>,
) {
    for layer in layers {
        match layer {
            Layer::Text { id, text, .. } => {
                for key in variable_keys(text) {
                    let entry = fields.entry(key).or_default();
                    entry.0.insert("text".into());
                    entry.1.insert(format!("{format}:{id}:text"));
                }
            }
            Layer::Image { id, source, .. } => {
                for key in variable_keys(source) {
                    let entry = fields.entry(key).or_default();
                    entry.0.insert("image".into());
                    entry.1.insert(format!("{format}:{id}:source"));
                }
            }
            Layer::Rect { .. } => {}
        }
    }
}

fn variable_keys(input: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = input;
    while let Some(start) = rest.find("{{") {
        rest = &rest[start + 2..];
        let Some(end) = rest.find("}}") else {
            break;
        };
        let key = rest[..end].trim();
        if !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        {
            out.insert(key.to_string());
        }
        rest = &rest[end + 2..];
    }
    out
}

fn safe_font_filename(raw: &str) -> anyhow::Result<String> {
    let file = std::path::Path::new(raw)
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid font filename"))?;
    let ext = std::path::Path::new(file)
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(ext.as_str(), "ttf" | "otf" | "ttc" | "otc") {
        return Err(anyhow::anyhow!(
            "supported font extensions: ttf, otf, ttc, otc"
        ));
    }
    let safe: String = file
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if safe.starts_with('.') || safe.is_empty() {
        return Err(anyhow::anyhow!("invalid font filename"));
    }
    Ok(safe)
}

fn font_catalog(font_dir: &PathBuf) -> anyhow::Result<Vec<Value>> {
    let mut seen = BTreeSet::new();
    let mut fonts = Vec::new();

    // Bundled/system fonts are discovered through fontconfig.
    let output = Command::new("fc-list")
        .arg("--format=%{family[0]}\t%{style[0]}\t%{file}\n")
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!("fc-list failed"));
    }
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.splitn(3, '\t');
        let family = parts.next().unwrap_or("").trim();
        let style = parts.next().unwrap_or("").trim();
        let file = parts.next().unwrap_or("").trim();
        if family.is_empty() || file.is_empty() {
            continue;
        }
        let path = PathBuf::from(file);
        let canonical = path.canonicalize().unwrap_or(path.clone());
        // Custom fonts are added explicitly below so folder drops are visible
        // immediately even before fontconfig refreshes its global cache.
        if canonical.starts_with(font_dir) {
            continue;
        }
        let (source, url, filename) = if let Ok(rel) = canonical.strip_prefix("/usr/share/fonts") {
            let rel = rel.to_string_lossy().trim_start_matches('/').to_string();
            (
                "bundled",
                format!("/font-files/system/{rel}"),
                canonical
                    .file_name()
                    .and_then(|v| v.to_str())
                    .unwrap_or("")
                    .to_string(),
            )
        } else {
            (
                "system",
                String::new(),
                canonical
                    .file_name()
                    .and_then(|v| v.to_str())
                    .unwrap_or("")
                    .to_string(),
            )
        };
        let key = format!("{family}\t{style}\t{file}");
        if !seen.insert(key) {
            continue;
        }
        fonts.push(json!({
            "family": family,
            "style": style,
            "filename": filename,
            "source": source,
            "url": url
        }));
    }

    // Scan the persistent custom directory directly. This also makes fonts
    // copied into the folder by SMB/Syncthing visible without a container restart.
    if font_dir.is_dir() {
        for entry in fs::read_dir(font_dir)? {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !matches!(ext.as_str(), "ttf" | "otf" | "ttc" | "otc") {
                continue;
            }
            let scan = Command::new("fc-scan")
                .arg("--format=%{family[0]}\t%{style[0]}")
                .arg(&path)
                .output()?;
            if !scan.status.success() {
                continue;
            }
            let meta = String::from_utf8_lossy(&scan.stdout);
            let mut parts = meta.splitn(2, '\t');
            let family = parts.next().unwrap_or("").trim();
            let style = parts.next().unwrap_or("").trim();
            if family.is_empty() {
                continue;
            }
            let filename = path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_string();
            let key = format!("custom\t{family}\t{style}\t{filename}");
            if !seen.insert(key) {
                continue;
            }
            fonts.push(json!({
                "family": family,
                "style": style,
                "filename": filename,
                "source": "custom",
                "url": format!("/font-files/custom/{filename}")
            }));
        }
    }

    fonts.sort_by(|a, b| {
        a["family"]
            .as_str()
            .unwrap_or("")
            .cmp(b["family"].as_str().unwrap_or(""))
            .then(
                a["style"]
                    .as_str()
                    .unwrap_or("")
                    .cmp(b["style"].as_str().unwrap_or("")),
            )
    });
    Ok(fonts)
}

struct ApiError(anyhow::Error);

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::warn!(error = %self.0, "request failed");
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": self.0.to_string() })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_template_name_maps_to_show_and_format() {
        assert_eq!(
            legacy_template_parts("DPAFM - YT").unwrap(),
            ("dpafm".into(), "youtube".into())
        );
        assert_eq!(
            legacy_template_parts("DPAFM - Acast").unwrap(),
            ("dpafm".into(), "acast".into())
        );
        assert_eq!(
            legacy_template_parts("CF - IG").unwrap(),
            ("cf".into(), "feed".into())
        );
    }

    #[test]
    fn variable_dataset_parser_is_deterministic() {
        let keys = variable_keys("{{title}} / {{ episode }} / {{title}} / {{bad key}}");
        assert_eq!(
            keys.into_iter().collect::<Vec<_>>(),
            vec!["episode".to_string(), "title".to_string()]
        );
    }
}
