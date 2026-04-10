//! Validates SDD block content for structure and completeness.

#[derive(Clone, Debug)]
pub enum Severity { Error, Warning, Info }

#[derive(Clone, Debug)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub message: String,
}

pub fn validate_constitution(content: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    if content.trim().is_empty() {
        issues.push(ValidationIssue { severity: Severity::Error, message: "Constitution is empty".into() });
        return issues;
    }
    // Check for principles section
    let principle_count = content.matches("### ").count();
    if principle_count < 3 {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            message: format!("Constitution has {} principles (recommended: at least 3)", principle_count),
        });
    }
    if !content.contains("Governance") && !content.contains("governance") {
        issues.push(ValidationIssue { severity: Severity::Info, message: "No Governance section found".into() });
    }
    if !content.contains("Constraint") && !content.contains("constraint") {
        issues.push(ValidationIssue { severity: Severity::Info, message: "No Constraints section found".into() });
    }
    issues
}

pub fn validate_specification(content: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    if content.trim().is_empty() {
        issues.push(ValidationIssue { severity: Severity::Error, message: "Specification is empty".into() });
        return issues;
    }
    // Check for user stories
    if !content.contains("User Story") && !content.contains("user story") {
        issues.push(ValidationIssue { severity: Severity::Error, message: "No User Stories found".into() });
    }
    // Check for priorities
    if !content.contains("P1") {
        issues.push(ValidationIssue { severity: Severity::Warning, message: "No P1 priority user story found".into() });
    }
    // Check for acceptance scenarios
    if !content.contains("Given") || !content.contains("When") || !content.contains("Then") {
        issues.push(ValidationIssue { severity: Severity::Warning, message: "Missing Given/When/Then acceptance scenarios".into() });
    }
    // Check for requirements
    let fr_count = content.matches("FR-").count();
    if fr_count == 0 {
        issues.push(ValidationIssue { severity: Severity::Warning, message: "No functional requirements (FR-xxx) found".into() });
    }
    // Check for success criteria
    if !content.contains("Success Criteria") && !content.contains("SC-") {
        issues.push(ValidationIssue { severity: Severity::Info, message: "No Success Criteria section found".into() });
    }
    // Check for technical details (should NOT be in spec)
    let tech_words = ["function", "class ", "import ", "pub fn", "async fn", "SELECT ", "CREATE TABLE"];
    for word in tech_words {
        if content.contains(word) {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                message: format!("Specification contains technical detail '{}' — specs should be technology-agnostic", word),
            });
        }
    }
    issues
}

pub fn validate_plan(content: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    if content.trim().is_empty() {
        issues.push(ValidationIssue { severity: Severity::Error, message: "Plan is empty".into() });
        return issues;
    }
    if !content.contains("Technical Context") && !content.contains("technical context") {
        issues.push(ValidationIssue { severity: Severity::Warning, message: "No Technical Context section found".into() });
    }
    if !content.contains("Constitution Check") && !content.contains("constitution check") {
        issues.push(ValidationIssue { severity: Severity::Info, message: "No Constitution Check section found".into() });
    }
    if !content.contains("Project Structure") && !content.contains("project structure") {
        issues.push(ValidationIssue { severity: Severity::Warning, message: "No Project Structure section found".into() });
    }
    issues
}

pub fn validate_tasks(content: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    if content.trim().is_empty() {
        issues.push(ValidationIssue { severity: Severity::Error, message: "Tasks list is empty".into() });
        return issues;
    }
    // Check for task IDs
    let task_count = content.matches("- [ ] T").count() + content.matches("- [x] T").count();
    if task_count == 0 {
        issues.push(ValidationIssue { severity: Severity::Warning, message: "No tasks with T### IDs found".into() });
    }
    // Check for phases
    if !content.contains("Phase") && !content.contains("phase") {
        issues.push(ValidationIssue { severity: Severity::Info, message: "Tasks not organized by phases".into() });
    }
    issues
}
