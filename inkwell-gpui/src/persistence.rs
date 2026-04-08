use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

fn session_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("inkwell-ide").join("session.json")
}

impl Default for SavedSession {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            token: String::new(),
            email: String::new(),
            dark_mode: true, // Dark mode by default like web/Tauri
            lang: "fr".into(),
            last_project_id: None,
            left_open: false,
            right_open: false,
        }
    }
}

pub fn load_session() -> SavedSession {
    let path = session_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        SavedSession::default()
    }
}

pub fn save_session(session: &SavedSession) {
    let path = session_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(session) {
        let _ = std::fs::write(path, json);
    }
}
