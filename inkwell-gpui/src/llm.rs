/// LLM and STT routing — calls APIs directly with local keys, falls back to server proxy.

/// Determine the API endpoint and auth header for a given model.
pub fn llm_endpoint(model: &str, api_key_openai: &str, api_key_anthropic: &str, api_key_google: &str, server_url: &str)
    -> (String, Vec<(String, String)>)
{
    // Route based on model name
    if model.starts_with("claude") && !api_key_anthropic.is_empty() {
        // Anthropic Messages API
        (
            "https://api.anthropic.com/v1/messages".into(),
            vec![
                ("x-api-key".into(), api_key_anthropic.to_string()),
                ("anthropic-version".into(), "2023-06-01".into()),
            ],
        )
    } else if model.starts_with("gemini") && !api_key_google.is_empty() {
        // Google Gemini API (OpenAI-compatible endpoint)
        (
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions".into(),
            vec![
                ("Authorization".into(), format!("Bearer {}", api_key_google)),
            ],
        )
    } else if (model.starts_with("gpt") || model.starts_with("o1") || model.starts_with("o3") || model.starts_with("o4")) && !api_key_openai.is_empty() {
        // OpenAI API
        (
            "https://api.openai.com/v1/chat/completions".into(),
            vec![
                ("Authorization".into(), format!("Bearer {}", api_key_openai)),
            ],
        )
    } else {
        // Fallback: local server proxy (Ollama or other local LLM)
        (
            format!("{server_url}/v1/chat/completions"),
            vec![],
        )
    }
}

/// Build the request body for a chat completion.
/// For Anthropic, converts to Messages API format.
pub fn build_llm_body(model: &str, messages: &[serde_json::Value], temperature: f32, max_tokens: u32, stream: bool) -> serde_json::Value {
    if model.starts_with("claude") {
        // Anthropic Messages API format
        let system = messages.iter()
            .filter(|m| m["role"] == "system")
            .map(|m| m["content"].as_str().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n\n");
        let msgs: Vec<serde_json::Value> = messages.iter()
            .filter(|m| m["role"] != "system")
            .cloned()
            .collect();
        let mut body = serde_json::json!({
            "model": model,
            "messages": if msgs.is_empty() { messages.to_vec() } else { msgs },
            "temperature": temperature,
            "max_tokens": max_tokens,
            "stream": stream,
        });
        if !system.is_empty() {
            body["system"] = serde_json::json!(system);
        }
        body
    } else {
        // OpenAI / Gemini / local — standard format
        serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "stream": stream,
        })
    }
}

/// Parse a chat completion response (handles both OpenAI and Anthropic formats).
pub fn parse_llm_response(model: &str, data: &serde_json::Value) -> Option<String> {
    if model.starts_with("claude") {
        // Anthropic format: data.content[0].text
        data["content"][0]["text"].as_str().map(|s| s.to_string())
            .or_else(|| data["choices"][0]["message"]["content"].as_str().map(|s| s.to_string()))
    } else {
        // OpenAI format: data.choices[0].message.content
        data["choices"][0]["message"]["content"].as_str().map(|s| s.to_string())
    }
}

/// STT endpoint based on provider and local keys.
#[allow(dead_code)]
pub fn stt_endpoint(provider: &crate::state::SttProvider, api_key_openai: &str, server_url: &str)
    -> (String, Vec<(String, String)>)
{
    match provider {
        crate::state::SttProvider::OpenaiWhisper if !api_key_openai.is_empty() => (
            "https://api.openai.com/v1/audio/transcriptions".into(),
            vec![("Authorization".into(), format!("Bearer {}", api_key_openai))],
        ),
        crate::state::SttProvider::Groq => (
            "https://api.groq.com/openai/v1/audio/transcriptions".into(),
            vec![], // needs groq key — fallback to server
        ),
        crate::state::SttProvider::Deepgram => (
            "https://api.deepgram.com/v1/listen".into(),
            vec![], // needs deepgram key — fallback to server
        ),
        _ => (
            // Local server Whisper
            format!("{server_url}/transcribe"),
            vec![],
        ),
    }
}


/// Load API keys from local settings (called from async context).
pub fn load_local_keys() -> (String, String, String, String) {
    let settings = crate::persistence::load_settings();
    let session = crate::persistence::load_session();
    (
        settings.api_key_openai,
        settings.api_key_anthropic,
        settings.api_key_google,
        session.server_url,
    )
}

