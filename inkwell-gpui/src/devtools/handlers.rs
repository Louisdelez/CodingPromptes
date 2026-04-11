use std::sync::{Arc, RwLock};
use serde_json::json;
use super::DevToolsSnapshot;

pub fn health_check(start_time: std::time::Instant) -> serde_json::Value {
    json!({
        "status": "ok",
        "uptime_secs": start_time.elapsed().as_secs(),
    })
}

pub fn app_state(snapshot: &Arc<RwLock<DevToolsSnapshot>>) -> serde_json::Value {
    if let Ok(s) = snapshot.read() {
        serde_json::to_value(&*s).unwrap_or(json!({"error": "serialize failed"}))
    } else {
        json!({"error": "lock poisoned"})
    }
}

pub fn get_project(snapshot: &Arc<RwLock<DevToolsSnapshot>>) -> serde_json::Value {
    if let Ok(s) = snapshot.read() {
        json!({
            "id": s.project_id,
            "name": s.project_name,
            "blocks": s.blocks,
            "selected_model": s.selected_model,
        })
    } else {
        json!({"error": "lock poisoned"})
    }
}

pub fn get_block(snapshot: &Arc<RwLock<DevToolsSnapshot>>, params: &serde_json::Value) -> serde_json::Value {
    let index = params["index"].as_u64().unwrap_or(0) as usize;
    if let Ok(s) = snapshot.read() {
        match s.blocks.get(index) {
            Some(b) => serde_json::to_value(b).unwrap_or(json!({"error": "serialize failed"})),
            None => json!({"error": format!("Block index {} out of range ({})", index, s.blocks.len())}),
        }
    } else {
        json!({"error": "lock poisoned"})
    }
}

pub fn get_metrics(snapshot: &Arc<RwLock<DevToolsSnapshot>>) -> serde_json::Value {
    if let Ok(s) = snapshot.read() {
        json!({
            "tokens": s.cached_tokens,
            "chars": s.cached_chars,
            "words": s.cached_words,
            "lines": s.cached_lines,
            "blocks_enabled": s.blocks_enabled,
            "blocks_total": s.blocks.len(),
        })
    } else {
        json!({"error": "lock poisoned"})
    }
}

pub fn list_tabs(snapshot: &Arc<RwLock<DevToolsSnapshot>>) -> serde_json::Value {
    if let Ok(s) = snapshot.read() {
        json!({
            "left_tab": s.left_tab,
            "right_tab": s.right_tab,
            "left_open": s.left_open,
            "right_open": s.right_open,
            "terminal_open": s.terminal_open,
        })
    } else {
        json!({"error": "lock poisoned"})
    }
}

pub fn get_logs(params: &serde_json::Value) -> serde_json::Value {
    let lines = params["lines"].as_u64().unwrap_or(50) as usize;
    let logs = super::get_logs(lines);
    json!({ "logs": logs, "count": logs.len() })
}

pub fn validate_state(snapshot: &Arc<RwLock<DevToolsSnapshot>>) -> serde_json::Value {
    let mut issues = Vec::new();

    if let Ok(s) = snapshot.read() {
        if s.project_name.is_empty() {
            issues.push("Project name is empty".to_string());
        }
        if s.blocks.is_empty() {
            issues.push("No blocks in project".to_string());
        }
        let empty_blocks: Vec<usize> = s.blocks.iter()
            .filter(|b| b.enabled && b.content.is_empty())
            .map(|b| b.index)
            .collect();
        if !empty_blocks.is_empty() {
            issues.push(format!("Empty enabled blocks at indices: {:?}", empty_blocks));
        }
        if s.selected_model.is_empty() {
            issues.push("No LLM model selected".to_string());
        }
    }

    json!({ "issues": issues, "valid": issues.is_empty() })
}
