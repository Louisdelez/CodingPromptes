use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("inkwell-ide")
}

fn projects_dir() -> PathBuf { data_dir().join("projects") }

// ── Session (auth + UI prefs) ──

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedSession {
    pub server_url: String,
    pub token: String,
    pub email: String,
    pub dark_mode: bool,
    pub lang: String,
    pub last_project_id: Option<String>,
    pub left_open: bool,
    pub right_open: bool,
}

impl Default for SavedSession {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            token: String::new(),
            email: String::new(),
            dark_mode: true,
            lang: "fr".into(),
            last_project_id: None,
            left_open: false,
            right_open: false,
        }
    }
}

pub fn load_session() -> SavedSession {
    let path = data_dir().join("session.json");
    if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        SavedSession::default()
    }
}

pub fn save_session(session: &SavedSession) {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(session) {
        let _ = std::fs::write(dir.join("session.json"), json);
    }
}

// ── Local project storage ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalProject {
    pub id: String,
    pub name: String,
    pub workspace_id: Option<String>,
    pub blocks: Vec<inkwell_core::types::PromptBlock>,
    pub variables: std::collections::HashMap<String, String>,
    pub tags: Vec<String>,
    pub framework: Option<String>,
    pub updated_at: i64,
}

pub fn load_all_projects() -> Vec<LocalProject> {
    let dir = projects_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut projects = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(data) = std::fs::read_to_string(entry.path()) {
                    if let Ok(proj) = serde_json::from_str::<LocalProject>(&data) {
                        projects.push(proj);
                    }
                }
            }
        }
    }
    projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    projects
}

pub fn save_project(project: &LocalProject) {
    let dir = projects_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(project) {
        let _ = std::fs::write(dir.join(format!("{}.json", project.id)), json);
    }
}

/// Flush the current project (from AppState) to disk synchronously.
/// Used by MCP handlers that are about to replace state.project
/// (new_project, open_project) — without this, rapid switches lose
/// pending edits that the periodic auto-save hadn't committed yet.
pub fn flush_project_from_state(state: &crate::state::AppState) {
    log::info!("[save] flush_project_from_state id={} name={:?} blocks={}",
        state.project.id, state.project.name, state.project.blocks.len());
    let local_project = LocalProject {
        id: state.project.id.clone(),
        name: state.project.name.clone(),
        workspace_id: state.project.workspace_id.clone(),
        blocks: state.project.blocks.iter().map(|b| {
            inkwell_core::types::PromptBlock {
                id: b.id.clone(),
                block_type: b.block_type,
                content: b.content.clone(),
                enabled: b.enabled,
            }
        }).collect(),
        variables: state.project.variables.clone(),
        tags: state.project.tags.clone(),
        framework: state.project.framework.clone(),
        updated_at: chrono::Utc::now().timestamp_millis(),
    };
    save_project(&local_project);
}

pub fn delete_project(id: &str) {
    let path = projects_dir().join(format!("{id}.json"));
    let _ = std::fs::remove_file(path);
}

/// Persist the current project selection — shared pointer used by CLI and MCP.
pub fn save_current_project_id(id: &str) {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("current-project-id.txt"), id);
}

// ── Settings (API keys) ──

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LocalSettings {
    pub api_key_openai: String,
    pub api_key_anthropic: String,
    pub api_key_google: String,
    pub github_repo: String,
    pub selected_model: String,
}

pub fn load_settings() -> LocalSettings {
    let path = data_dir().join("settings.json");
    if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        LocalSettings::default()
    }
}

pub fn save_settings(settings: &LocalSettings) {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(dir.join("settings.json"), json);
    }
}

// ── Custom frameworks ──

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalFramework {
    pub name: String,
    pub blocks: Vec<(inkwell_core::types::BlockType, String)>,
}

pub fn load_frameworks() -> Vec<LocalFramework> {
    let path = data_dir().join("frameworks.json");
    if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![]
    }
}

pub fn save_frameworks(frameworks: &[LocalFramework]) {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(frameworks) {
        let _ = std::fs::write(dir.join("frameworks.json"), json);
    }
}

// ── Background sync to server ──

pub fn sync_project_to_server(server_url: &str, token: &str, project: &LocalProject) {
    if token.is_empty() || server_url.is_empty() { return; }
    let server = server_url.to_string();
    let tok = token.to_string();
    let proj = project.clone();
    crate::app::rt().spawn(async move {
        let mut client = inkwell_core::api_client::ApiClient::new(&server);
        client.set_token(tok);
        let blocks_json = serde_json::to_string(&proj.blocks).unwrap_or_default();
        let vars_json = serde_json::to_string(&proj.variables).unwrap_or_default();
        // Try update first, then create
        let data = serde_json::json!({
            "name": proj.name,
            "blocks_json": blocks_json,
            "variables_json": vars_json,
            "framework": proj.framework,
            "tags_json": serde_json::to_string(&proj.tags).unwrap_or_default(),
        });
        if client.update_project(&proj.id, &data).await.is_err() {
            let mut create_data = data.clone();
            create_data["id"] = serde_json::json!(proj.id);
            let _ = client.create_project(&create_data).await;
        }
    });
}
