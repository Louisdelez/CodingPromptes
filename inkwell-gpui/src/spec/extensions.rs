//! Extensions system — pluggable commands and hooks.
//! Inspired by SpecKit's extension ecosystem.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub ext_type: ExtensionType,
    pub enabled: bool,
    pub commands: Vec<ExtensionCommand>,
    pub hooks: Vec<ExtensionHook>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExtensionType {
    Docs,        // Reads/validates/generates spec artifacts
    Code,        // Reviews/validates/modifies source code
    Process,     // Orchestrates workflow across phases
    Integration, // Syncs with external platforms
    Visibility,  // Reports on project health
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionCommand {
    pub name: String,
    pub description: String,
    pub prompt_template: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionHook {
    pub event: String,       // before_specify, after_plan, etc.
    pub command: String,
    pub optional: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ExtensionRegistry {
    pub extensions: Vec<Extension>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self { extensions: Self::builtin_extensions() }
    }

    fn builtin_extensions() -> Vec<Extension> {
        vec![
            Extension {
                id: "git".into(),
                name: "Git Integration".into(),
                description: "Feature branch creation and auto-commit for specs".into(),
                version: "1.0.0".into(),
                ext_type: ExtensionType::Integration,
                enabled: true,
                commands: vec![
                    ExtensionCommand {
                        name: "speckit.git.feature".into(),
                        description: "Create feature branch".into(),
                        prompt_template: "Create a git feature branch for the current spec".into(),
                    },
                    ExtensionCommand {
                        name: "speckit.git.commit".into(),
                        description: "Commit spec artifacts".into(),
                        prompt_template: "Commit all spec artifacts with a descriptive message".into(),
                    },
                ],
                hooks: vec![
                    ExtensionHook {
                        event: "before_specify".into(),
                        command: "speckit.git.feature".into(),
                        optional: true,
                    },
                    ExtensionHook {
                        event: "after_plan".into(),
                        command: "speckit.git.commit".into(),
                        optional: true,
                    },
                ],
            },
            Extension {
                id: "checklist".into(),
                name: "Quality Checklist".into(),
                description: "Generate quality checklists for specs and plans".into(),
                version: "1.0.0".into(),
                ext_type: ExtensionType::Docs,
                enabled: true,
                commands: vec![
                    ExtensionCommand {
                        name: "speckit.checklist".into(),
                        description: "Generate quality checklist".into(),
                        prompt_template: "Generate a quality checklist based on the current spec and plan".into(),
                    },
                ],
                hooks: vec![],
            },
            Extension {
                id: "analyze".into(),
                name: "Plan Analyzer".into(),
                description: "Audit implementation plans for completeness".into(),
                version: "1.0.0".into(),
                ext_type: ExtensionType::Docs,
                enabled: true,
                commands: vec![
                    ExtensionCommand {
                        name: "speckit.analyze".into(),
                        description: "Analyze plan for issues".into(),
                        prompt_template: "Analyze the implementation plan for completeness, consistency, and constitutional compliance".into(),
                    },
                ],
                hooks: vec![],
            },
        ]
    }

    pub fn get_enabled(&self) -> Vec<&Extension> {
        self.extensions.iter().filter(|e| e.enabled).collect()
    }

    pub fn toggle(&mut self, id: &str) {
        if let Some(ext) = self.extensions.iter_mut().find(|e| e.id == id) {
            ext.enabled = !ext.enabled;
        }
    }

    pub fn get_commands(&self) -> Vec<(&str, &str)> {
        self.extensions.iter()
            .filter(|e| e.enabled)
            .flat_map(|e| e.commands.iter().map(|c| (c.name.as_str(), c.description.as_str())))
            .collect()
    }

    pub fn save(&self, path: &Path) {
        if let Ok(json) = serde_json::to_string_pretty(&self.extensions) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(json) => {
                let extensions = serde_json::from_str(&json).unwrap_or_else(|_| Self::builtin_extensions());
                Self { extensions }
            }
            Err(_) => Self::new(),
        }
    }
}
