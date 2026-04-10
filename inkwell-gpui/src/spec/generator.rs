//! Builds LLM prompts for SDD block generation, improvement, and clarification.

use super::templates;

#[derive(Clone, Copy)]
pub enum SpecPhase {
    Constitution,
    Specification,
    Plan,
    Tasks,
    Implementation,
}

#[derive(Clone, Copy)]
pub enum SpecAction {
    Generate,
    Improve,
    Validate,
    Clarify,
}

pub struct SpecContext {
    pub project_name: String,
    pub constitution: String,
    pub specification: String,
    pub plan: String,
    pub tasks: String,
    pub implementation: String,
    pub tech_stack: String,
    pub steering_context: String,
    pub preset_engine: Option<super::presets::PresetEngine>,
}

impl SpecContext {
    pub fn empty(project_name: &str) -> Self {
        Self {
            project_name: project_name.to_string(),
            constitution: String::new(),
            specification: String::new(),
            plan: String::new(),
            tasks: String::new(),
            implementation: String::new(),
            tech_stack: String::new(),
            steering_context: String::new(),
            preset_engine: None,
        }
    }

    /// Build context from a list of blocks (type, content)
    pub fn from_blocks(project_name: &str, blocks: &[(inkwell_core::types::BlockType, String)]) -> Self {
        let mut ctx = Self::empty(project_name);
        for (bt, content) in blocks {
            match bt {
                inkwell_core::types::BlockType::SddConstitution => ctx.constitution = content.clone(),
                inkwell_core::types::BlockType::SddSpecification => ctx.specification = content.clone(),
                inkwell_core::types::BlockType::SddPlan => ctx.plan = content.clone(),
                inkwell_core::types::BlockType::SddTasks => ctx.tasks = content.clone(),
                inkwell_core::types::BlockType::SddImplementation => ctx.implementation = content.clone(),
                _ => {}
            }
        }
        ctx
    }
}

/// Build the system prompt for a given phase and action
pub fn build_system_prompt(phase: SpecPhase, action: SpecAction) -> String {
    match action {
        SpecAction::Generate => match phase {
            SpecPhase::Constitution => "You are a software architect expert in Spec-Driven Development. Create a project constitution that defines core principles, constraints, and governance rules. Use the SpecKit format with numbered principles (I, II, III...). Be concise but comprehensive. Write in the user's language.".into(),
            SpecPhase::Specification => "You are a product manager expert in writing specifications. Create a feature specification with prioritized user stories (P1/P2/P3), acceptance scenarios in Given/When/Then format, functional requirements (FR-001...), and success criteria. Do NOT include technical implementation details — focus on WHAT and WHY, not HOW. Write in the user's language.".into(),
            SpecPhase::Plan => "You are a technical architect. Create an implementation plan based on the constitution and specification. Include: Technical Context (language, dependencies, storage), Constitution Check, Project Structure, Design Decisions, and Error Handling strategy. Write in the user's language.".into(),
            SpecPhase::Tasks => "You are a project manager. Break down the implementation plan into discrete, trackable tasks. Use format: `- [ ] T### [P?] [US#] Description`. Group by phases (Setup, Core, Polish). Mark parallelizable tasks with [P]. Reference user stories with [US1], [US2]. Write in the user's language.".into(),
            SpecPhase::Implementation => "You are a senior developer. Based on the tasks list, write the implementation code. Follow the plan's architecture and the constitution's principles. Write clean, well-commented code.".into(),
        },
        SpecAction::Improve => "You are a senior reviewer. Improve the following content: make it clearer, more complete, better structured. Keep the same format and intent but enhance quality. Write in the same language as the input.".into(),
        SpecAction::Validate => "You are a quality auditor. Review the following content and list any issues, missing sections, inconsistencies, or improvements needed. Output a numbered list of findings with severity (ERROR/WARNING/INFO).".into(),
        SpecAction::Clarify => "You are a business analyst. Read the following content and generate 3-5 clarifying questions that would help make the specification more precise. Focus on ambiguities, missing details, and edge cases.".into(),
    }
}

