mod api_routes;
mod database;
mod jwt_auth;

use axum::{
    Json, Router,
    body::Bytes,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use database::Database;
use serde::Serialize;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<tokio::sync::Mutex<Database>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

// --- LLM Proxy (optional, forwards to Ollama if configured) ---

async fn proxy_chat(body: Bytes) -> impl IntoResponse {
    let ollama_url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".into());
    let url = format!("{}/v1/chat/completions", ollama_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    match client.post(&url).header("Content-Type", "application/json").body(body.to_vec()).send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let headers = resp.headers().clone();
            match resp.bytes().await {
                Ok(body) => {
                    let mut response = (status, body).into_response();
                    if let Some(ct) = headers.get("content-type") {
                        response.headers_mut().insert("content-type", ct.clone());
                    }
                    response
                }
                Err(e) => (StatusCode::BAD_GATEWAY, format!("Read error: {e}")).into_response(),
            }
        }
        Err(e) => (StatusCode::BAD_GATEWAY, format!("Proxy error: {e}")).into_response(),
    }
}

async fn proxy_models() -> impl IntoResponse {
    let ollama_url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".into());
    let url = format!("{}/v1/models", ollama_url.trim_end_matches('/'));

    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            match resp.bytes().await {
                Ok(body) => {
                    let mut response = (status, body).into_response();
                    response.headers_mut().insert("content-type", "application/json".parse().unwrap());
                    response
                }
                Err(e) => (StatusCode::BAD_GATEWAY, format!("Read error: {e}")).into_response(),
            }
        }
        Err(e) => (StatusCode::BAD_GATEWAY, format!("Proxy error: {e}")).into_response(),
    }
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8910);

    let db = Database::open().expect("Failed to open database");
    let state = Arc::new(AppState {
        db: Arc::new(tokio::sync::Mutex::new(db)),
    });

    let cors = CorsLayer::very_permissive();

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/chat/completions", post(proxy_chat))
        .route("/v1/models", get(proxy_models))
        .nest("/api", api_routes::router())
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    println!("Inkwell API Server v{} listening on {addr}", env!("CARGO_PKG_VERSION"));
    println!("  API:     http://0.0.0.0:{port}/api/");
    println!("  LLM:     http://0.0.0.0:{port}/v1/chat/completions");
    println!("  Health:  http://0.0.0.0:{port}/health");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
