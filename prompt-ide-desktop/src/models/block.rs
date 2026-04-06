use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    Role,
    Context,
    Task,
    Examples,
    Constraints,
    Format,
}

impl BlockType {
    pub fn all() -> &'static [BlockType] {
        &[
            BlockType::Role,
            BlockType::Context,
            BlockType::Task,
            BlockType::Examples,
            BlockType::Constraints,
            BlockType::Format,
        ]
    }

    pub fn label(&self, lang: &str) -> &'static str {
        match (self, lang) {
            (BlockType::Role, "fr") => "Role / Persona",
            (BlockType::Role, _) => "Role / Persona",
            (BlockType::Context, "fr") => "Contexte",
            (BlockType::Context, _) => "Context",
            (BlockType::Task, "fr") => "Tache / Directive",
            (BlockType::Task, _) => "Task / Directive",
            (BlockType::Examples, "fr") => "Exemples (Few-shot)",
            (BlockType::Examples, _) => "Examples (Few-shot)",
            (BlockType::Constraints, "fr") => "Contraintes",
            (BlockType::Constraints, _) => "Constraints",
            (BlockType::Format, "fr") => "Format de sortie",
            (BlockType::Format, _) => "Output format",
        }
    }

    pub fn color(&self) -> iced::Color {
        match self {
            BlockType::Role => iced::Color::from_rgb(0.655, 0.545, 0.984),
            BlockType::Context => iced::Color::from_rgb(0.376, 0.647, 0.984),
            BlockType::Task => iced::Color::from_rgb(0.204, 0.827, 0.600),
            BlockType::Examples => iced::Color::from_rgb(0.984, 0.749, 0.141),
            BlockType::Constraints => iced::Color::from_rgb(0.973, 0.443, 0.443),
            BlockType::Format => iced::Color::from_rgb(0.580, 0.639, 0.722),
        }
    }

    pub fn placeholder(&self, lang: &str) -> &'static str {
        match (self, lang) {
            (BlockType::Role, "fr") => "Tu es un expert en...",
            (BlockType::Role, _) => "You are an expert in...",
            (BlockType::Context, "fr") => "Informations de fond, donnees...",
            (BlockType::Context, _) => "Background information, data...",
            (BlockType::Task, "fr") => "Redige, Analyse, Compare...",
            (BlockType::Task, _) => "Write, Analyze, Compare...",
            (BlockType::Examples, _) => "<example>\nInput: ...\nOutput: ...\n</example>",
            (BlockType::Constraints, "fr") => "- Maximum 500 mots\n- Ton professionnel",
            (BlockType::Constraints, _) => "- Maximum 500 words\n- Professional tone",
            (BlockType::Format, "fr") => "JSON, Markdown, liste numerotee...",
            (BlockType::Format, _) => "JSON, Markdown, numbered list...",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptBlock {
    pub id: String,
    pub block_type: BlockType,
    pub content: String,
    pub enabled: bool,
}

impl PromptBlock {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            block_type,
            content: String::new(),
            enabled: true,
        }
    }
}
