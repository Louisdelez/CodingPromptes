use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone)]
pub struct InkwellProject {
    pub name: String,
    pub constitution: String,
    pub specification: String,
    pub plan: String,
    pub tasks: String,
    pub implementation: String,
    pub steering: std::collections::HashMap<String, String>,
    pub created_at: String,
}

impl InkwellProject {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            constitution: String::new(),
            specification: String::new(),
            plan: String::new(),
            tasks: String::new(),
            implementation: String::new(),
            steering: std::collections::HashMap::new(),
            created_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
        }
    }

    pub fn project_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("inkwell-ide")
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = Self::project_dir();
        std::fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(dir.join("current-project.json"), json)
    }

    pub fn load() -> Option<Self> {
        let path = Self::project_dir().join("current-project.json");
        let json = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&json).ok()
    }

    pub fn save_to_inkwell(&self, dir: &Path) -> std::io::Result<()> {
        let inkwell_dir = dir.join(".inkwell");
        std::fs::create_dir_all(&inkwell_dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(inkwell_dir.join("project.json"), json)
    }
}
