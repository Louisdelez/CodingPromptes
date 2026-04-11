//! MCP tools exposed to AI agents

use serde_json::{json, Value};
use std::path::PathBuf;

fn project_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("inkwell-ide")
}

fn load_project() -> Option<Value> {
    let path = project_dir().join("current-project.json");
    let json = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&json).ok()
}

fn load_all_projects() -> Vec<Value> {
    let dir = project_dir().join("projects");
    let mut projects = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(json) = std::fs::read_to_string(entry.path()) {
                if let Ok(p) = serde_json::from_str::<Value>(&json) {
                    projects.push(p);
                }
            }
        }
    }
    // Also load current project, but avoid duplicates
    if let Some(current) = load_project() {
        let current_id = current.get("id").cloned();
        if current_id.is_none() || !projects.iter().any(|p| p.get("id") == current_id.as_ref()) {
            projects.push(current);
        }
    }
    projects
}

pub fn list_tools() -> Vec<Value> {
    vec![
        json!({
            "name": "inkwell_status",
            "description": "Get the current Inkwell project status — shows which SDD phases are complete, steering rules, and project metadata.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "inkwell_read_phase",
            "description": "Read a specific SDD phase content (constitution, specification, plan, tasks, implementation).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "phase": { "type": "string", "enum": ["constitution", "specification", "plan", "tasks", "implementation"], "description": "The SDD phase to read" }
                },
                "required": ["phase"]
            }
        }),
        json!({
            "name": "inkwell_read_project",
            "description": "Read the full Inkwell project — all SDD phases concatenated as markdown.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "inkwell_list_projects",
            "description": "List all Inkwell projects.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "inkwell_validate",
            "description": "Validate all SDD phases — check structure, completeness, and constitutional compliance.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "inkwell_read_steering",
            "description": "Read active steering rules (product, tech, structure contexts).",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "inkwell_read_tasks",
            "description": "Read and parse tasks from the tasks phase — returns structured task list with completion status.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "inkwell_search",
            "description": "Search across all SDD phases for a keyword.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "inkwell_write_phase",
            "description": "Write content to a specific SDD phase.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "phase": { "type": "string", "enum": ["constitution", "specification", "plan", "tasks", "implementation"] },
                    "content": { "type": "string", "description": "Content to write" }
                },
                "required": ["phase", "content"]
            }
        }),
        json!({
            "name": "inkwell_write_steering",
            "description": "Write a steering rule (product, tech, or structure).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Rule name (product, tech, structure)" },
                    "content": { "type": "string", "description": "Rule content" }
                },
                "required": ["name", "content"]
            }
        }),
    ]
}

pub fn call_tool(name: &str, args: &Value) -> String {
    match name {
        "inkwell_status" => tool_status(),
        "inkwell_read_phase" => tool_read_phase(args["phase"].as_str().unwrap_or("")),
        "inkwell_read_project" => tool_read_project(),
        "inkwell_list_projects" => tool_list_projects(),
        "inkwell_validate" => tool_validate(),
        "inkwell_read_steering" => tool_read_steering(),
        "inkwell_read_tasks" => tool_read_tasks(),
        "inkwell_search" => tool_search(args["query"].as_str().unwrap_or("")),
        "inkwell_write_phase" => tool_write_phase(args["phase"].as_str().unwrap_or(""), args["content"].as_str().unwrap_or("")),
        "inkwell_write_steering" => tool_write_steering(args["name"].as_str().unwrap_or(""), args["content"].as_str().unwrap_or("")),
        _ => format!("Unknown tool: {}", name),
    }
}

fn tool_status() -> String {
    match load_project() {
        Some(p) => {
            let phases = ["constitution", "specification", "plan", "tasks", "implementation"];
            let mut status = format!("# Project: {}\n\n", p["name"].as_str().unwrap_or("?"));
            status.push_str("## SDD Phases\n");
            let mut done = 0;
            for phase in &phases {
                let content = p[phase].as_str().unwrap_or("");
                let icon = if content.is_empty() { "○" } else { done += 1; "✓" };
                status.push_str(&format!("- {} {} ({} chars)\n", icon, phase, content.len()));
            }
            status.push_str(&format!("\nProgress: {}/5 phases complete\n", done));
            status
        }
        None => "No active Inkwell project. Run `inkwell init` first.".into(),
    }
}

fn tool_read_phase(phase: &str) -> String {
    const VALID_PHASES: &[&str] = &["constitution", "specification", "plan", "tasks", "implementation"];
    if !VALID_PHASES.contains(&phase) {
        return format!("Invalid phase '{}'. Valid: constitution, specification, plan, tasks, implementation", phase);
    }
    match load_project() {
        Some(p) => p[phase].as_str().unwrap_or("Phase is empty.").to_string(),
        None => "No active project.".into(),
    }
}

fn tool_read_project() -> String {
    match load_project() {
        Some(p) => {
            let mut out = format!("# {}\n\n", p["name"].as_str().unwrap_or("Project"));
            for phase in &["constitution", "specification", "plan", "tasks", "implementation"] {
                let content = p[phase].as_str().unwrap_or("");
                if !content.is_empty() {
                    out.push_str(&format!("## {}\n\n{}\n\n---\n\n", phase.to_uppercase(), content));
                }
            }
            out
        }
        None => "No active project.".into(),
    }
}

