use inkwell_core::types::{BlockType, PromptBlock};
use inkwell_core::prompt::{compile_prompt, extract_variables};
use inkwell_core::models;
use std::collections::HashMap;

fn block(bt: BlockType, content: &str, enabled: bool) -> PromptBlock {
    PromptBlock {
        id: "test".into(),
        block_type: bt,
        content: content.into(),
        enabled,
    }
}

// ── compile_prompt ──

#[test]
fn compile_empty() {
    assert_eq!(compile_prompt(&[], &HashMap::new()), "");
}

#[test]
fn compile_single_block() {
    let blocks = vec![block(BlockType::Role, "Hello world", true)];
    assert_eq!(compile_prompt(&blocks, &HashMap::new()), "Hello world");
}

#[test]
fn compile_skips_disabled() {
    let blocks = vec![
        block(BlockType::Role, "Visible", true),
        block(BlockType::Context, "Hidden", false),
        block(BlockType::Task, "Also visible", true),
    ];
    assert_eq!(compile_prompt(&blocks, &HashMap::new()), "Visible\nAlso visible");
}

#[test]
fn compile_skips_empty_blocks() {
    let blocks = vec![
        block(BlockType::Role, "Content", true),
        block(BlockType::Context, "", true),
        block(BlockType::Task, "More", true),
    ];
    assert_eq!(compile_prompt(&blocks, &HashMap::new()), "Content\nMore");
}

#[test]
fn compile_substitutes_variables() {
    let blocks = vec![block(BlockType::Role, "Hello {{name}}, you are {{role}}", true)];
    let mut vars = HashMap::new();
    vars.insert("name".into(), "Alice".into());
    vars.insert("role".into(), "admin".into());
    assert_eq!(compile_prompt(&blocks, &vars), "Hello Alice, you are admin");
}

#[test]
fn compile_undefined_var_stays_literal() {
    let blocks = vec![block(BlockType::Role, "Hello {{unknown}}", true)];
    assert_eq!(compile_prompt(&blocks, &HashMap::new()), "Hello {{unknown}}");
}

#[test]
fn compile_repeated_var() {
    let blocks = vec![block(BlockType::Role, "{{x}} and {{x}} and {{x}}", true)];
    let mut vars = HashMap::new();
    vars.insert("x".into(), "OK".into());
    assert_eq!(compile_prompt(&blocks, &vars), "OK and OK and OK");
}

#[test]
fn compile_var_with_dots_hyphens() {
    let blocks = vec![block(BlockType::Role, "{{user.name}} {{api-key}} {{v1.2}}", true)];
    let mut vars = HashMap::new();
    vars.insert("user.name".into(), "Bob".into());
    vars.insert("api-key".into(), "sk-123".into());
    vars.insert("v1.2".into(), "beta".into());
    assert_eq!(compile_prompt(&blocks, &vars), "Bob sk-123 beta");
}

#[test]
fn compile_multiline() {
    let blocks = vec![block(BlockType::Role, "Line 1\nLine 2\nLine 3", true)];
    assert_eq!(compile_prompt(&blocks, &HashMap::new()), "Line 1\nLine 2\nLine 3");
}

// ── extract_variables ──

#[test]
fn extract_empty() {
    assert_eq!(extract_variables(&[]), Vec::<String>::new());
}

#[test]
fn extract_simple() {
    let blocks = vec![block(BlockType::Role, "Hello {{name}}", true)];
    assert_eq!(extract_variables(&blocks), vec!["name"]);
}

#[test]
fn extract_multiple() {
    let blocks = vec![block(BlockType::Role, "{{a}} and {{b}} and {{c}}", true)];
    assert_eq!(extract_variables(&blocks), vec!["a", "b", "c"]);
}

#[test]
fn extract_deduplicates() {
    let blocks = vec![block(BlockType::Role, "{{x}} {{x}} {{x}}", true)];
    assert_eq!(extract_variables(&blocks), vec!["x"]);
}

#[test]
fn extract_skips_disabled() {
    let blocks = vec![
        block(BlockType::Role, "{{visible}}", true),
        block(BlockType::Context, "{{hidden}}", false),
    ];
    assert_eq!(extract_variables(&blocks), vec!["visible"]);
}

#[test]
fn extract_dots_hyphens() {
    let blocks = vec![block(BlockType::Role, "{{user.name}} {{api-key}}", true)];
    assert_eq!(extract_variables(&blocks), vec!["user.name", "api-key"]);
}

#[test]
fn extract_ignores_spaces_in_braces() {
    let blocks = vec![block(BlockType::Role, "{{ spaced }}", true)];
    assert_eq!(extract_variables(&blocks), Vec::<String>::new());
}

// ── BlockType ──

#[test]
fn block_type_from_name_valid() {
    assert_eq!(BlockType::from_name("role"), Some(BlockType::Role));
    assert_eq!(BlockType::from_name("context"), Some(BlockType::Context));
    assert_eq!(BlockType::from_name("task"), Some(BlockType::Task));
    assert_eq!(BlockType::from_name("examples"), Some(BlockType::Examples));
    assert_eq!(BlockType::from_name("constraints"), Some(BlockType::Constraints));
    assert_eq!(BlockType::from_name("format"), Some(BlockType::Format));
    assert_eq!(BlockType::from_name("sdd-constitution"), Some(BlockType::SddConstitution));
    assert_eq!(BlockType::from_name("sdd_constitution"), Some(BlockType::SddConstitution));
}

#[test]
fn block_type_from_name_invalid() {
    assert_eq!(BlockType::from_name("garbage"), None);
    assert_eq!(BlockType::from_name(""), None);
    assert_eq!(BlockType::from_name("Role"), None); // case sensitive
}

#[test]
fn block_type_is_sdd() {
    assert!(BlockType::SddConstitution.is_sdd());
    assert!(BlockType::SddSpecification.is_sdd());
    assert!(BlockType::SddPlan.is_sdd());
    assert!(BlockType::SddTasks.is_sdd());
    assert!(BlockType::SddImplementation.is_sdd());
    assert!(!BlockType::Role.is_sdd());
    assert!(!BlockType::Task.is_sdd());
}

// ── Models ──

#[test]
fn models_supported() {
    assert!(models::is_supported("gpt-4o"));
    assert!(models::is_supported("gpt-4o-mini"));
    assert!(models::is_supported("claude-opus-4-6"));
    assert!(models::is_supported("gemini-2.5-pro"));
    assert!(models::is_supported("llama3.2"));
    assert!(models::is_supported("mistral"));
}

#[test]
fn models_case_insensitive() {
    assert!(models::is_supported("GPT-4O"));
    assert!(models::is_supported("Claude-Opus-4-6"));
}

#[test]
fn models_ollama_wildcard() {
    assert!(models::is_supported("ollama/mistral"));
    assert!(models::is_supported("ollama/llama3:8b"));
    assert!(models::is_supported("custom:latest"));
}

#[test]
fn models_rejects_invalid() {
    assert!(!models::is_supported("fake-model"));
    assert!(!models::is_supported(""));
    assert!(!models::is_supported("not-a-model"));
}
