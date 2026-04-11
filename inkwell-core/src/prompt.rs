use crate::types::PromptBlock;
use std::collections::HashMap;
use std::sync::LazyLock;

// Match {{var_name}} where var_name accepts letters, digits, underscore, dot, hyphen.
// Allows namespaced names like {{user.name}}, {{api-key}}, {{v1.2.3}}.
static VAR_REGEX: LazyLock<regex_lite::Regex> =
    LazyLock::new(|| regex_lite::Regex::new(r"\{\{([\w.-]+)\}\}").unwrap());

pub fn compile_prompt(blocks: &[PromptBlock], variables: &HashMap<String, String>) -> String {
    let mut result = String::new();
    for block in blocks {
        if !block.enabled { continue; }
        let mut content = block.content.clone();
        for (key, value) in variables {
            content = content.replace(&format!("{{{{{key}}}}}"), value);
        }
        if !result.is_empty() && !content.is_empty() {
            result.push('\n');
        }
        result.push_str(&content);
    }
    result
}

pub fn extract_variables(blocks: &[PromptBlock]) -> Vec<String> {
    let mut vars = Vec::new();
    for block in blocks {
        if !block.enabled { continue; }
        for cap in VAR_REGEX.captures_iter(&block.content) {
            let var = cap.get(1).unwrap().as_str().to_string();
            if !vars.contains(&var) {
                vars.push(var);
            }
        }
    }
    vars
}
