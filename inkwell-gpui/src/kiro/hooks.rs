//! Hook system — executes automated actions on IDE events.
//! Inspired by Kiro's agent hooks.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum HookEvent {
    BlockChange,
    SpecGenerated,
    ProjectSave,
    ProjectOpen,
    Manual,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HookAction {
    LlmPrompt(String),
    ValidateSpec,
    AutoSave,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hook {
    pub name: String,
    pub description: String,
    pub event: HookEvent,
    pub action: HookAction,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct HookEngine {
    pub hooks: Vec<Hook>,
}

impl HookEngine {
    pub fn new() -> Self {
        Self { hooks: Self::default_hooks() }
    }

    fn default_hooks() -> Vec<Hook> {
        vec![
            Hook {
                name: "Validate on generate".into(),
                description: "Validate spec structure after SDD generation".into(),
                event: HookEvent::SpecGenerated,
                action: HookAction::ValidateSpec,
                enabled: true,
            },
            Hook {
                name: "Auto-save on change".into(),
                description: "Save project when blocks change".into(),
                event: HookEvent::BlockChange,
                action: HookAction::AutoSave,
                enabled: true,
            },
        ]
    }

    pub fn fire(&self, event: &HookEvent) -> Vec<&Hook> {
        self.hooks.iter()
            .filter(|h| h.enabled && h.event == *event)
            .collect()
    }

    #[allow(dead_code)]
    pub fn add(&mut self, hook: Hook) {
        self.hooks.push(hook);
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, index: usize) {
        if index < self.hooks.len() {
            self.hooks.remove(index);
        }
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(h) = self.hooks.get_mut(index) {
            h.enabled = !h.enabled;
        }
    }

    #[allow(dead_code)]
    pub fn save(&self, path: &std::path::Path) {
        if let Ok(json) = serde_json::to_string_pretty(&self.hooks) {
            let _ = std::fs::write(path, json);
        }
    }

    #[allow(dead_code)]
    pub fn load(path: &std::path::Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(json) => {
                let hooks: Vec<Hook> = serde_json::from_str(&json).unwrap_or_else(|_| Self::default_hooks());
                Self { hooks }
            }
            Err(_) => Self::new(),
        }
    }
}
