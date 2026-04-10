//! Structured settings with JSON persistence.
//! Loaded at startup, saved on change. Replaces flat string fields.

use serde::{Deserialize, Serialize};

const SETTINGS_DIR: &str = "inkwell-ide";
const SETTINGS_FILE: &str = "settings.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    pub theme: String,
    pub lang: String,
    pub server_url: String,
    pub api_keys: ApiKeys,
    pub github_repo: String,
    pub selected_model: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ApiKeys {
    pub openai: String,
    pub anthropic: String,
    pub google: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            lang: "fr".into(),
            server_url: "http://localhost:8910".into(),
            api_keys: ApiKeys::default(),
            github_repo: String::new(),
            selected_model: "gpt-4o-mini".into(),
        }
    }
}

impl AppSettings {
    /// Load settings from disk, or return defaults
    pub fn load() -> Self {
        let path = Self::settings_path();
        match std::fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save settings to disk
    pub fn save(&self) {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }

    fn settings_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(SETTINGS_DIR)
            .join(SETTINGS_FILE)
    }
}