/// Build the user prompt with context
pub fn build_user_prompt(phase: SpecPhase, action: SpecAction, ctx: &SpecContext) -> String {
    match action {
        SpecAction::Generate => {
            // Resolve template via presets (if available) or fallback to core
            let template_type = match phase {
                SpecPhase::Constitution => "constitution",
                SpecPhase::Specification => "specification",
                SpecPhase::Plan => "plan",
                SpecPhase::Tasks => "tasks",
                SpecPhase::Implementation => "",
            };
            let raw_template = if let Some(ref presets) = ctx.preset_engine {
                presets.resolve_template(template_type)
            } else {
                match phase {
                    SpecPhase::Constitution => templates::CONSTITUTION_TEMPLATE.to_string(),
                    SpecPhase::Specification => templates::SPEC_TEMPLATE.to_string(),
                    SpecPhase::Plan => templates::PLAN_TEMPLATE.to_string(),
                    SpecPhase::Tasks => templates::TASKS_TEMPLATE.to_string(),
                    SpecPhase::Implementation => String::new(),
                }
            };
            let template = raw_template.replace("{PROJECT}", &ctx.project_name).replace("{FEATURE}", &ctx.project_name);

            let mut prompt = String::new();

            // Add preceding context
            if !ctx.constitution.is_empty() && !matches!(phase, SpecPhase::Constitution) {
                prompt.push_str(&format!("## Constitution\n{}\n\n", ctx.constitution));
            }
            if !ctx.specification.is_empty() && matches!(phase, SpecPhase::Plan | SpecPhase::Tasks | SpecPhase::Implementation) {
                prompt.push_str(&format!("## Specification\n{}\n\n", ctx.specification));
            }
            if !ctx.plan.is_empty() && matches!(phase, SpecPhase::Tasks | SpecPhase::Implementation) {
                prompt.push_str(&format!("## Plan\n{}\n\n", ctx.plan));
            }
            if !ctx.tasks.is_empty() && matches!(phase, SpecPhase::Implementation) {
                prompt.push_str(&format!("## Tasks\n{}\n\n", ctx.tasks));
            }
            if !ctx.tech_stack.is_empty() {
                prompt.push_str(&format!("## Tech Stack\n{}\n\n", ctx.tech_stack));
            }
            if !ctx.steering_context.is_empty() {
                prompt.push_str(&format!("## Steering Context (Project Rules)\n{}\n\n", ctx.steering_context));
            }

            prompt.push_str(&format!("Generate the content following this template structure:\n\n{}\n\nProject: {}\nDate: {}", template, ctx.project_name, chrono::Local::now().format("%Y-%m-%d")));
            prompt
        }
        SpecAction::Improve => {
            let current = match phase {
                SpecPhase::Constitution => &ctx.constitution,
                SpecPhase::Specification => &ctx.specification,
                SpecPhase::Plan => &ctx.plan,
                SpecPhase::Tasks => &ctx.tasks,
                SpecPhase::Implementation => &ctx.implementation,
            };
            format!("Improve the following content:\n\n{}", current)
        }
        SpecAction::Validate => {
            let current = match phase {
                SpecPhase::Constitution => &ctx.constitution,
                SpecPhase::Specification => &ctx.specification,
                SpecPhase::Plan => &ctx.plan,
                SpecPhase::Tasks => &ctx.tasks,
                SpecPhase::Implementation => &ctx.implementation,
            };
            format!("Validate the following content:\n\n{}", current)
        }
        SpecAction::Clarify => {
            let current = match phase {
                SpecPhase::Constitution => &ctx.constitution,
                SpecPhase::Specification => &ctx.specification,
                SpecPhase::Plan => &ctx.plan,
                SpecPhase::Tasks => &ctx.tasks,
                SpecPhase::Implementation => &ctx.implementation,
            };
            format!("Generate clarifying questions for:\n\n{}", current)
        }
    }
}

/// Convert BlockType to SpecPhase
pub fn block_type_to_phase(bt: inkwell_core::types::BlockType) -> Option<SpecPhase> {
    match bt {
        inkwell_core::types::BlockType::SddConstitution => Some(SpecPhase::Constitution),
        inkwell_core::types::BlockType::SddSpecification => Some(SpecPhase::Specification),
        inkwell_core::types::BlockType::SddPlan => Some(SpecPhase::Plan),
        inkwell_core::types::BlockType::SddTasks => Some(SpecPhase::Tasks),
        inkwell_core::types::BlockType::SddImplementation => Some(SpecPhase::Implementation),
        _ => None,
    }
}
