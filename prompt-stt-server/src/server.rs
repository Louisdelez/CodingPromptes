use axum::{
    Json,
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::watch;
use tower_http::cors::CorsLayer;

use crate::models;
use crate::ollama::OllamaState;
use crate::whisper_engine::WhisperEngine;
use crate::database::Database;
use crate::api_routes;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub engine: WhisperEngine,
    pub ollama: OllamaState,
    pub status_tx: watch::Sender<ServerStatus>,
    pub db: Arc<tokio::sync::Mutex<Database>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
    pub model_loaded: bool,
    pub current_model: Option<String>,
    pub transcriptions_count: u64,
}

#[derive(Deserialize)]
pub struct TranscribeRequest {
    pub audio: String,
    pub language: Option<String>,
}

#[derive(Serialize)]
pub struct TranscribeResponse {
    pub text: String,
    pub language: Option<String>,
    pub duration_ms: u64,
}

#[derive(Serialize)]
pub struct ModelsResponse {
    pub available: Vec<ModelEntry>,
    pub active: Option<String>,
}

#[derive(Serialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub size_mb: u64,
    pub installed: bool,
    pub description: String,
}

// --- Health ---

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    stt: SttHealth,
    llm: LlmHealth,
}

#[derive(Serialize)]
struct SttHealth {
    model_loaded: bool,
}

#[derive(Serialize)]
struct LlmHealth {
    ollama_connected: bool,
    ollama_url: String,
    models_count: usize,
}

async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let ollama_status = state.ollama.status.read().await;
    let ollama_config = state.ollama.config.read().await;
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        stt: SttHealth {
            model_loaded: state.engine.is_loaded(),
        },
        llm: LlmHealth {
            ollama_connected: ollama_status.connected,
            ollama_url: ollama_config.url.clone(),
            models_count: ollama_status.models.len(),
        },
    })
}

// --- STT Models ---

async fn list_stt_models(State(state): State<Arc<AppState>>) -> Json<ModelsResponse> {
    let available = models::available_models();
    let entries: Vec<ModelEntry> = available
        .iter()
        .map(|m| ModelEntry {
            id: m.id.clone(),
            name: m.name.clone(),
            size_mb: m.size_mb,
            installed: models::is_model_installed(m),
            description: m.description.clone(),
        })
        .collect();

    let active = state.engine.current_model_path().and_then(|p| {
        available
            .iter()
            .find(|m| p.contains(&m.filename))
            .map(|m| m.id.clone())
    });

    Json(ModelsResponse {
        available: entries,
        active,
    })
}

// --- STT Transcribe ---

async fn transcribe(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TranscribeRequest>,
) -> Result<Json<TranscribeResponse>, (StatusCode, String)> {
    if !state.engine.is_loaded() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "No model loaded. Please load a model first.".into(),
        ));
    }

    use base64::Engine;
    let audio_bytes = base64::engine::general_purpose::STANDARD
        .decode(&req.audio)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64: {e}")))?;

    let audio_f32 = decode_wav_to_f32(&audio_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid audio: {e}")))?;

    let lang = req.language.clone();
    let start = std::time::Instant::now();

    let engine = state.engine.clone();
    let lang_clone = lang.clone();
    let text =
        tokio::task::spawn_blocking(move || engine.transcribe(&audio_f32, lang_clone.as_deref()))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task error: {e}")))?
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(Json(TranscribeResponse {
        text,
        language: lang,
        duration_ms,
    }))
}

// --- Ollama Proxy: Chat Completions ---

async fn proxy_chat_completions(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> impl IntoResponse {
    match state.ollama.proxy_chat(body).await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let headers = resp.headers().clone();
            match resp.bytes().await {
                Ok(body) => {
                    let mut response = (status, body).into_response();
                    // Forward content-type
                    if let Some(ct) = headers.get("content-type") {
                        response.headers_mut().insert("content-type", ct.clone());
                    }
                    response
                }
                Err(e) => (StatusCode::BAD_GATEWAY, format!("Read error: {e}")).into_response(),
            }
        }
        Err(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
    }
}

// --- Ollama Proxy: List Models ---

