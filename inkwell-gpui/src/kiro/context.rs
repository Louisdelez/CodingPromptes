//! Context providers for agentic chat — inspired by Kiro's #codebase, #file, #terminal providers.

use std::path::Path;

fn safe_truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Available context providers (triggered by # in chat)
#[derive(Clone, Debug, PartialEq)]
pub enum ContextProvider {
    Codebase,   // #codebase — project structure overview
    File(String), // #file path — specific file content
    Spec,       // #spec — current SDD spec content
    Terminal,   // #terminal — recent terminal output
    Git,        // #git — recent git history
    Steering,   // #steering — active steering rules
}

/// Parse context mentions from a chat message
pub fn parse_mentions(message: &str) -> Vec<ContextProvider> {
    let mut providers = Vec::new();
    for word in message.split_whitespace() {
        match word {
            "#codebase" => providers.push(ContextProvider::Codebase),
            "#spec" => providers.push(ContextProvider::Spec),
            "#terminal" => providers.push(ContextProvider::Terminal),
            "#git" => providers.push(ContextProvider::Git),
            "#steering" => providers.push(ContextProvider::Steering),
            w if w.starts_with("#file:") => {
                let path = w.trim_start_matches("#file:");
                providers.push(ContextProvider::File(path.to_string()));
            }
            _ => {}
        }
    }
    providers
}

/// Resolve a context provider to actual content
pub fn resolve(provider: &ContextProvider, project_dir: Option<&Path>) -> String {
    match provider {
        ContextProvider::Codebase => resolve_codebase(project_dir),
        ContextProvider::File(path) => resolve_file(path),
        ContextProvider::Spec => String::new(), // Resolved externally from blocks
        ContextProvider::Terminal => String::new(), // Resolved externally from terminal sessions
        ContextProvider::Git => resolve_git(project_dir),
        ContextProvider::Steering => String::new(), // Resolved externally from steering engine
    }
}

fn resolve_codebase(project_dir: Option<&Path>) -> String {
    let dir = project_dir.unwrap_or(Path::new("."));
    let mut output = String::from("## Project Structure\n```\n");

    // List files (max 50 for context)
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut count = 0;
        for entry in entries.flatten() {
            if count >= 50 { break; }
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') { continue; } // Skip hidden
            if path.is_dir() {
                output.push_str(&format!("{}/\n", name));
                // List first level of subdirectory
                if let Ok(sub) = std::fs::read_dir(&path) {
                    for sub_entry in sub.flatten().take(10) {
                        let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                        output.push_str(&format!("  {}\n", sub_name));
                        count += 1;
                    }
                }
            } else {
                output.push_str(&format!("{}\n", name));
            }
            count += 1;
        }
    }
    output.push_str("```\n");
    output
}

fn resolve_file(path: &str) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let truncated = if content.len() > 5000 {
                format!("{}...\n[truncated, {} total chars]", safe_truncate(&content, 5000), content.len())
            } else { content };
            format!("## File: {}\n```\n{}\n```\n", path, truncated)
        }
        Err(e) => format!("## File: {} (error: {})\n", path, e),
    }
}

fn resolve_git(project_dir: Option<&Path>) -> String {
    let dir = project_dir.unwrap_or(Path::new("."));
    let commits = crate::spec::git::recent_commits(dir, 10);
    if commits.is_empty() {
        return "## Git: No commits found\n".into();
    }
    let mut output = String::from("## Recent Git History\n");
    for (hash, msg) in &commits {
        output.push_str(&format!("- {} {}\n", hash, msg));
    }
    output
}

/// Build enhanced prompt with resolved context providers
pub fn build_contextual_prompt(message: &str, extra_context: &str) -> String {
    let mentions = parse_mentions(message);
    if mentions.is_empty() && extra_context.is_empty() {
        return message.to_string();
    }

    let mut prompt = String::new();

    // Resolve each provider
    for provider in &mentions {
        let ctx = resolve(provider, None);
        if !ctx.is_empty() {
            prompt.push_str(&ctx);
            prompt.push('\n');
        }
    }

    // Add extra context (steering, spec, terminal)
    if !extra_context.is_empty() {
        prompt.push_str(extra_context);
        prompt.push('\n');
    }

    // Add original message
    prompt.push_str(&format!("## User Request\n{}\n", message));
    prompt
}
