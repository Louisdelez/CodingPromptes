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
            // Force a cache refresh first so cached_prompt reflects any edits
            // (including recent MCP writes or just-loaded projects) — otherwise
            // the execution would be recorded with an empty prompt_preview.
            store.update(cx, |s, _| {
                if s.prompt_dirty || s.cached_prompt.is_empty() {
                    s.refresh_cache();
                }
            });
            log::info!("[llm] run_prompt starting model={}", store.read(cx).selected_model);
            let s = store.read(cx);
            let model = s.selected_model.clone();
            let prompt = s.cached_prompt.clone();
            let server_url = s.server_url.clone();
            let tx = s.msg_tx.clone();
            let _ = s;

            state.playground_loading = true;
            state.playground_response.clear();
            // Immediately mirror to the store so the DevTools snapshot reflects
            // the loading state without waiting for the next periodic sync tick.
            store.update(cx, |s, cx| {
                s.playground_loading = true;
                s.playground_response.clear();
                cx.emit(crate::store::StoreEvent::PlaygroundUpdated);
            });

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build().unwrap_or_default();
            let start = std::time::Instant::now();
            let prompt_copy = prompt.clone();
            let model_copy = model.clone();

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
                            // Record the execution so executions_count increments and
                            // Analytics reflects the run.
                            let preview_p = prompt_copy.chars().take(120).collect::<String>();
                            let preview_r = text.chars().take(120).collect::<String>();
                            let exec = crate::types::Execution {
                                model: model_copy,
                                tokens_in: 0,
                                tokens_out: 0,
                                latency_ms: start.elapsed().as_millis() as u64,
                                cost: 0.0,
                                timestamp: chrono::Utc::now().timestamp_millis(),
                                prompt_preview: preview_p,
                                response_preview: preview_r,
                            };
                            let _ = tx.send(AsyncMsg::ExecutionRecorded(exec));
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
            let s = store.read(cx);
            if s.sdd_running {
                return json!({"ok": false, "error": "SDD pipeline already running"});
            }
            log::info!("[llm] run_sdd starting model={}", s.selected_model);
            let sdd_blocks = crate::spec::workflow::find_sdd_blocks(&s.project.blocks);
            let phase_count = sdd_blocks.len();
            if phase_count == 0 {
                return json!({"ok": false, "error": "No SDD blocks in current project"});
            }
            let project_name = s.project.name.clone();
            let model = s.selected_model.clone();
            let server = s.server_url.clone();
            let tx = s.msg_tx.clone();
            let blocks: Vec<(inkwell_core::types::BlockType, String)> = s
                .project
                .blocks
                .iter()
                .filter(|b| b.enabled && b.block_type.is_sdd())
                .map(|b| (b.block_type, b.content.clone()))
                .collect();
            let _ = s;

            state.sdd_running = true;
            store.update(cx, |s, cx| {
                s.sdd_running = true;
                cx.emit(crate::store::StoreEvent::ProjectChanged);
            });

            // Run all SDD phases sequentially in the background.
            // Each phase's output becomes context for the next.
            let _ = crate::app::rt().spawn(async move {
                let client = reqwest::Client::new();
                let mut ctx = crate::spec::generator::SpecContext::from_blocks(&project_name, &blocks);

                for (block_idx, phase) in &sdd_blocks {
                    let (system, user) = crate::spec::workflow::build_llm_messages(
                        *phase,
                        crate::spec::generator::SpecAction::Generate,
                        &ctx,
                    );
                    let body = serde_json::json!({
                        "model": model,
                        "messages": [
                            {"role": "system", "content": system},
                            {"role": "user", "content": user}
                        ],
                        "temperature": 0.3,
                        "max_tokens": 4096,
                        "stream": false,
                    });
                    if let Ok(resp) = crate::app::llm_post(&client, &model, &server, body).send().await {
                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                            let text = crate::llm::parse_llm_response(&model, &data).unwrap_or_default();
                            match phase {
                                crate::spec::generator::SpecPhase::Constitution => ctx.constitution = text.clone(),
                                crate::spec::generator::SpecPhase::Specification => ctx.specification = text.clone(),
                                crate::spec::generator::SpecPhase::Plan => ctx.plan = text.clone(),
                                crate::spec::generator::SpecPhase::Tasks => ctx.tasks = text.clone(),
                                crate::spec::generator::SpecPhase::Implementation => ctx.implementation = text.clone(),
                            }
                            let _ = tx.send(crate::types::AsyncMsg::SddBlockResult {
                                idx: *block_idx,
                                content: text,
                            });
                        }
                    }
                }
                let _ = tx.send(crate::types::AsyncMsg::LlmDone);
            });

            json!({"ok": true, "message": "SDD pipeline triggered", "phases": phase_count})
        }

        "devtools/send_chat" => {
            let message = params["message"].as_str().unwrap_or("").to_string();
            if message.is_empty() {
                return json!({"ok": false, "error": "Empty message"});
            }

            log::info!("[chat] send_chat len={}", message.len());
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
            log::info!("[save] save_project triggered for project={}", state.project.id);
            state.save_pending = true;
            state.save_timer = 1;
            json!({"ok": true, "message": "Save triggered"})
        }

        _ => json!({"error": format!("Unknown action: {}", method)}),
    }
}
