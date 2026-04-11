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
    let v = &params["index"];
    let index = if let Some(n) = v.as_u64() {
        n as usize
    } else if let Some(n) = v.as_i64() {
        if n < 0 {
            return json!({"error": format!("'index' must be >= 0 (got {})", n)});
        }
        n as usize
    } else {
        return json!({"error": "'index' must be a non-negative integer"});
    };
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

pub fn get_variables(snapshot: &Arc<RwLock<DevToolsSnapshot>>) -> serde_json::Value {
    if let Ok(s) = snapshot.read() {
        json!({ "variables": s.variables })
    } else {
        json!({"error": "lock poisoned"})
    }
}

pub fn get_chat_messages(snapshot: &Arc<RwLock<DevToolsSnapshot>>, params: &serde_json::Value) -> serde_json::Value {
    let limit = params["limit"].as_u64().map(|n| n as usize);
    if let Ok(s) = snapshot.read() {
        let msgs: Vec<_> = match limit {
            Some(n) if n < s.chat_messages.len() => s.chat_messages[s.chat_messages.len() - n..].to_vec(),
            _ => s.chat_messages.clone(),
        };
        json!({ "messages": msgs, "count": s.chat_messages.len() })
    } else {
        json!({"error": "lock poisoned"})
    }
}

pub fn get_executions(snapshot: &Arc<RwLock<DevToolsSnapshot>>, params: &serde_json::Value) -> serde_json::Value {
    let limit = params["limit"].as_u64().map(|n| n as usize).unwrap_or(20);
    if let Ok(s) = snapshot.read() {
        let execs: Vec<_> = s.executions.iter().take(limit).cloned().collect();
        json!({ "executions": execs, "count": s.executions.len() })
    } else {
        json!({"error": "lock poisoned"})
    }
}

pub fn get_playground_response(snapshot: &Arc<RwLock<DevToolsSnapshot>>) -> serde_json::Value {
    if let Ok(s) = snapshot.read() {
        json!({
            "response": s.playground_response,
            "loading": s.playground_loading,
        })
    } else {
        json!({"error": "lock poisoned"})
    }
}

pub fn get_settings(snapshot: &Arc<RwLock<DevToolsSnapshot>>) -> serde_json::Value {
    if let Ok(s) = snapshot.read() {
        json!({
            "dark_mode": s.dark_mode,
            "selected_model": s.selected_model,
            "screen": s.screen,
            "project_name": s.project_name,
        })
    } else {
        json!({"error": "lock poisoned"})
    }
}

pub fn list_frameworks() -> serde_json::Value {
    let fws = crate::persistence::load_frameworks();
    let items: Vec<serde_json::Value> = fws.iter().map(|f| {
        json!({
            "name": f.name,
            "blocks_count": f.blocks.len(),
            "block_types": f.blocks.iter().map(|(bt, _)| format!("{:?}", bt)).collect::<Vec<_>>(),
        })
    }).collect();
    json!({ "frameworks": items, "count": items.len() })
}

pub fn list_projects() -> serde_json::Value {
    let projects = crate::persistence::load_all_projects();
    let items: Vec<serde_json::Value> = projects.iter().map(|p| {
        json!({
            "id": p.id,
            "name": p.name,
            "blocks_count": p.blocks.len(),
            "variables_count": p.variables.len(),
            "tags": p.tags,
            "updated_at": p.updated_at,
        })
    }).collect();
    json!({ "projects": items, "count": items.len() })
}

pub fn get_logs(params: &serde_json::Value) -> serde_json::Value {
    let lines = params["lines"].as_u64().unwrap_or(50) as usize;
    let logs = super::get_logs(lines);
    json!({ "logs": logs, "count": logs.len() })
}

pub fn validate_state(snapshot: &Arc<RwLock<DevToolsSnapshot>>) -> serde_json::Value {
    let mut issues = Vec::new();
    let mut info: Vec<String> = Vec::new();

    if let Ok(s) = snapshot.read() {
        if s.project_name.is_empty() {
            issues.push("Project name is empty".to_string());
        }
        if s.blocks.is_empty() {
            issues.push("No blocks in project".to_string());
        }
        // SDD blocks start empty by design — they're filled by the pipeline.
        // Report them separately in `info`, not as problems.
        let is_sdd_type = |bt: &str| -> bool {
            matches!(
                bt,
                "SddConstitution" | "SddSpecification" | "SddPlan" | "SddTasks" | "SddImplementation"
            )
        };
        let empty_non_sdd: Vec<usize> = s
            .blocks
            .iter()
            .filter(|b| b.enabled && b.content.trim().is_empty() && !is_sdd_type(&b.block_type))
            .map(|b| b.index)
            .collect();
        let empty_sdd: Vec<usize> = s
            .blocks
            .iter()
            .filter(|b| b.enabled && b.content.trim().is_empty() && is_sdd_type(&b.block_type))
            .map(|b| b.index)
            .collect();
        if !empty_non_sdd.is_empty() {
            issues.push(format!("Empty enabled blocks at indices: {:?}", empty_non_sdd));
        }
        if !empty_sdd.is_empty() {
            info.push(format!("SDD blocks awaiting generation: {:?}", empty_sdd));
        }
        if s.selected_model.is_empty() {
            issues.push("No LLM model selected".to_string());
        }
    }

    json!({ "issues": issues, "info": info, "valid": issues.is_empty() })
}
