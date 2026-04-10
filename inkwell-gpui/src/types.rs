//! Pure types — no GPUI dependencies, no InputState, no Entity.
//! These are data structures shared across the app.

use inkwell_core::types::*;
use std::collections::HashMap;
use std::sync::mpsc;

// ── Screens & Modes ──

#[derive(Clone, Copy, PartialEq)]
pub enum Screen { Auth, Ide }

#[derive(Clone, Copy, PartialEq)]
pub enum AuthMode { Login, Register }

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum LeftTab { Library, Frameworks, Versions }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RightTab {
    Preview, Playground, Stt, History, Export, Fleet,
    Terminal, Optimize, Lint, Chat, Analytics, Collab, Chain,
}

// ── Project & Blocks ──

#[derive(Clone)]
#[allow(dead_code)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub workspace_id: Option<String>,
    pub blocks: Vec<Block>,
    pub variables: HashMap<String, String>,
    pub tags: Vec<String>,
    pub framework: Option<String>,
}

impl Project {
    pub fn default_prompt() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Nouveau prompt".into(),
            workspace_id: None,
            blocks: vec![
                Block::new(BlockType::Role),
                Block::new(BlockType::Context),
                Block::new(BlockType::Task),
            ],
            variables: HashMap::new(),
            tags: vec![],
            framework: None,
        }
    }

    #[allow(dead_code)]
    pub fn compiled_prompt(&self) -> String {
        let blocks: Vec<PromptBlock> = self.blocks.iter()
            .map(|b| PromptBlock {
                id: b.id.clone(), block_type: b.block_type,
                content: b.content.clone(), enabled: b.enabled,
            })
            .collect();
        inkwell_core::prompt::compile_prompt(&blocks, &self.variables)
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Block {
    pub id: String,
    pub block_type: BlockType,
    pub content: String,
    pub enabled: bool,
    pub editing: bool,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            block_type, content: String::new(),
            enabled: true, editing: false,
        }
    }
}

// ── Summaries & Collections ──

#[derive(Clone)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub workspace_id: Option<String>,
}

#[derive(Clone)]
pub struct CustomFramework {
    pub name: String,
    pub blocks: Vec<(BlockType, String)>,
}

// ── Terminal ──

pub struct TerminalSession {
    pub label: String,
    pub output: String,
    pub running: bool,
    pub input_tx: Option<mpsc::Sender<String>>,
}

// ── Execution History ──

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Execution {
    pub model: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub latency_ms: u64,
    pub cost: f64,
    pub timestamp: i64,
    pub prompt_preview: String,
    pub response_preview: String,
}

// ── Enums ──

#[derive(Clone, Copy, PartialEq)]
pub enum SttProvider { Local, OpenaiWhisper, Groq, Deepgram }

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum AnalyticsRange { Week, Month, All }

// ── Collaboration ──

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CollabUser {
    pub name: String,
    pub email: String,
    pub online: bool,
}

// ── Async Messages ──

#[derive(Debug)]
#[allow(dead_code)]
pub enum AsyncMsg {
    AuthSuccess { session: AuthSession, projects: Vec<PromptProject>, workspaces: Vec<Workspace> },
    AuthError(String),
    LlmResponse(String),
    LlmDone,
    LlmError(String),
    TerminalOutput(String),
    SddBlockResult { idx: usize, content: String },
    ExportReady(String),
    VersionsLoaded(Vec<inkwell_core::types::Version>),
    NodesLoaded(Vec<inkwell_core::types::GpuNode>),
    LlmChunk(String),
    SttResult { block_idx: usize, text: String },
    SttError(String),
    CustomFrameworkSaved,
    MultiModelResult { model: String, response: String },
    MultiModelDone,
    ExecutionRecorded(Execution),
    CollabUsersLoaded(Vec<CollabUser>),
    GitHubPushed(String),
}
