//! Steering system — persistent AI guidance through markdown rules.
//! Inspired by Kiro's steering files.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InclusionMode {
    Always,
    FileMatch,
    Manual,
    Auto,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SteeringRule {
    pub name: String,
    pub description: String,
    pub content: String,
    pub inclusion: InclusionMode,
    pub file_match: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Default)]
pub struct SteeringEngine {
    pub rules: Vec<SteeringRule>,
}

impl SteeringEngine {
    pub fn new() -> Self {
        Self { rules: Self::default_rules() }
    }

    fn default_rules() -> Vec<SteeringRule> {
        vec![
            SteeringRule {
                name: "product".into(),
                description: "Product context — purpose, users, features".into(),
                content: String::new(),
                inclusion: InclusionMode::Always,
                file_match: None,
                enabled: true,
            },
            SteeringRule {
                name: "tech".into(),
                description: "Tech context — frameworks, tools, constraints".into(),
                content: String::new(),
                inclusion: InclusionMode::Always,
                file_match: None,
                enabled: true,
            },
            SteeringRule {
                name: "structure".into(),
                description: "Project structure — file organization, naming".into(),
                content: String::new(),
                inclusion: InclusionMode::Always,
                file_match: None,
                enabled: true,
            },
        ]
    }

    /// Get all active steering context for a given file path
    pub fn get_context(&self, file_path: Option<&str>) -> String {
        let mut ctx = String::new();
        for rule in &self.rules {
            if !rule.enabled || rule.content.is_empty() { continue; }
            let include = match &rule.inclusion {
                InclusionMode::Always => true,
                InclusionMode::FileMatch => {
                    if let (Some(pattern), Some(path)) = (&rule.file_match, file_path) {
                        path.contains(pattern)
                    } else { false }
                }
                InclusionMode::Manual | InclusionMode::Auto => false,
            };
            if include {
                ctx.push_str(&format!("## Steering: {}\n{}\n\n", rule.name, rule.content));
            }
        }
        ctx
    }

    pub fn add(&mut self, rule: SteeringRule) {
        self.rules.push(rule);
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.rules.len() {
            self.rules.remove(index);
        }
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(r) = self.rules.get_mut(index) {
            r.enabled = !r.enabled;
        }
    }

    pub fn save(&self, path: &std::path::Path) {
        if let Ok(json) = serde_json::to_string_pretty(&self.rules) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn load(path: &std::path::Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(json) => {
                let rules: Vec<SteeringRule> = serde_json::from_str(&json).unwrap_or_else(|_| Self::default_rules());
                Self { rules }
            }
            Err(_) => Self::new(),
        }
    }
}
