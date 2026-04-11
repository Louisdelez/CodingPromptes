//! Autopilot mode — autonomous task execution from SDD tasks list.
//! Inspired by Kiro's advanced agent mode.

/// Extract tasks from a tasks.md content
pub fn parse_tasks(content: &str) -> Vec<Task> {
    let mut tasks = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        // Match: - [ ] T001 [P] [US1] Description
        // or:    - [x] T002 Description
        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            if let Some(task) = parse_task_line(rest, false) {
                tasks.push(task);
            }
        } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            if let Some(task) = parse_task_line(rest, true) {
                tasks.push(task);
            }
        }
    }
    tasks
}

fn parse_task_line(line: &str, completed: bool) -> Option<Task> {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.is_empty() { return None; }

    let id = parts[0].to_string();
    let rest = parts.get(1).unwrap_or(&"").to_string();

    let parallel = rest.contains("[P]");
    let story = if let Some(start) = rest.find("[US") {
        if let Some(end) = rest[start..].find(']') {
            Some(rest[start+1..start+end].to_string())
        } else { None }
    } else { None };

    let description = rest
        .replace("[P]", "").replace("[P] ", "")
        .trim().to_string();
    // Remove story tag from description
    let description = if let Some(ref s) = story {
        description.replace(&format!("[{}]", s), "").trim().to_string()
    } else { description };

    Some(Task { id, description, completed, parallel, story })
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub completed: bool,
    pub parallel: bool,
    pub story: Option<String>,
}

/// Build an LLM prompt to implement a specific task
pub fn build_task_prompt(task: &Task, plan_context: &str, constitution: &str) -> String {
    let mut prompt = String::new();
    if !constitution.is_empty() {
        prompt.push_str(&format!("## Project Constitution\n{}\n\n", constitution));
    }
    if !plan_context.is_empty() {
        prompt.push_str(&format!("## Implementation Plan\n{}\n\n", plan_context));
    }
    prompt.push_str(&format!(
        "## Task to Implement\n**{}**: {}\n\nImplement this task. Write the code, explain what you did.",
        task.id, task.description
    ));
    prompt
}

/// Get next uncompleted task
pub fn next_task(tasks: &[Task]) -> Option<&Task> {
    tasks.iter().find(|t| !t.completed)
}

/// Get progress stats
pub fn progress(tasks: &[Task]) -> (usize, usize) {
    let completed = tasks.iter().filter(|t| t.completed).count();
    (completed, tasks.len())
}

/// Mark a task as completed in the raw tasks content
#[allow(dead_code)]
pub fn mark_completed(content: &str, task_id: &str) -> String {
    content.replace(
        &format!("- [ ] {}", task_id),
        &format!("- [x] {}", task_id),
    )
}
