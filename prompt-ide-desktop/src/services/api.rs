use serde::{Deserialize, Serialize};
use crate::models::config::AppConfig;

#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub text: String,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub latency_ms: i64,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

pub async fn call_llm(
    prompt: &str,
    model_id: &str,
    provider: &str,
    config: &AppConfig,
    temperature: f32,
    max_tokens: u32,
) -> Result<ApiResponse, String> {
    let start = std::time::Instant::now();

    match provider {
        "openai" => call_openai(prompt, model_id, &config.openai_key, temperature, max_tokens, start).await,
        "anthropic" => call_anthropic(prompt, model_id, &config.anthropic_key, temperature, max_tokens, start).await,
        "google" => call_google(prompt, model_id, &config.google_key, temperature, max_tokens, start).await,
        "local" => call_local(prompt, model_id, &config.local_server_url, temperature, max_tokens, start).await,
        _ => Err(format!("Unknown provider: {provider}")),
    }
}

async fn call_openai(prompt: &str, model: &str, key: &str, temp: f32, max_tokens: u32, start: std::time::Instant) -> Result<ApiResponse, String> {
    if key.is_empty() { return Err("OpenAI API key missing".into()); }

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": temp,
        "max_tokens": max_tokens,
    });

    let res = client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {key}"))
        .json(&body)
        .send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let err = res.text().await.unwrap_or_default();
        return Err(format!("OpenAI error: {err}"));
    }

    let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(ApiResponse {
        text: data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string(),
        tokens_in: data["usage"]["prompt_tokens"].as_i64().unwrap_or(0),
        tokens_out: data["usage"]["completion_tokens"].as_i64().unwrap_or(0),
        latency_ms: start.elapsed().as_millis() as i64,
    })
}

async fn call_anthropic(prompt: &str, model: &str, key: &str, temp: f32, max_tokens: u32, start: std::time::Instant) -> Result<ApiResponse, String> {
    if key.is_empty() { return Err("Anthropic API key missing".into()); }

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "max_tokens": max_tokens,
        "temperature": temp,
        "messages": [{"role": "user", "content": prompt}],
    });

    let res = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let err = res.text().await.unwrap_or_default();
        return Err(format!("Anthropic error: {err}"));
    }

    let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let text = data["content"].as_array()
        .map(|arr| arr.iter().filter_map(|c| c["text"].as_str()).collect::<Vec<_>>().join(""))
        .unwrap_or_default();

    Ok(ApiResponse {
        text,
        tokens_in: data["usage"]["input_tokens"].as_i64().unwrap_or(0),
        tokens_out: data["usage"]["output_tokens"].as_i64().unwrap_or(0),
        latency_ms: start.elapsed().as_millis() as i64,
    })
}

async fn call_google(prompt: &str, model: &str, key: &str, temp: f32, max_tokens: u32, start: std::time::Instant) -> Result<ApiResponse, String> {
    if key.is_empty() { return Err("Google API key missing".into()); }

    let client = reqwest::Client::new();
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={key}");
    let body = serde_json::json!({
        "contents": [{"parts": [{"text": prompt}]}],
        "generationConfig": {"temperature": temp, "maxOutputTokens": max_tokens},
    });

    let res = client.post(&url).json(&body).send().await.map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let err = res.text().await.unwrap_or_default();
        return Err(format!("Google error: {err}"));
    }

    let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let text = data["candidates"][0]["content"]["parts"].as_array()
        .map(|arr| arr.iter().filter_map(|p| p["text"].as_str()).collect::<Vec<_>>().join(""))
        .unwrap_or_default();

    Ok(ApiResponse {
        text,
        tokens_in: data["usageMetadata"]["promptTokenCount"].as_i64().unwrap_or(0),
        tokens_out: data["usageMetadata"]["candidatesTokenCount"].as_i64().unwrap_or(0),
        latency_ms: start.elapsed().as_millis() as i64,
    })
}

async fn call_local(prompt: &str, model: &str, base_url: &str, temp: f32, max_tokens: u32, start: std::time::Instant) -> Result<ApiResponse, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": temp,
        "max_tokens": max_tokens,
        "stream": false,
    });

    let res = client.post(&url).json(&body).send().await.map_err(|e| format!("Local server: {e}"))?;

    if !res.status().is_success() {
        let err = res.text().await.unwrap_or_default();
        return Err(format!("Local server error: {err}"));
    }

    let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(ApiResponse {
        text: data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string(),
        tokens_in: data["usage"]["prompt_tokens"].as_i64().unwrap_or(0),
        tokens_out: data["usage"]["completion_tokens"].as_i64().unwrap_or(0),
        latency_ms: start.elapsed().as_millis() as i64,
    })
}
