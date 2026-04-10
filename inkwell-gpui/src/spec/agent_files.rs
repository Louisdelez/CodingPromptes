//! Generate agent context files (AGENTS.md, CLAUDE.md, etc.)
//! Inspired by SpecKit's agent integration system.

use std::path::Path;

/// Generate AGENTS.md for generic AI agents
pub fn generate_agents_md(project_name: &str, constitution: &str, tech_stack: &str) -> String {
    let mut content = format!("# AGENTS.md — {}\n\n", project_name);
    content.push_str("## Project Context\n\n");

    if !constitution.is_empty() {
        content.push_str("### Constitution\n");
        content.push_str(constitution);
        content.push_str("\n\n");
    }

    if !tech_stack.is_empty() {
        content.push_str("### Tech Stack\n");
        content.push_str(tech_stack);
        content.push_str("\n\n");
    }

    content.push_str("## Development Workflow\n\n");
    content.push_str("This project uses Spec-Driven Development (SDD):\n");
    content.push_str("1. Define Constitution (project principles)\n");
    content.push_str("2. Write Specification (user stories, requirements)\n");
    content.push_str("3. Create Plan (technical architecture)\n");
    content.push_str("4. Break into Tasks (discrete, trackable)\n");
    content.push_str("5. Implement (code generation)\n\n");

    content.push_str("## Spec Files Location\n\n");
    content.push_str("```\n");
    content.push_str(".specify/\n");
    content.push_str("  specs/     — Feature specifications\n");
    content.push_str("  memory/    — Constitution and project memory\n");
    content.push_str("  templates/ — Spec templates\n");
    content.push_str("```\n");

    content
}

/// Generate CLAUDE.md for Anthropic Claude Code
pub fn generate_claude_md(project_name: &str, constitution: &str, steering_rules: &[(&str, &str)]) -> String {
    let mut content = format!("# CLAUDE.md — {}\n\n", project_name);

    if !constitution.is_empty() {
        content.push_str("## Project Constitution\n\n");
        content.push_str(constitution);
        content.push_str("\n\n");
    }

    if !steering_rules.is_empty() {
        content.push_str("## Steering Rules\n\n");
        for (name, desc) in steering_rules {
            content.push_str(&format!("### {}\n{}\n\n", name, desc));
        }
    }

    content.push_str("## Commands\n\n");
    content.push_str("- `/speckit.specify` — Create feature specification\n");
    content.push_str("- `/speckit.plan` — Create implementation plan\n");
    content.push_str("- `/speckit.tasks` — Generate task list\n");
    content.push_str("- `/speckit.clarify` — Clarify requirements\n");
    content.push_str("- `/speckit.implement` — Execute tasks\n");
    content.push_str("- `/speckit.checklist` — Quality validation\n");
    content.push_str("- `/speckit.analyze` — Audit plan\n");

    content
}

/// Write agent context files to project directory
pub fn export_agent_files(
    dir: &Path,
    project_name: &str,
    constitution: &str,
    tech_stack: &str,
    steering_rules: &[(&str, &str)],
) -> std::io::Result<()> {
    // AGENTS.md at project root
    let agents_md = generate_agents_md(project_name, constitution, tech_stack);
    std::fs::write(dir.join("AGENTS.md"), &agents_md)?;

    // CLAUDE.md in .claude/ directory
    let claude_dir = dir.join(".claude");
    std::fs::create_dir_all(&claude_dir)?;
    let claude_md = generate_claude_md(project_name, constitution, steering_rules);
    std::fs::write(claude_dir.join("CLAUDE.md"), &claude_md)?;

    // .kiro/steering/ files
    let steering_dir = dir.join(".kiro").join("steering");
    std::fs::create_dir_all(&steering_dir)?;
    for (name, content) in steering_rules {
        std::fs::write(steering_dir.join(format!("{}.md", name)), content)?;
    }

    Ok(())
}
