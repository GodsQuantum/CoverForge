mod model;
mod render;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use model::{LegacyGenerateRequest, RenderRequest, Template};
use render::Renderer;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    renderer: Arc<Renderer>,
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
    let state = AppState { renderer };

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/templates", get(list_templates))
        .route("/v1/templates/{id}", get(get_template).put(put_template))
        .route("/v1/render", post(render))
        .route("/v1/render/batch", post(render_batch))
        .route("/api/generate", post(legacy_generate))
        .nest_service("/outputs", ServeDir::new(output_dir))
        .fallback_service(ServeDir::new("web/build").append_index_html_on_directories(true))
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
        "templates_editable": true
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

async fn render(
    State(s): State<AppState>,
    Json(req): Json<RenderRequest>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::to_value(s.renderer.render(&req)?)?))
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
    let template = template_slug(&req.template_name);
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
        .render_legacy(&template, &variables, &req.output_filename)?;

    Ok(Json(json!({
        "ok": true,
        "template_name": req.template_name,
        "template_key": template,
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
    fn legacy_template_name_is_stable_slug() {
        assert_eq!(template_slug("DPAFM - YT"), "dpafm-yt");
        assert_eq!(template_slug("DPAFM - Acast"), "dpafm-acast");
    }
}
