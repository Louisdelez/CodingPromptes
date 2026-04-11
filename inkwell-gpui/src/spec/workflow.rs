//! Orchestrates the SDD pipeline: Constitution → Specification → Plan → Tasks → Implementation

use super::generator::{SpecPhase, SpecAction, SpecContext, build_system_prompt, build_user_prompt, block_type_to_phase};
use super::validator;
use crate::types::Block;
use inkwell_core::types::BlockType;

/// Result of running a phase
pub struct PhaseResult {
    pub block_index: usize,
    pub content: String,
    pub phase: SpecPhase,
}

/// Ordered SDD phases for sequential execution
const SDD_ORDER: &[BlockType] = &[
    BlockType::SddConstitution,
    BlockType::SddSpecification,
    BlockType::SddPlan,
    BlockType::SddTasks,
    BlockType::SddImplementation,
];

/// Build the list of (block_index, phase) pairs for SDD blocks in order
pub fn find_sdd_blocks(blocks: &[Block]) -> Vec<(usize, SpecPhase)> {
    let mut result = Vec::new();
    for target_type in SDD_ORDER {
        for (idx, block) in blocks.iter().enumerate() {
            if block.block_type == *target_type && block.enabled {
                if let Some(phase) = block_type_to_phase(block.block_type) {
                    result.push((idx, phase));
                }
            }
        }
    }
    result
}

/// Build context from existing blocks
#[allow(dead_code)]
pub fn build_context(blocks: &[Block], project_name: &str) -> SpecContext {
    let pairs: Vec<(BlockType, String)> = blocks.iter()
        .filter(|b| b.enabled && b.block_type.is_sdd())
        .map(|b| (b.block_type, b.content.clone()))
        .collect();
    SpecContext::from_blocks(project_name, &pairs)
}

/// Build the LLM messages for a single phase generation
pub fn build_llm_messages(phase: SpecPhase, action: SpecAction, ctx: &SpecContext) -> (String, String) {
    let system = build_system_prompt(phase, action);
    let user = build_user_prompt(phase, action, ctx);
    (system, user)
}

/// Validate all SDD blocks and return issues grouped by block index
pub fn validate_all(blocks: &[Block]) -> Vec<(usize, Vec<validator::ValidationIssue>)> {
    let mut results = Vec::new();
    for (idx, block) in blocks.iter().enumerate() {
        if !block.enabled { continue; }
        let issues = match block.block_type {
            BlockType::SddConstitution => validator::validate_constitution(&block.content),
            BlockType::SddSpecification => validator::validate_specification(&block.content),
            BlockType::SddPlan => validator::validate_plan(&block.content),
            BlockType::SddTasks => validator::validate_tasks(&block.content),
            _ => continue,
        };
        if !issues.is_empty() {
            results.push((idx, issues));
        }
    }
    results
}
