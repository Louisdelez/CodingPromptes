use serde::{Deserialize, Serialize};

// Block types — matches TypeScript BlockType exactly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum BlockType {
    Role,
    Context,
    Task,
    Examples,
    Constraints,
    Format,
    SddConstitution,
    SddSpecification,
    SddPlan,
    SddTasks,
    SddImplementation,
}

impl BlockType {
    pub fn is_sdd(&self) -> bool {
        matches!(self, Self::SddConstitution | Self::SddSpecification | Self::SddPlan | Self::SddTasks | Self::SddImplementation)
    }

    /// Parse a canonical lowercase / kebab-case block type name.
    /// Returns None for unknown names (callers should surface a clear error).
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "role" => Some(Self::Role),
            "context" => Some(Self::Context),
            "task" => Some(Self::Task),
            "examples" => Some(Self::Examples),
            "constraints" => Some(Self::Constraints),
            "format" => Some(Self::Format),
            "sdd-constitution" | "sdd_constitution" => Some(Self::SddConstitution),
            "sdd-specification" | "sdd_specification" => Some(Self::SddSpecification),
            "sdd-plan" | "sdd_plan" => Some(Self::SddPlan),
            "sdd-tasks" | "sdd_tasks" => Some(Self::SddTasks),
            "sdd-implementation" | "sdd_implementation" => Some(Self::SddImplementation),
            _ => None,
        }
    }

    pub const ALL_NAMES: &'static [&'static str] = &[
        "role", "context", "task", "examples", "constraints", "format",
        "sdd-constitution", "sdd-specification", "sdd-plan", "sdd-tasks", "sdd-implementation",
    ];

    pub fn color(&self) -> &'static str {
        match self {
            Self::Role => "#a78bfa",
            Self::Context => "#60a5fa",
            Self::Task => "#34d399",
            Self::Examples => "#fbbf24",
            Self::Constraints => "#f87171",
            Self::Format => "#94a3b8",
            Self::SddConstitution => "#a78bfa",
            Self::SddSpecification => "#60a5fa",
            Self::SddPlan => "#34d399",
            Self::SddTasks => "#fbbf24",
            Self::SddImplementation => "#f87171",
        }
    }

    pub fn label(&self, lang: &str) -> &'static str {
        match (self, lang) {
            (Self::Role, "fr") => "Role / Persona",
            (Self::Role, _) => "Role / Persona",
            (Self::Context, "fr") => "Contexte",
            (Self::Context, _) => "Context",
            (Self::Task, "fr") => "Tache / Directive",
            (Self::Task, _) => "Task / Directive",
            (Self::Examples, "fr") => "Exemples (Few-shot)",
            (Self::Examples, _) => "Examples (Few-shot)",
            (Self::Constraints, "fr") => "Contraintes",
            (Self::Constraints, _) => "Constraints",
            (Self::Format, "fr") => "Format de sortie",
            (Self::Format, _) => "Output format",
            (Self::SddConstitution, _) => "Constitution",
            (Self::SddSpecification, _) => "Specification",
            (Self::SddPlan, _) => "Plan",
            (Self::SddTasks, "fr") => "Taches",
            (Self::SddTasks, _) => "Tasks",
            (Self::SddImplementation, _) => "Implementation",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptBlock {
    pub id: String,
    #[serde(rename = "type")]
    pub block_type: BlockType,
    pub content: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: String,
    pub color: String,
    pub constitution: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptProject {
    pub id: String,
    pub name: String,
    #[serde(rename = "workspaceId")]
    pub workspace_id: Option<String>,
    pub blocks: Vec<PromptBlock>,
    pub variables: std::collections::HashMap<String, String>,
    pub tags: Vec<String>,
    pub framework: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
    pub max_context: u64,
    pub node_address: Option<String>,
    pub node_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub avatar: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuNode {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub gpu_info: String,
    pub status: String,
    pub address: String,
    pub capabilities_json: String,
    pub last_heartbeat: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    pub project_id: String,
    pub blocks_json: String,
    pub variables_json: String,
    pub label: String,
    pub created_at: i64,
}
