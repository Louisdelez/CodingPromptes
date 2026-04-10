//! Slash command router for SpecKit commands.
//! Parses /speckit.* commands from chat and executes them.

use std::sync::mpsc;
use crate::types::*;
use crate::spec::generator::{SpecPhase, SpecAction, SpecContext};
use crate::spec::workflow;
use inkwell_core::types::BlockType;

#[derive(Clone, Debug)]
pub enum SpecCommand {
    Specify(String),
    Plan(String),
    Tasks,
    Clarify(String),
    Constitution(String),
    Implement,
    Checklist,
    Analyze,
    TasksToIssues,
}

/// Parse a slash command from chat input
pub fn parse_command(input: &str) -> Option<SpecCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with("/speckit.") { return None; }

    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    let cmd = parts[0];
    let args = parts.get(1).unwrap_or(&"").trim().to_string();

    match cmd {
        "/speckit.specify" => Some(SpecCommand::Specify(args)),
        "/speckit.plan" => Some(SpecCommand::Plan(args)),
        "/speckit.tasks" => Some(SpecCommand::Tasks),
        "/speckit.clarify" => Some(SpecCommand::Clarify(args)),
        "/speckit.constitution" => Some(SpecCommand::Constitution(args)),
        "/speckit.implement" => Some(SpecCommand::Implement),
        "/speckit.checklist" => Some(SpecCommand::Checklist),
        "/speckit.analyze" => Some(SpecCommand::Analyze),
        "/speckit.taskstoissues" => Some(SpecCommand::TasksToIssues),
        _ => None,
    }
}

/// Execute a SpecKit command — returns (system_prompt, user_prompt, target_block_type)
pub fn build_command_prompt(
    cmd: &SpecCommand,
    blocks: &[Block],
    project_name: &str,
) -> Option<(String, String, Option<BlockType>)> {
    let pairs: Vec<(BlockType, String)> = blocks.iter()
        .filter(|b| b.enabled && b.block_type.is_sdd())
        .map(|b| (b.block_type, b.content.clone()))
        .collect();
    let ctx = SpecContext::from_blocks(project_name, &pairs);

    match cmd {
        SpecCommand::Specify(desc) => {
            let mut c = ctx.clone();
            if !desc.is_empty() { c.steering_context = format!("Feature description: {}", desc); }
            let (sys, usr) = workflow::build_llm_messages(SpecPhase::Specification, SpecAction::Generate, &c);
            Some((sys, usr, Some(BlockType::SddSpecification)))
        }
        SpecCommand::Plan(tech) => {
            let mut c = ctx.clone();
            if !tech.is_empty() { c.tech_stack = tech.clone(); }
            let (sys, usr) = workflow::build_llm_messages(SpecPhase::Plan, SpecAction::Generate, &c);
            Some((sys, usr, Some(BlockType::SddPlan)))
        }
        SpecCommand::Tasks => {
            let (sys, usr) = workflow::build_llm_messages(SpecPhase::Tasks, SpecAction::Generate, &ctx);
            Some((sys, usr, Some(BlockType::SddTasks)))
        }
        SpecCommand::Clarify(detail) => {
            let mut c = ctx.clone();
            if !detail.is_empty() { c.steering_context = format!("Additional detail: {}", detail); }
            let (sys, usr) = workflow::build_llm_messages(SpecPhase::Specification, SpecAction::Clarify, &c);
            Some((sys, usr, None)) // No block creation, shows in chat
        }
        SpecCommand::Constitution(desc) => {
            let mut c = ctx.clone();
            if !desc.is_empty() { c.steering_context = format!("Project description: {}", desc); }
            let (sys, usr) = workflow::build_llm_messages(SpecPhase::Constitution, SpecAction::Generate, &c);
            Some((sys, usr, Some(BlockType::SddConstitution)))
        }
        SpecCommand::Implement => {
            let (sys, usr) = workflow::build_llm_messages(SpecPhase::Implementation, SpecAction::Generate, &ctx);
            Some((sys, usr, Some(BlockType::SddImplementation)))
        }
        SpecCommand::Checklist => {
            let sys = "You are a quality auditor. Generate a comprehensive quality checklist based on the specification and plan. Use format: - [ ] CHK001 Description".to_string();
            let mut usr = String::new();
            if !ctx.specification.is_empty() { usr.push_str(&format!("## Specification\n{}\n\n", ctx.specification)); }
            if !ctx.plan.is_empty() { usr.push_str(&format!("## Plan\n{}\n\n", ctx.plan)); }
            usr.push_str("Generate a quality checklist for this feature.");
            Some((sys, usr, None))
        }
        SpecCommand::Analyze => {
            let sys = "You are an architecture reviewer. Analyze the implementation plan for completeness, consistency, and constitutional compliance. List issues with severity.".to_string();
            let mut usr = String::new();
            if !ctx.constitution.is_empty() { usr.push_str(&format!("## Constitution\n{}\n\n", ctx.constitution)); }
            if !ctx.specification.is_empty() { usr.push_str(&format!("## Specification\n{}\n\n", ctx.specification)); }
            if !ctx.plan.is_empty() { usr.push_str(&format!("## Plan\n{}\n\n", ctx.plan)); }
            usr.push_str("Analyze this plan for issues.");
            Some((sys, usr, None))
        }
        SpecCommand::TasksToIssues => {
            let sys = "Convert the following task list into GitHub issue format. For each task, output:\n\n## Issue: [Task Description]\n**Labels**: [phase], [story]\n**Body**: [Details]\n\nGroup by user story.".to_string();
            let usr = if !ctx.tasks.is_empty() {
                format!("## Tasks\n{}\n\nConvert to GitHub issues.", ctx.tasks)
            } else { "No tasks found. Run /speckit.tasks first.".to_string() };
            Some((sys, usr, None))
        }
    }
}

/// List available commands with descriptions
pub fn help() -> Vec<(&'static str, &'static str)> {
    vec![
        ("/speckit.specify", "Creer une specification depuis une description"),
        ("/speckit.plan", "Creer un plan d'implementation"),
        ("/speckit.tasks", "Generer la liste de taches"),
        ("/speckit.clarify", "Clarifier les requirements"),
        ("/speckit.constitution", "Definir les principes du projet"),
        ("/speckit.implement", "Executer les taches (autopilot)"),
        ("/speckit.checklist", "Generer un checklist qualite"),
        ("/speckit.analyze", "Auditer le plan d'implementation"),
        ("/speckit.taskstoissues", "Convertir les taches en issues GitHub"),
    ]
}
