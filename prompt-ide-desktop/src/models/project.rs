use serde::{Deserialize, Serialize};
use super::block::PromptBlock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub color: String,
    pub user_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptProject {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub workspace_id: Option<String>,
    pub blocks: Vec<PromptBlock>,
    pub variables: std::collections::HashMap<String, String>,
    pub framework: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl PromptProject {
    pub fn new(user_id: &str, workspace_id: Option<String>) -> Self {
        use super::block::{BlockType, PromptBlock};
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Nouveau prompt".into(),
            user_id: user_id.to_string(),
            workspace_id,
            blocks: vec![
                PromptBlock::new(BlockType::Role),
                PromptBlock::new(BlockType::Context),
                PromptBlock::new(BlockType::Task),
            ],
            variables: std::collections::HashMap::new(),
            framework: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn compile(&self) -> String {
        let mut text = self.blocks
            .iter()
            .filter(|b| b.enabled)
            .map(|b| b.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        for (key, value) in &self.variables {
            text = text.replace(&format!("{{{{{key}}}}}"), value);
        }
        text.trim().to_string()
    }

    pub fn extract_variables(&self) -> Vec<String> {
        let all = self.blocks.iter().map(|b| b.content.as_str()).collect::<Vec<_>>().join("\n");
        let re = regex_lite::Regex::new(r"\{\{(\w+)\}\}").unwrap_or_else(|_| unreachable!());
        let mut vars: Vec<String> = Vec::new();
        for cap in re.captures_iter(&all) {
            let name = cap[1].to_string();
            if !vars.contains(&name) {
                vars.push(name);
            }
        }
        vars
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVersion {
    pub id: String,
    pub project_id: String,
    pub blocks_json: String,
    pub variables_json: String,
    pub label: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: String,
    pub project_id: String,
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub response: String,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost: f64,
    pub latency_ms: i64,
    pub created_at: i64,
}
