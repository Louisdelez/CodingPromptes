//! Export Inkwell projects to SpecKit (.specify/) and Kiro (.kiro/) formats.

use crate::types::Block;
use inkwell_core::types::BlockType;
use std::path::Path;

/// Export blocks to SpecKit .specify/ format
pub fn export_speckit(blocks: &[Block], project_name: &str, output_dir: &Path) -> std::io::Result<()> {
    let feature_name = project_name.to_lowercase().replace(' ', "-");
    let spec_dir = output_dir.join(".specify").join("specs").join(format!("001-{}", feature_name));
    let memory_dir = output_dir.join(".specify").join("memory");
    std::fs::create_dir_all(&spec_dir)?;
    std::fs::create_dir_all(&memory_dir)?;

    for block in blocks {
        if !block.enabled || block.content.is_empty() { continue; }
        match block.block_type {
            BlockType::SddConstitution => {
                std::fs::write(memory_dir.join("constitution.md"), &block.content)?;
            }
            BlockType::SddSpecification => {
                std::fs::write(spec_dir.join("spec.md"), &block.content)?;
            }
            BlockType::SddPlan => {
                std::fs::write(spec_dir.join("plan.md"), &block.content)?;
            }
            BlockType::SddTasks => {
                std::fs::write(spec_dir.join("tasks.md"), &block.content)?;
            }
            BlockType::SddImplementation => {
                std::fs::write(spec_dir.join("implementation.md"), &block.content)?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Export blocks to Kiro .kiro/ format
pub fn export_kiro(blocks: &[Block], project_name: &str, output_dir: &Path) -> std::io::Result<()> {
    let feature_name = project_name.to_lowercase().replace(' ', "-");
    let specs_dir = output_dir.join(".kiro").join("specs").join(&feature_name);
    let steering_dir = output_dir.join(".kiro").join("steering");
    std::fs::create_dir_all(&specs_dir)?;
    std::fs::create_dir_all(&steering_dir)?;

    for block in blocks {
        if !block.enabled || block.content.is_empty() { continue; }
        match block.block_type {
            BlockType::SddConstitution => {
                // Constitution → steering/product.md (project principles)
                std::fs::write(steering_dir.join("product.md"), &block.content)?;
            }
            BlockType::SddSpecification => {
                std::fs::write(specs_dir.join("requirements.md"), &block.content)?;
            }
            BlockType::SddPlan => {
                std::fs::write(specs_dir.join("design.md"), &block.content)?;
            }
            BlockType::SddTasks => {
                std::fs::write(specs_dir.join("tasks.md"), &block.content)?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Import from SpecKit .specify/ format into blocks
pub fn import_speckit(input_dir: &Path) -> Vec<(BlockType, String)> {
    let mut blocks = Vec::new();
    let spec_dirs: Vec<_> = std::fs::read_dir(input_dir.join(".specify").join("specs"))
        .into_iter().flatten().flatten()
        .filter(|e| e.path().is_dir())
        .collect();

    // Load constitution
    let constitution_path = input_dir.join(".specify").join("memory").join("constitution.md");
    if let Ok(content) = std::fs::read_to_string(&constitution_path) {
        blocks.push((BlockType::SddConstitution, content));
    }

    // Load first spec directory
    if let Some(spec_dir) = spec_dirs.first() {
        let dir = spec_dir.path();
        if let Ok(c) = std::fs::read_to_string(dir.join("spec.md")) { blocks.push((BlockType::SddSpecification, c)); }
        if let Ok(c) = std::fs::read_to_string(dir.join("plan.md")) { blocks.push((BlockType::SddPlan, c)); }
        if let Ok(c) = std::fs::read_to_string(dir.join("tasks.md")) { blocks.push((BlockType::SddTasks, c)); }
    }

    blocks
}

/// Import from Kiro .kiro/ format into blocks
pub fn import_kiro(input_dir: &Path) -> Vec<(BlockType, String)> {
    let mut blocks = Vec::new();
    let steering_dir = input_dir.join(".kiro").join("steering");
    let specs_dir = input_dir.join(".kiro").join("specs");

    // Load constitution from steering/product.md
    if let Ok(content) = std::fs::read_to_string(steering_dir.join("product.md")) {
        blocks.push((BlockType::SddConstitution, content));
    }

    // Find first spec directory
    if let Ok(entries) = std::fs::read_dir(&specs_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let dir = entry.path();
                if let Ok(c) = std::fs::read_to_string(dir.join("requirements.md")) { blocks.push((BlockType::SddSpecification, c)); }
                if let Ok(c) = std::fs::read_to_string(dir.join("design.md")) { blocks.push((BlockType::SddPlan, c)); }
                if let Ok(c) = std::fs::read_to_string(dir.join("tasks.md")) { blocks.push((BlockType::SddTasks, c)); }
                break; // Only import first spec
            }
        }
    }

    blocks
}
