use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use inkwell_core::types::{BlockType, PromptBlock};

/// Same format as GPUI's LocalProject — full compatibility
#[derive(Serialize, Deserialize, Clone)]
pub struct LocalProject {
    pub id: String,
    pub name: String,
    pub workspace_id: Option<String>,
    pub blocks: Vec<PromptBlock>,
    pub variables: std::collections::HashMap<String, String>,
    pub tags: Vec<String>,
    pub framework: Option<String>,
    pub updated_at: i64,
}

impl LocalProject {
    pub fn new(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            workspace_id: None,
            blocks: vec![
                PromptBlock { id: uuid::Uuid::new_v4().to_string(), block_type: BlockType::SddConstitution, content: String::new(), enabled: true },
                PromptBlock { id: uuid::Uuid::new_v4().to_string(), block_type: BlockType::SddSpecification, content: String::new(), enabled: true },
                PromptBlock { id: uuid::Uuid::new_v4().to_string(), block_type: BlockType::SddPlan, content: String::new(), enabled: true },
                PromptBlock { id: uuid::Uuid::new_v4().to_string(), block_type: BlockType::SddTasks, content: String::new(), enabled: true },
                PromptBlock { id: uuid::Uuid::new_v4().to_string(), block_type: BlockType::SddImplementation, content: String::new(), enabled: true },
            ],
            variables: std::collections::HashMap::new(),
            tags: vec![chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()],
            framework: Some("sdd".into()),
            updated_at: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn data_dir() -> PathBuf {
        dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("inkwell-ide")
    }

    pub fn projects_dir() -> PathBuf { Self::data_dir().join("projects") }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = Self::projects_dir();
        std::fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(dir.join(format!("{}.json", self.id)), json)?;
        // Also save as current project pointer
        std::fs::write(Self::data_dir().join("current-project-id.txt"), &self.id)?;
        Ok(())
    }

    pub fn load_current() -> Option<Self> {
        let id = std::fs::read_to_string(Self::data_dir().join("current-project-id.txt")).ok()?;
        let id = id.trim();
        if !id.chars().all(|c| c.is_ascii_hexdigit() || c == '-') || id.len() < 32 {
            return None;
        }
        let path = Self::projects_dir().join(format!("{}.json", id));
        let json = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&json).ok()
    }

    pub fn save_to_inkwell(&self, dir: &std::path::Path) -> std::io::Result<()> {
        let inkwell_dir = dir.join(".inkwell");
        std::fs::create_dir_all(&inkwell_dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(inkwell_dir.join("project.json"), json)
    }

    pub fn load_all() -> Vec<Self> {
        let dir = Self::projects_dir();
        let _ = std::fs::create_dir_all(&dir);
        let mut projects = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(data) = std::fs::read_to_string(entry.path()) {
                        if let Ok(proj) = serde_json::from_str::<Self>(&data) {
                            projects.push(proj);
                        }
                    }
                }
            }
        }
        projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        projects
    }

    // SDD phase helpers
    pub fn get_phase(&self, bt: BlockType) -> &str {
        self.blocks.iter().find(|b| b.block_type == bt && b.enabled).map(|b| b.content.as_str()).unwrap_or("")
    }

    pub fn set_phase(&mut self, bt: BlockType, content: String) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.block_type == bt) {
            block.content = content;
        }
        self.updated_at = chrono::Utc::now().timestamp_millis();
    }

    pub fn constitution(&self) -> &str { self.get_phase(BlockType::SddConstitution) }
    pub fn specification(&self) -> &str { self.get_phase(BlockType::SddSpecification) }
    pub fn plan(&self) -> &str { self.get_phase(BlockType::SddPlan) }
    pub fn tasks(&self) -> &str { self.get_phase(BlockType::SddTasks) }
    pub fn implementation(&self) -> &str { self.get_phase(BlockType::SddImplementation) }
}

/// Settings shared with GPUI app
#[derive(Serialize, Deserialize, Default)]
pub struct LocalSettings {
    pub api_key_openai: String,
    pub api_key_anthropic: String,
    pub api_key_google: String,
    pub selected_model: String,
    pub github_repo: String,
}

impl LocalSettings {
    pub fn load() -> Self {
        let path = LocalProject::data_dir().join("settings.json");
        std::fs::read_to_string(path).ok()
            .and_then(|j| serde_json::from_str(&j).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = LocalProject::data_dir();
        std::fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(dir.join("settings.json"), &json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(dir.join("settings.json"), std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}
