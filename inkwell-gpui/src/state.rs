use inkwell_core::types::*;
use inkwell_core::api_client::ApiClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::mpsc;

pub struct AppState {
    pub screen: Screen,
    pub lang: String,
    pub api: Arc<Mutex<ApiClient>>,
    // Auth
    pub server_url: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub auth_error: Option<String>,
    pub auth_loading: bool,
    pub session: Option<AuthSession>,
    // Project
    pub project: Project,
    pub projects: Vec<ProjectSummary>,
    pub workspaces: Vec<Workspace>,
    // UI
    pub left_tab: LeftTab,
    pub right_tab: RightTab,
    pub left_open: bool,
    pub right_open: bool,
    pub show_add_menu: bool,
    pub selected_model: String,
    // SDD
    pub sdd_description: String,
    pub sdd_running: bool,
    // Playground
    pub playground_response: String,
    pub playground_loading: bool,
    // Editing
    pub editing_block_idx: Option<usize>,
    // Terminal
    pub terminal_output: String,
    pub terminal_running: bool,
    // Async message channel
    pub msg_rx: mpsc::Receiver<AsyncMsg>,
    pub msg_tx: mpsc::Sender<AsyncMsg>,
}

#[derive(Debug)]
pub enum AsyncMsg {
    AuthSuccess { session: AuthSession, projects: Vec<PromptProject>, workspaces: Vec<Workspace> },
    AuthError(String),
    LlmResponse(String),
    LlmDone,
    LlmError(String),
    TerminalOutput(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Screen { Auth, Ide }

#[derive(Clone, Copy, PartialEq)]
pub enum LeftTab { Library, Frameworks, Versions }

#[derive(Clone, Copy, PartialEq)]
pub enum RightTab { Preview, Playground, Stt, History, Export, Fleet, Terminal }

pub struct Project {
    pub id: String,
    pub name: String,
    pub workspace_id: Option<String>,
    pub blocks: Vec<Block>,
    pub variables: HashMap<String, String>,
    pub tags: Vec<String>,
    pub framework: Option<String>,
}

pub struct Block {
    pub id: String,
    pub block_type: BlockType,
    pub content: String,
    pub enabled: bool,
    pub editing: bool,
}

#[derive(Clone)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
}

impl AppState {
    pub fn new() -> Self {
        let server_url = "http://localhost:8910".to_string();
        let (msg_tx, msg_rx) = mpsc::channel();
        Self {
            screen: Screen::Auth,
            lang: "fr".into(),
            api: Arc::new(Mutex::new(ApiClient::new(&server_url))),
            server_url,
            email: String::new(),
            password: String::new(),
            display_name: String::new(),
            auth_error: None,
            auth_loading: false,
            session: None,
            project: Project::default_prompt(),
            projects: vec![],
            workspaces: vec![],
            left_tab: LeftTab::Library,
            right_tab: RightTab::Preview,
            left_open: true,
            right_open: true,
            show_add_menu: false,
            selected_model: "gpt-4o-mini".into(),
            sdd_description: String::new(),
            sdd_running: false,
            playground_response: String::new(),
            playground_loading: false,
            editing_block_idx: None,
            terminal_output: String::new(),
            terminal_running: false,
            msg_rx,
            msg_tx,
        }
    }
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

    pub fn compiled_prompt(&self) -> String {
        let blocks: Vec<inkwell_core::types::PromptBlock> = self.blocks.iter()
            .map(|b| inkwell_core::types::PromptBlock {
                id: b.id.clone(),
                block_type: b.block_type,
                content: b.content.clone(),
                enabled: b.enabled,
            })
            .collect();
        inkwell_core::prompt::compile_prompt(&blocks, &self.variables)
    }

    pub fn token_count(&self) -> usize {
        // Simple word-based approximation (4 chars ≈ 1 token)
        let text = self.compiled_prompt();
        (text.len() as f64 / 4.0).ceil() as usize
    }

    pub fn char_count(&self) -> usize {
        self.compiled_prompt().len()
    }

    pub fn word_count(&self) -> usize {
        let text = self.compiled_prompt();
        if text.is_empty() { 0 } else { text.split_whitespace().count() }
    }
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            block_type,
            content: String::new(),
            enabled: true,
            editing: false,
        }
    }
}
