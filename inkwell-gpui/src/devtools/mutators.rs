use serde_json::json;
use crate::state::*;
use crate::store::StoreEvent;

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
            let idx = params["index"].as_u64().unwrap_or(0) as usize;
            let content = params["content"].as_str().unwrap_or("").to_string();
            if idx < state.project.blocks.len() {
                state.project.blocks[idx].content = content;
                state.prompt_dirty = true;
                // Reset input entity to force re-creation
                if let Some(inp) = state.block_inputs.get_mut(idx) { *inp = None; }
                store.update(cx, |_, cx| cx.emit(StoreEvent::BlockContentChanged(idx)));
                json!({"ok": true})
            } else {
                json!({"ok": false, "error": "Block index out of range"})
            }
        }

        "devtools/add_block" => {
            let bt_str = params["block_type"].as_str().unwrap_or("task");
            let content = params["content"].as_str().unwrap_or("").to_string();
            let block_type = match bt_str {
                "role" => inkwell_core::types::BlockType::Role,
                "context" => inkwell_core::types::BlockType::Context,
                "task" => inkwell_core::types::BlockType::Task,
                "examples" => inkwell_core::types::BlockType::Examples,
                "constraints" => inkwell_core::types::BlockType::Constraints,
                "format" => inkwell_core::types::BlockType::Format,
                _ => inkwell_core::types::BlockType::Task,
            };
            let mut block = Block::new(block_type);
            block.content = content;
            let idx = state.project.blocks.len();
            state.project.blocks.push(block);
            state.block_inputs.push(None);
            state.prompt_dirty = true;
            store.update(cx, |_, cx| cx.emit(StoreEvent::ProjectChanged));
            json!({"ok": true, "index": idx})
        }

        "devtools/delete_block" => {
            let idx = params["index"].as_u64().unwrap_or(0) as usize;
            if idx < state.project.blocks.len() {
                state.project.blocks.remove(idx);
                if idx < state.block_inputs.len() { state.block_inputs.remove(idx); }
                state.prompt_dirty = true;
                store.update(cx, |_, cx| cx.emit(StoreEvent::ProjectChanged));
                json!({"ok": true})
            } else {
                json!({"ok": false, "error": "Block index out of range"})
            }
        }

        "devtools/toggle_block" => {
            let idx = params["index"].as_u64().unwrap_or(0) as usize;
            if idx < state.project.blocks.len() {
                state.project.blocks[idx].enabled = !state.project.blocks[idx].enabled;
                let enabled = state.project.blocks[idx].enabled;
                state.prompt_dirty = true;
                store.update(cx, |_, cx| cx.emit(StoreEvent::ProjectChanged));
                json!({"ok": true, "enabled": enabled})
            } else {
                json!({"ok": false, "error": "Block index out of range"})
            }
        }

        "devtools/reorder_blocks" => {
            let from = params["from"].as_u64().unwrap_or(0) as usize;
            let to = params["to"].as_u64().unwrap_or(0) as usize;
            let len = state.project.blocks.len();
            if from < len && to < len {
                let block = state.project.blocks.remove(from);
                let insert_at = if from < to { to.saturating_sub(1) } else { to };
                state.project.blocks.insert(insert_at, block);
                state.prompt_dirty = true;
                store.update(cx, |_, cx| cx.emit(StoreEvent::ProjectChanged));
                json!({"ok": true})
            } else {
                json!({"ok": false, "error": "Index out of range"})
            }
        }

        "devtools/select_tab" => {
            let tab_str = params["tab"].as_str().unwrap_or("");
            let tab = match tab_str {
                "Preview" => Some(RightTab::Preview),
                "Playground" => Some(RightTab::Playground),
                "Stt" => Some(RightTab::Stt),
                "History" => Some(RightTab::History),
                "Export" => Some(RightTab::Export),
                "Terminal" => Some(RightTab::Terminal),
                "Optimize" => Some(RightTab::Optimize),
                "Lint" => Some(RightTab::Lint),
                "Chat" => Some(RightTab::Chat),
                "Analytics" => Some(RightTab::Analytics),
                "Sdd" => Some(RightTab::Sdd),
                "Chain" => Some(RightTab::Chain),
                _ => None,
            };
            if let Some(t) = tab {
                store.update(cx, |s, cx| {
                    s.right_tab = t;
                    cx.emit(StoreEvent::SwitchRightTab(t));
                });
                json!({"ok": true})
            } else {
                json!({"ok": false, "error": format!("Unknown tab: {}", tab_str)})
            }
        }

        "devtools/toggle_panel" => {
            let panel = params["panel"].as_str().unwrap_or("left");
            match panel {
                "left" => {
                    store.update(cx, |s, _| s.left_open = !s.left_open);
                    let open = store.read(cx).left_open;
                    json!({"ok": true, "open": open})
                }
                "right" => {
                    store.update(cx, |s, _| s.right_open = !s.right_open);
                    let open = store.read(cx).right_open;
                    json!({"ok": true, "open": open})
                }
                _ => json!({"ok": false, "error": "Unknown panel (use left or right)"}),
            }
        }

        "devtools/set_model" => {
            let model = params["model"].as_str().unwrap_or("").to_string();
            state.selected_model = model.clone();
            store.update(cx, |s, cx| {
                s.selected_model = model;
                cx.emit(StoreEvent::SettingsChanged);
            });
            json!({"ok": true})
        }

        "devtools/open_project" => {
            let project_id = params["project_id"].as_str().unwrap_or("").to_string();
            // Send via the async message channel to trigger project loading
            let _ = state.msg_tx.send(AsyncMsg::ExportReady(format!("__LOADPROJECT__{}", project_id)));
            json!({"ok": true})
        }

        _ => json!({"error": format!("Unknown write method: {}", method)}),
    }
}
