//! Native Inkwell format — specs live in .inkwell/project.json
//! No .specify/ or .kiro/ — everything is self-contained.

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::types::Block;
use inkwell_core::types::BlockType;

#[derive(Serialize, Deserialize)]
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

/// Save blocks to .inkwell/project.json
pub fn save_native(blocks: &[Block], project_name: &str, steering_rules: &[(String, String)], dir: &Path) -> std::io::Result<()> {
    let inkwell_dir = dir.join(".inkwell");
    std::fs::create_dir_all(&inkwell_dir)?;

    let mut project = InkwellProject {
        name: project_name.to_string(),
        constitution: String::new(),
        specification: String::new(),
        plan: String::new(),
        tasks: String::new(),
        implementation: String::new(),
        steering: std::collections::HashMap::new(),
        created_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
    };

    for block in blocks {
        if !block.enabled { continue; }
        match block.block_type {
            BlockType::SddConstitution => project.constitution = block.content.clone(),
            BlockType::SddSpecification => project.specification = block.content.clone(),
            BlockType::SddPlan => project.plan = block.content.clone(),
            BlockType::SddTasks => project.tasks = block.content.clone(),
            BlockType::SddImplementation => project.implementation = block.content.clone(),
            _ => {}
        }
    }

    for (name, content) in steering_rules {
        project.steering.insert(name.clone(), content.clone());
    }

    let json = serde_json::to_string_pretty(&project).unwrap_or_default();
    std::fs::write(inkwell_dir.join("project.json"), json)
}

/// Load from .inkwell/project.json
#[allow(dead_code)]
pub fn load_native(dir: &Path) -> Option<InkwellProject> {
    let path = dir.join(".inkwell").join("project.json");
    let json = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&json).ok()
}
