use serde_json::json;
use crate::state::*;

/// Handle action commands on the GPUI main thread.
pub fn handle_action(
    method: &str,
    params: &serde_json::Value,
    state: &mut AppState,
    store: &gpui::Entity<crate::store::AppStore>,
    cx: &mut gpui::Context<crate::app::InkwellApp>,
) -> serde_json::Value {
    match method {
        "devtools/run_prompt" => {
            // Trigger prompt execution same as Ctrl+Enter
            let s = store.read(cx);
            let model = s.selected_model.clone();
            let prompt = s.cached_prompt.clone();
            let server_url = s.server_url.clone();
            let tx = s.msg_tx.clone();
            let _ = s;

            state.playground_loading = true;
            state.playground_response.clear();

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build().unwrap_or_default();

            crate::app::rt().spawn(async move {
                let body = serde_json::json!({
                    "model": model,
                    "messages": [{"role": "user", "content": prompt}],
                    "temperature": 0.7,
                    "max_tokens": 4096,
                    "stream": false,
                });
                let req = crate::app::llm_post(&client, &model, &server_url, body);
                match req.send().await {
                    Ok(resp) => {
                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                            let text = crate::llm::parse_llm_response(&model, &data).unwrap_or_default();
                            let _ = tx.send(AsyncMsg::LlmResponse(text));
                        }
                        let _ = tx.send(AsyncMsg::LlmDone);
                    }
                    Err(e) => { let _ = tx.send(AsyncMsg::LlmError(e.to_string())); }
                }
            });
            json!({"ok": true, "message": "Prompt execution started"})
        }

        "devtools/run_sdd" => {
            state.sdd_running = true;
            store.update(cx, |s, _| s.sdd_running = true);
            // Trigger SDD via the existing message channel
            // The editor_pane SDD handler will pick this up
            json!({"ok": true, "message": "SDD pipeline triggered"})
        }

        "devtools/send_chat" => {
            let message = params["message"].as_str().unwrap_or("").to_string();
            if message.is_empty() {
                return json!({"ok": false, "error": "Empty message"});
            }

            // Push user message
            state.chat_messages.push(("user".to_string(), message.clone()));
            if state.chat_messages.len() > 200 {
                state.chat_messages.drain(..state.chat_messages.len() - 200);
            }

            // Send to LLM
            let s = store.read(cx);
            let model = s.selected_model.clone();
            let system = s.chat_system_prompt.clone();
            let server_url = s.server_url.clone();
            let tx = s.msg_tx.clone();
            let _ = s;

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build().unwrap_or_default();

            crate::app::rt().spawn(async move {
                let body = serde_json::json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": system},
                        {"role": "user", "content": message},
                    ],
                    "temperature": 0.7,
                    "max_tokens": 4096,
                    "stream": false,
                });
                let req = crate::app::llm_post(&client, &model, &server_url, body);
                match req.send().await {
                    Ok(resp) => {
                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                            let text = crate::llm::parse_llm_response(&model, &data).unwrap_or_default();
                            let _ = tx.send(AsyncMsg::LlmResponse(format!("__CHAT__{}", text)));
                        }
                    }
                    Err(e) => { let _ = tx.send(AsyncMsg::LlmError(e.to_string())); }
                }
            });

            store.update(cx, |_, cx| cx.emit(crate::store::StoreEvent::ChatMessageReceived));
            json!({"ok": true})
        }

        "devtools/save_project" => {
            state.save_pending = true;
            state.save_timer = 1; // Trigger save on next sync cycle
            json!({"ok": true, "message": "Save triggered"})
        }

        _ => json!({"error": format!("Unknown action: {}", method)}),
    }
}
