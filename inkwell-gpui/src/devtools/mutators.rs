use serde_json::json;
use crate::state::*;
use crate::store::StoreEvent;

/// Strictly parse an integer index from a JSON param.
/// Rejects negative, non-integer, and missing values.
fn parse_index(params: &serde_json::Value, key: &str) -> Result<usize, serde_json::Value> {
    let v = &params[key];
    if v.is_null() {
        return Err(json!({"ok": false, "error": format!("Missing '{}' parameter", key)}));
    }
    if let Some(n) = v.as_u64() {
        return Ok(n as usize);
    }
    if let Some(n) = v.as_i64() {
        if n < 0 {
            return Err(json!({"ok": false, "error": format!("'{}' must be >= 0 (got {})", key, n)}));
        }
        return Ok(n as usize);
    }
    Err(json!({"ok": false, "error": format!("'{}' must be a non-negative integer", key)}))
}

fn check_bounds(idx: usize, len: usize) -> Option<serde_json::Value> {
    if idx >= len {
        Some(json!({"ok": false, "error": format!("Block index {} out of range (len={})", idx, len)}))
    } else {
        None
    }
}

/// Handle write commands on the GPUI main thread.
/// Called from poll_messages when a DevToolsCommand is received.
pub fn handle_write(
    method: &str,
    params: &serde_json::Value,
    state: &mut AppState,
    store: &gpui::Entity<crate::store::AppStore>,
    cx: &mut gpui::Context<crate::app::InkwellApp>,
) -> serde_json::Value {
    match method {
        "devtools/set_block" => {
            let idx = match parse_index(params, "index") {
                Ok(i) => i,
                Err(e) => return e,
            };
            let len = state.project.blocks.len();
            if let Some(err) = check_bounds(idx, len) { return err; }
            let content = match params["content"].as_str() {
                Some(s) => s.to_string(),
                None => return json!({"ok": false, "error": "Missing or non-string 'content' parameter"}),
            };
            state.project.blocks[idx].content = content.clone();
            state.prompt_dirty = true;
            // Reset input entity to force re-creation from the new content on next sync
            if let Some(inp) = state.block_inputs.get_mut(idx) { *inp = None; }
            // Also push to store immediately so BlockEditor observers see it.
            store.update(cx, |s, cx| {
                if let Some(b) = s.project.blocks.get_mut(idx) {
                    b.content = content;
                }
                s.prompt_dirty = true;
                s.refresh_cache();
                cx.emit(StoreEvent::BlockContentChanged(idx));
                cx.emit(StoreEvent::PromptCacheUpdated);
            });
            json!({"ok": true})
        }

        "devtools/add_block" => {
            let bt_str = match params["block_type"].as_str() {
                Some(s) => s,
                None => return json!({"ok": false, "error": "Missing 'block_type' parameter"}),
            };
            let block_type = match inkwell_core::types::BlockType::from_name(bt_str) {
                Some(bt) => bt,
                None => return json!({
                    "ok": false,
                    "error": format!("Unknown block_type '{}'. Valid: {:?}",
                        bt_str, inkwell_core::types::BlockType::ALL_NAMES),
                }),
            };
            let content = params["content"].as_str().unwrap_or("").to_string();
            let mut block = Block::new(block_type);
            block.content = content;
            let idx = state.project.blocks.len();
            state.project.blocks.push(block.clone());
            state.block_inputs.push(None);
            state.prompt_dirty = true;
            store.update(cx, |s, cx| {
                s.project.blocks.push(Block {
                    id: block.id.clone(),
                    block_type: block.block_type,
                    content: block.content.clone(),
                    enabled: block.enabled,
                    editing: false,
                });
                s.prompt_dirty = true;
                s.refresh_cache();
                cx.emit(StoreEvent::ProjectChanged);
                cx.emit(StoreEvent::PromptCacheUpdated);
            });
            json!({"ok": true, "index": idx})
        }

        "devtools/delete_block" => {
            let idx = match parse_index(params, "index") {
                Ok(i) => i,
                Err(e) => return e,
            };
            let len = state.project.blocks.len();
            if let Some(err) = check_bounds(idx, len) { return err; }
            state.project.blocks.remove(idx);
            if idx < state.block_inputs.len() { state.block_inputs.remove(idx); }
            state.prompt_dirty = true;
            store.update(cx, |s, cx| {
                if idx < s.project.blocks.len() { s.project.blocks.remove(idx); }
                s.prompt_dirty = true;
                s.refresh_cache();
                cx.emit(StoreEvent::ProjectChanged);
                cx.emit(StoreEvent::PromptCacheUpdated);
            });
            json!({"ok": true})
        }

        "devtools/toggle_block" => {
            let idx = match parse_index(params, "index") {
                Ok(i) => i,
                Err(e) => return e,
            };
            let len = state.project.blocks.len();
            if let Some(err) = check_bounds(idx, len) { return err; }
            state.project.blocks[idx].enabled = !state.project.blocks[idx].enabled;
            let enabled = state.project.blocks[idx].enabled;
            state.prompt_dirty = true;
            store.update(cx, |s, cx| {
                if let Some(b) = s.project.blocks.get_mut(idx) { b.enabled = enabled; }
                s.prompt_dirty = true;
                s.refresh_cache();
                cx.emit(StoreEvent::ProjectChanged);
                cx.emit(StoreEvent::PromptCacheUpdated);
            });
            json!({"ok": true, "enabled": enabled})
        }

        "devtools/reorder_blocks" => {
            let from = match parse_index(params, "from") {
                Ok(i) => i,
                Err(e) => return e,
            };
            let to = match parse_index(params, "to") {
                Ok(i) => i,
                Err(e) => return e,
            };
            let len = state.project.blocks.len();
            if from >= len || to >= len {
                return json!({"ok": false, "error": format!("Index out of range (len={})", len)});
            }
            if from == to {
                return json!({"ok": true, "note": "no-op"});
            }
            let block = state.project.blocks.remove(from);
            let insert_at = if from < to { to.saturating_sub(1) } else { to };
            state.project.blocks.insert(insert_at, block);
            // Rebuild input vec in new order
            if from < state.block_inputs.len() {
                let inp = state.block_inputs.remove(from);
                let at = insert_at.min(state.block_inputs.len());
                state.block_inputs.insert(at, inp);
            }
            state.prompt_dirty = true;
            // Push the reordered blocks into the store (full rebuild)
            let new_blocks: Vec<Block> = state.project.blocks.iter().map(|b| Block {
                id: b.id.clone(),
                block_type: b.block_type,
                content: b.content.clone(),
                enabled: b.enabled,
                editing: false,
            }).collect();
            store.update(cx, |s, cx| {
                s.project.blocks = new_blocks;
                s.prompt_dirty = true;
                s.refresh_cache();
                cx.emit(StoreEvent::ProjectChanged);
                cx.emit(StoreEvent::PromptCacheUpdated);
            });
            json!({"ok": true})
        }

        "devtools/select_tab" => {
            let tab_str = params["tab"].as_str().unwrap_or("");
            let tab = match tab_str {
                "Preview" => Some(RightTab::Preview),
                "Playground" => Some(RightTab::Playground),
                "Stt" => Some(RightTab::Stt),
                "History" => Some(RightTab::History),
                "Export" => Some(RightTab::Export),
                "Fleet" => Some(RightTab::Fleet),
                "Terminal" => Some(RightTab::Terminal),
                "Optimize" => Some(RightTab::Optimize),
                "Lint" => Some(RightTab::Lint),
                "Chat" => Some(RightTab::Chat),
                "Analytics" => Some(RightTab::Analytics),
                "Collab" => Some(RightTab::Collab),
                "Sdd" => Some(RightTab::Sdd),
                "Chain" => Some(RightTab::Chain),
                _ => None,
            };
            if let Some(t) = tab {
                store.update(cx, |s, cx| {
                    s.right_tab = t;
                    s.right_open = true;
                    cx.emit(StoreEvent::SwitchRightTab(t));
                });
                json!({"ok": true})
            } else {
                json!({"ok": false, "error": format!("Unknown tab: {}", tab_str)})
            }
        }

        "devtools/toggle_panel" => {
            let panel = params["panel"].as_str().unwrap_or("");
            match panel {
                "left" => {
                    store.update(cx, |s, cx| {
                        s.left_open = !s.left_open;
                        cx.emit(StoreEvent::ProjectChanged);
                    });
                    let open = store.read(cx).left_open;
                    json!({"ok": true, "open": open})
                }
                "right" => {
                    store.update(cx, |s, cx| {
                        s.right_open = !s.right_open;
                        cx.emit(StoreEvent::ProjectChanged);
                    });
                    let open = store.read(cx).right_open;
                    json!({"ok": true, "open": open})
                }
                _ => json!({"ok": false, "error": "Unknown panel (use 'left' or 'right')"}),
            }
        }

        "devtools/set_model" => {
            let model = match params["model"].as_str() {
                Some(s) => s.to_string(),
                None => return json!({"ok": false, "error": "Missing 'model' parameter"}),
            };
            if !inkwell_core::models::is_supported(&model) {
                return json!({
                    "ok": false,
                    "error": format!("Unsupported model '{}'. See inkwell_core::models::SUPPORTED_MODELS", model),
                    "supported": inkwell_core::models::SUPPORTED_MODELS,
                });
            }
            state.selected_model = model.clone();
            store.update(cx, |s, cx| {
                s.selected_model = model;
                cx.emit(StoreEvent::SettingsChanged);
            });
            json!({"ok": true})
        }

        "devtools/new_project" => {
            let name = params["name"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Nouveau prompt".to_string());
            // CRITICAL: flush any pending edits of the current project to disk
            // BEFORE replacing state.project. Otherwise rapid switches
            // (new_project → edits → new_project) lose the first project's work.
            crate::persistence::flush_project_from_state(state);
            state.save_pending = false;
            let mut p = crate::state::Project::default_prompt();
            p.name = name.clone();
            let new_id = p.id.clone();
            state.project = p.clone();
            state.block_inputs.clear();
            state.variable_inputs.clear();
            state.prompt_dirty = true;
            // Reset transient UI state from the previous project, same as open_project.
            state.playground_response.clear();
            state.playground_loading = false;
            state.sdd_running = false;
            let store_blocks: Vec<Block> = p
                .blocks
                .iter()
                .map(|b| Block {
                    id: b.id.clone(),
                    block_type: b.block_type,
                    content: b.content.clone(),
                    enabled: b.enabled,
                    editing: false,
                })
                .collect();
            // Insert the new project into the projects list immediately so the
            // sidebar Library reflects it without waiting for the next app restart.
            let summary = crate::state::ProjectSummary {
                id: new_id.clone(),
                name: name.clone(),
                workspace_id: None,
            };
            state.projects.retain(|p| p.id != new_id);
            state.projects.insert(0, summary.clone());
            store.update(cx, |s, cx| {
                s.project = crate::state::Project {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    workspace_id: None,
                    blocks: store_blocks,
                    variables: std::collections::HashMap::new(),
                    tags: vec![],
                    framework: None,
                };
                s.projects.retain(|p| p.id != summary.id);
                s.projects.insert(0, summary);
                s.feature_counter += 1;
                s.prompt_dirty = true;
                s.playground_response.clear();
                s.playground_loading = false;
                s.sdd_running = false;
                s.refresh_cache();
                cx.emit(StoreEvent::ProjectChanged);
                cx.emit(StoreEvent::PromptCacheUpdated);
            });
            let _ = crate::persistence::save_current_project_id(&new_id);
            // Ensure the project shows up on disk on the next tick.
            state.save_pending = true;
            state.save_timer = 1;
            json!({"ok": true, "project_id": new_id, "name": name})
        }

        "devtools/rename_project" => {
            let name = match params["name"].as_str() {
                Some(s) if !s.trim().is_empty() => s.to_string(),
                _ => return json!({"ok": false, "error": "Missing or empty 'name' parameter"}),
            };
            let pid = state.project.id.clone();
            state.project.name = name.clone();
            // Mirror the rename into the project list so the sidebar updates.
            for p in state.projects.iter_mut() {
                if p.id == pid {
                    p.name = name.clone();
                }
            }
            store.update(cx, |s, cx| {
                s.project.name = name.clone();
                for p in s.projects.iter_mut() {
                    if p.id == pid {
                        p.name = name.clone();
                    }
                }
                cx.emit(StoreEvent::ProjectChanged);
            });
            state.save_pending = true;
            state.save_timer = 1;
            json!({"ok": true, "name": name})
        }

        "devtools/set_variable" => {
            let key = match params["key"].as_str() {
                Some(s) if !s.is_empty() => s.to_string(),
                _ => return json!({"ok": false, "error": "Missing 'key' parameter"}),
            };
            let value = match params["value"].as_str() {
                Some(s) => s.to_string(),
                None => return json!({"ok": false, "error": "Missing or non-string 'value' parameter"}),
            };
            state.project.variables.insert(key.clone(), value.clone());
            state.prompt_dirty = true;
            store.update(cx, |s, cx| {
                s.project.variables.insert(key.clone(), value.clone());
                s.prompt_dirty = true;
                s.refresh_cache();
                cx.emit(StoreEvent::PromptCacheUpdated);
            });
            json!({"ok": true, "key": key, "value": value})
        }

        "devtools/delete_variable" => {
            let key = match params["key"].as_str() {
                Some(s) if !s.is_empty() => s.to_string(),
                _ => return json!({"ok": false, "error": "Missing 'key' parameter"}),
            };
            let existed = state.project.variables.remove(&key).is_some();
            if existed {
                state.prompt_dirty = true;
                store.update(cx, |s, cx| {
                    s.project.variables.remove(&key);
                    s.prompt_dirty = true;
                    s.refresh_cache();
                    cx.emit(StoreEvent::PromptCacheUpdated);
                });
            }
            json!({"ok": true, "removed": existed})
        }

        "devtools/select_left_tab" => {
            let tab_str = params["tab"].as_str().unwrap_or("");
            let tab = match tab_str {
                "Library" => Some(LeftTab::Library),
                "Frameworks" => Some(LeftTab::Frameworks),
                "Versions" => Some(LeftTab::Versions),
                _ => None,
            };
            if let Some(t) = tab {
                state.left_tab = t;
                store.update(cx, |s, cx| {
                    s.left_tab = t;
                    s.left_open = true;
                    cx.emit(StoreEvent::ProjectChanged);
                });
                json!({"ok": true})
            } else {
                json!({"ok": false, "error": format!("Unknown left tab: {}. Valid: Library, Frameworks, Versions", tab_str)})
            }
        }

        "devtools/open_project" => {
            let project_id = match params["project_id"].as_str() {
                Some(s) => s.to_string(),
                None => return json!({"ok": false, "error": "Missing 'project_id' parameter"}),
            };
            // CRITICAL: same as new_project — flush pending edits before swap.
            crate::persistence::flush_project_from_state(state);
            state.save_pending = false;
            let local = crate::persistence::load_all_projects();
            let Some(lp) = local.iter().find(|p| p.id == project_id) else {
                return json!({"ok": false, "error": format!("Project '{}' not found on disk", project_id)});
            };
            // Apply to state
            state.project.id = lp.id.clone();
            state.project.name = lp.name.clone();
            state.project.framework = lp.framework.clone();
            state.project.tags = lp.tags.clone();
            state.project.variables = lp.variables.clone();
            state.project.blocks = lp.blocks.iter().map(|b| Block {
                id: b.id.clone(),
                block_type: b.block_type,
                content: b.content.clone(),
                enabled: b.enabled,
                editing: false,
            }).collect();
            state.block_inputs.clear();
            state.variable_inputs.clear();
            state.prompt_dirty = true;
            // Reset transient UI state from the previous project
            state.playground_response.clear();
            state.playground_loading = false;
            state.sdd_running = false;
            // Apply to store
            let store_blocks: Vec<Block> = state.project.blocks.iter().map(|b| Block {
                id: b.id.clone(),
                block_type: b.block_type,
                content: b.content.clone(),
                enabled: b.enabled,
                editing: false,
            }).collect();
            let id_copy = state.project.id.clone();
            let name_copy = state.project.name.clone();
            let fw_copy = state.project.framework.clone();
            store.update(cx, |s, cx| {
                s.project.id = id_copy;
                s.project.name = name_copy;
                s.project.framework = fw_copy;
                s.project.blocks = store_blocks;
                s.playground_response.clear();
                s.playground_loading = false;
                s.sdd_running = false;
                s.prompt_dirty = true;
                s.refresh_cache();
                cx.emit(StoreEvent::ProjectChanged);
                cx.emit(StoreEvent::PromptCacheUpdated);
            });
            // Persist selection
            let _ = crate::persistence::save_current_project_id(&project_id);
            json!({"ok": true, "project_id": project_id})
        }

        _ => json!({"error": format!("Unknown write method: {}", method)}),
    }
}