fn tool_list_projects() -> String {
    let projects = load_all_projects();
    if projects.is_empty() { return "No projects found.".into(); }
    let mut out = "# Inkwell Projects\n\n".to_string();
    for p in &projects {
        out.push_str(&format!("- {} (created: {})\n", p["name"].as_str().unwrap_or("?"), p["created_at"].as_str().unwrap_or("?")));
    }
    out
}

fn tool_validate() -> String {
    match load_project() {
        Some(p) => {
            let mut out = "# Validation Results\n\n".to_string();
            let checks = [
                ("constitution", vec!["###", "Governance"]),
                ("specification", vec!["User Story", "Given", "FR-"]),
                ("plan", vec!["Technical Context", "Project Structure"]),
                ("tasks", vec!["- [ ] T", "Phase"]),
            ];
            for (phase, markers) in &checks {
                let content = p[phase].as_str().unwrap_or("");
                if content.is_empty() {
                    out.push_str(&format!("- ✗ {} — EMPTY\n", phase));
                } else {
                    let missing: Vec<&&str> = markers.iter().filter(|m| !content.contains(**m)).collect();
                    if missing.is_empty() {
                        out.push_str(&format!("- ✓ {} — OK\n", phase));
                    } else {
                        out.push_str(&format!("- ⚠ {} — missing: {}\n", phase, missing.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", ")));
                    }
                }
            }
            out
        }
        None => "No active project.".into(),
    }
}

fn tool_read_steering() -> String {
    let path = project_dir().join("steering.json");
    match std::fs::read_to_string(&path) {
        Ok(json) => {
            if let Ok(rules) = serde_json::from_str::<Vec<Value>>(&json) {
                let mut out = "# Steering Rules\n\n".to_string();
                for rule in &rules {
                    let name = rule["name"].as_str().unwrap_or("?");
                    let content = rule["content"].as_str().unwrap_or("");
                    let enabled = rule["enabled"].as_bool().unwrap_or(true);
                    out.push_str(&format!("## {} {}\n{}\n\n", name, if enabled { "✓" } else { "✗" }, if content.is_empty() { "(empty)" } else { content }));
                }
                out
            } else { "Invalid steering.json".into() }
        }
        Err(_) => "No steering rules configured.".into(),
    }
}

fn tool_read_tasks() -> String {
    match load_project() {
        Some(p) => {
            let tasks = p["tasks"].as_str().unwrap_or("");
            if tasks.is_empty() { return "No tasks. Run `/inkwell.tasks` first.".into(); }
            let mut out = "# Tasks\n\n".to_string();
            let total = tasks.matches("- [ ] ").count() + tasks.matches("- [x] ").count();
            let done = tasks.matches("- [x] ").count();
            out.push_str(&format!("Progress: {}/{} completed\n\n", done, total));
            out.push_str(tasks);
            out
        }
        None => "No active project.".into(),
    }
}

fn tool_search(query: &str) -> String {
    match load_project() {
        Some(p) => {
            let q = query.to_lowercase();
            let mut results = Vec::new();
            for phase in &["constitution", "specification", "plan", "tasks", "implementation"] {
                let content = p[phase].as_str().unwrap_or("");
                if content.to_lowercase().contains(&q) {
                    let lines: Vec<&str> = content.lines().filter(|l| l.to_lowercase().contains(&q)).take(5).collect();
                    results.push(format!("## {}\n{}", phase, lines.join("\n")));
                }
            }
            if results.is_empty() { format!("No results for '{}'", query) }
            else { format!("# Search: '{}'\n\n{}", query, results.join("\n\n")) }
        }
        None => "No active project.".into(),
    }
}

fn tool_write_phase(phase: &str, content: &str) -> String {
    const VALID_PHASES: &[&str] = &["constitution", "specification", "plan", "tasks", "implementation"];
    if !VALID_PHASES.contains(&phase) {
        return format!("Invalid phase '{}'. Valid: constitution, specification, plan, tasks, implementation", phase);
    }
    match load_project() {
        Some(mut p) => {
            p[phase] = json!(content);
            let path = project_dir().join("current-project.json");
            if let Ok(json) = serde_json::to_string_pretty(&p) {
                let tmp_path = path.with_extension("json.tmp");
                if let Err(e) = std::fs::write(&tmp_path, &json) {
                    return format!("Write error: {}", e);
                }
                if let Err(e) = std::fs::rename(&tmp_path, &path) {
                    return format!("Rename error: {}", e);
                }
                format!("Written {} chars to phase '{}'", content.len(), phase)
            } else { "Failed to serialize project".into() }
        }
        None => "No active project. Run `inkwell init` first.".into(),
    }
}

fn tool_write_steering(name: &str, content: &str) -> String {
    let path = project_dir().join("steering.json");
    let mut rules: Vec<Value> = std::fs::read_to_string(&path).ok()
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default();

    if let Some(rule) = rules.iter_mut().find(|r| r["name"].as_str() == Some(name)) {
        rule["content"] = json!(content);
    } else {
        rules.push(json!({"name": name, "content": content, "enabled": true, "description": name, "inclusion": "Always"}));
    }

    if let Ok(json) = serde_json::to_string_pretty(&rules) {
        let tmp_path = path.with_extension("json.tmp");
        if let Err(e) = std::fs::write(&tmp_path, &json) {
            return format!("Write error: {}", e);
        }
        if let Err(e) = std::fs::rename(&tmp_path, &path) {
            return format!("Rename error: {}", e);
        }
        format!("Steering rule '{}' updated ({} chars)", name, content.len())
    } else { "Failed to save steering".into() }
}
