//! Layout state persistence — saves/restores panel sizes and visibility.

use serde::{Deserialize, Serialize};

const LAYOUT_DIR: &str = "inkwell-ide";
const LAYOUT_FILE: &str = "layout.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedLayout {
    pub left_open: bool,
    pub left_width: f32,
    pub right_open: bool,
    pub right_width: f32,
    pub terminal_open: bool,
    pub dark_mode: bool,
}

impl Default for SavedLayout {
    fn default() -> Self {
        Self {
            left_open: true,
            left_width: 288.0,
            right_open: true,
            right_width: 384.0,
            terminal_open: false,
            dark_mode: true,
        }
    }
}

impl SavedLayout {
    /// Load layout from disk, or return defaults
    pub fn load() -> Self {
        let path = Self::layout_path();
        match std::fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save layout to disk
    pub fn save(&self) {
        let path = Self::layout_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }

    fn layout_path() -> std::path::PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(LAYOUT_DIR)
            .join(LAYOUT_FILE)
    }
}
