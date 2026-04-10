//! Presets system — customizable templates without modifying core.
//! Inspired by SpecKit's preset resolution stack.
//!
//! Resolution order (highest priority first):
//! 1. Project-local overrides (.specify/templates/overrides/)
//! 2. Installed presets (by priority number)
//! 3. Core templates (fallback)

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: u32, // Lower = higher priority
    pub templates: PresetTemplates,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PresetTemplates {
    pub constitution: Option<String>,
    pub specification: Option<String>,
    pub plan: Option<String>,
    pub tasks: Option<String>,
    pub checklist: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct PresetEngine {
    pub presets: Vec<Preset>,
}

impl PresetEngine {
    pub fn new() -> Self {
        Self { presets: vec![Self::default_preset()] }
    }

    fn default_preset() -> Preset {
        Preset {
            id: "default".into(),
            name: "Standard SDD".into(),
            description: "Standard Spec-Driven Development workflow".into(),
            priority: 100,
            templates: PresetTemplates::default(),
        }
    }

    /// Resolve a template by checking presets in priority order, falling back to core
    pub fn resolve_template(&self, template_type: &str) -> String {
        // Check presets in priority order (lower number = higher priority)
        let mut sorted: Vec<&Preset> = self.presets.iter().collect();
        sorted.sort_by_key(|p| p.priority);

        for preset in sorted {
            let override_template = match template_type {
                "constitution" => &preset.templates.constitution,
                "specification" => &preset.templates.specification,
                "plan" => &preset.templates.plan,
                "tasks" => &preset.templates.tasks,
                "checklist" => &preset.templates.checklist,
                _ => &None,
            };
            if let Some(t) = override_template {
                return t.clone();
            }
        }

        // Fallback to core templates
        match template_type {
            "constitution" => super::templates::CONSTITUTION_TEMPLATE.to_string(),
            "specification" => super::templates::SPEC_TEMPLATE.to_string(),
            "plan" => super::templates::PLAN_TEMPLATE.to_string(),
            "tasks" => super::templates::TASKS_TEMPLATE.to_string(),
            "checklist" => super::templates::CHECKLIST_TEMPLATE.to_string(),
            _ => String::new(),
        }
    }

    pub fn add(&mut self, preset: Preset) {
        self.presets.push(preset);
        self.presets.sort_by_key(|p| p.priority);
    }

    pub fn remove(&mut self, id: &str) {
        self.presets.retain(|p| p.id != id);
    }

    pub fn save(&self, path: &Path) {
        if let Ok(json) = serde_json::to_string_pretty(&self.presets) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(json) => {
                let presets = serde_json::from_str(&json).unwrap_or_else(|_| vec![Self::default_preset()]);
                Self { presets }
            }
            Err(_) => Self::new(),
        }
    }
}
