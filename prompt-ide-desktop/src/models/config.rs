use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeMode,
    pub lang: String,
    pub openai_key: String,
    pub anthropic_key: String,
    pub google_key: String,
    pub groq_key: String,
    pub local_server_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            lang: "fr".into(),
            openai_key: String::new(),
            anthropic_key: String::new(),
            google_key: String::new(),
            groq_key: String::new(),
            local_server_url: "http://localhost:8910".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDef {
    pub id: &'static str,
    pub name: &'static str,
    pub provider: &'static str,
    pub input_cost: f64,
    pub output_cost: f64,
}

pub fn available_models() -> Vec<ModelDef> {
    vec![
        ModelDef { id: "gpt-4o", name: "GPT-4o", provider: "openai", input_cost: 0.0025, output_cost: 0.01 },
        ModelDef { id: "gpt-4o-mini", name: "GPT-4o Mini", provider: "openai", input_cost: 0.00015, output_cost: 0.0006 },
        ModelDef { id: "gpt-4.1", name: "GPT-4.1", provider: "openai", input_cost: 0.002, output_cost: 0.008 },
        ModelDef { id: "gpt-4.1-mini", name: "GPT-4.1 Mini", provider: "openai", input_cost: 0.0004, output_cost: 0.0016 },
        ModelDef { id: "o3-mini", name: "o3-mini", provider: "openai", input_cost: 0.0011, output_cost: 0.0044 },
        ModelDef { id: "claude-sonnet-4-6", name: "Claude Sonnet 4.6", provider: "anthropic", input_cost: 0.003, output_cost: 0.015 },
        ModelDef { id: "claude-opus-4-6", name: "Claude Opus 4.6", provider: "anthropic", input_cost: 0.015, output_cost: 0.075 },
        ModelDef { id: "claude-haiku-4-5", name: "Claude Haiku 4.5", provider: "anthropic", input_cost: 0.0008, output_cost: 0.004 },
        ModelDef { id: "gemini-2.5-pro", name: "Gemini 2.5 Pro", provider: "google", input_cost: 0.00125, output_cost: 0.01 },
        ModelDef { id: "gemini-2.5-flash", name: "Gemini 2.5 Flash", provider: "google", input_cost: 0.00015, output_cost: 0.0006 },
    ]
}