async fn proxy_list_models(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.ollama.proxy_models().await {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            match resp.bytes().await {
                Ok(body) => {
                    let mut response = (status, body).into_response();
                    response
                        .headers_mut()
                        .insert("content-type", "application/json".parse().unwrap());
                    response
                }
                Err(e) => (StatusCode::BAD_GATEWAY, format!("Read error: {e}")).into_response(),
            }
        }
        Err(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
    }
}

// --- Ollama Status ---

#[derive(Serialize)]
struct OllamaStatusResponse {
    connected: bool,
    url: String,
    models: Vec<OllamaModelEntry>,
    error: Option<String>,
}

#[derive(Serialize)]
struct OllamaModelEntry {
    name: String,
    size_gb: f64,
    parameter_size: Option<String>,
    quantization: Option<String>,
}

async fn ollama_status(State(state): State<Arc<AppState>>) -> Json<OllamaStatusResponse> {
    let status = state.ollama.status.read().await;
    let config = state.ollama.config.read().await;
    Json(OllamaStatusResponse {
        connected: status.connected,
        url: config.url.clone(),
        models: status
            .models
            .iter()
            .map(|m| OllamaModelEntry {
                name: m.name.clone(),
                size_gb: m.size as f64 / 1_073_741_824.0,
                parameter_size: m.parameter_size.clone(),
                quantization: m.quantization_level.clone(),
            })
            .collect(),
        error: status.error.clone(),
    })
}

// --- WAV decoding ---

fn decode_wav_to_f32(data: &[u8]) -> Result<Vec<f32>, String> {
    let cursor = std::io::Cursor::new(data);
    let reader = hound::WavReader::new(cursor).map_err(|e| format!("WAV parse error: {e}"))?;

    let spec = reader.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let max_val = (1 << (spec.bits_per_sample - 1)) as f32;
            reader
                .into_samples::<i32>()
                .filter_map(|s| s.ok())
                .map(|s| s as f32 / max_val)
                .collect()
        }
        hound::SampleFormat::Float => reader
            .into_samples::<f32>()
            .filter_map(|s| s.ok())
            .collect(),
    };

    let mono = if spec.channels == 2 {
        samples
            .chunks(2)
            .map(|c| if c.len() == 2 { (c[0] + c[1]) / 2.0 } else { c[0] })
            .collect()
    } else {
        samples
    };

    if spec.sample_rate != 16000 {
        Ok(simple_resample(&mono, spec.sample_rate, 16000))
    } else {
        Ok(mono)
    }
}

fn simple_resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio) as usize;
    (0..new_len)
        .map(|i| {
            let src_idx = i as f64 * ratio;
            let idx = src_idx as usize;
            let frac = src_idx - idx as f64;
            if idx + 1 < samples.len() {
                samples[idx] * (1.0 - frac as f32) + samples[idx + 1] * frac as f32
            } else {
                samples[idx.min(samples.len() - 1)]
            }
        })
        .collect()
}

// --- Start Server ---

pub async fn start_server(
    port: u16,
    engine: WhisperEngine,
    ollama: OllamaState,
    status_tx: watch::Sender<ServerStatus>,
    db: Database,
) {
    let state = Arc::new(AppState {
        engine,
        ollama: ollama.clone(),
        status_tx: status_tx.clone(),
        db: Arc::new(tokio::sync::Mutex::new(db)),
    });

    // Background task: refresh Ollama status every 5s
    let ollama_bg = ollama.clone();
    tokio::spawn(async move {
        loop {
            ollama_bg.refresh_status().await;
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let cors = CorsLayer::very_permissive();

    let app = Router::new()
        // Health
        .route("/health", get(health))
        // STT
        .route("/transcribe", post(transcribe))
        .route("/models", get(list_stt_models))
        // LLM proxy (OpenAI-compatible)
        .route("/v1/chat/completions", post(proxy_chat_completions))
        .route("/v1/models", get(proxy_list_models))
        // Ollama info
        .route("/ollama/status", get(ollama_status))
        // Data API (JWT auth)
        .nest("/api", api_routes::router())
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    let _ = status_tx.send(ServerStatus {
        running: true,
        port,
        model_loaded: false,
        current_model: None,
        transcriptions_count: 0,
    });

    axum::serve(listener, app).await.unwrap();
}
