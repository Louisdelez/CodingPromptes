use inkwell_core::types::*;
use inkwell_core::api_client::ApiClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::mpsc;

#[allow(dead_code)]
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
    pub auth_mode: AuthMode,
    pub session: Option<AuthSession>,
    pub server_url_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub email_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub password_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    // Block input states (one per block)
    pub block_inputs: Vec<Option<gpui::Entity<gpui_component::input::InputState>>>,
    // Save
    pub save_pending: bool,
    pub save_timer: u32,
    // Versions
    pub versions: Vec<inkwell_core::types::Version>,
    // Fleet
    pub gpu_nodes: Vec<inkwell_core::types::GpuNode>,
    // STT
    pub stt_recording: bool,
    pub stt_target_block: Option<usize>,
    pub stt_stop_tx: Option<mpsc::Sender<()>>,
    // Settings
    pub show_settings: bool,
    pub api_key_openai: String,
    pub api_key_anthropic: String,
    pub api_key_google: String,
    // Project
    pub project: Project,
    pub projects: Vec<ProjectSummary>,
    pub workspaces: Vec<Workspace>,
    // UI
    pub left_tab: LeftTab,
    pub right_tab: RightTab,
    pub left_open: bool,
    pub right_open: bool,
    pub dark_mode: bool,
    pub show_add_menu: bool,
    pub custom_frameworks: Vec<CustomFramework>,
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
    pub terminal_input_tx: Option<mpsc::Sender<String>>,
    pub terminal_input_buf: String,
    pub terminal_input_entity: Option<gpui::Entity<gpui_component::input::InputState>>,
    // Undo
    pub undo_stack: Vec<Vec<Block>>,
    // Persistence
    pub confirm_delete: Option<String>, // project id to confirm delete
    pub search_query: String,
    // Async message channel
    pub msg_rx: mpsc::Receiver<AsyncMsg>,
    pub msg_tx: mpsc::Sender<AsyncMsg>,
}

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
    // Custom frameworks
    CustomFrameworkSaved,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Screen { Auth, Ide }

#[derive(Clone, Copy, PartialEq)]
pub enum AuthMode { Login, Register }

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum LeftTab { Library, Frameworks, Versions }

#[derive(Clone, Copy, PartialEq)]
pub enum RightTab { Preview, Playground, Stt, History, Export, Fleet, Terminal }

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

#[derive(Clone)]
#[allow(dead_code)]
pub struct Block {
    pub id: String,
    pub block_type: BlockType,
    pub content: String,
    pub enabled: bool,
    pub editing: bool,
}

#[derive(Clone)]
pub struct CustomFramework {
    pub name: String,
    pub blocks: Vec<(inkwell_core::types::BlockType, String)>,
}

pub struct ProjectSummary {
    pub id: String,
    pub name: String,
}

impl AppState {
    pub fn new() -> Self {
        let saved = crate::persistence::load_session();
        let server_url = if saved.server_url.is_empty() { "http://localhost:8910".to_string() } else { saved.server_url };
        let (msg_tx, msg_rx) = mpsc::channel();

        // If we have a saved token, try to auto-login
        let has_token = !saved.token.is_empty();

        Self {
            screen: if has_token { Screen::Ide } else { Screen::Auth },
            lang: if saved.lang.is_empty() { "fr".into() } else { saved.lang },
            api: Arc::new(Mutex::new(ApiClient::new(&server_url))),
            server_url,
            email: saved.email.clone(),
            password: String::new(),
            display_name: String::new(),
            server_url_input: None,
            email_input: None,
            password_input: None,
            block_inputs: vec![],
            save_pending: false,
            save_timer: 0,
            versions: vec![],
            gpu_nodes: vec![],
            stt_recording: false,
            stt_target_block: None,
            stt_stop_tx: None,
            show_settings: false,
            api_key_openai: String::new(),
            api_key_anthropic: String::new(),
            api_key_google: String::new(),
            auth_error: None,
            auth_loading: false,
            auth_mode: AuthMode::Login,
            session: None,
            project: Project::default_prompt(),
            projects: vec![],
            workspaces: vec![],
            left_tab: LeftTab::Library,
            right_tab: RightTab::Preview,
            left_open: saved.left_open || !has_token,
            right_open: saved.right_open || !has_token,
            dark_mode: saved.dark_mode,
            show_add_menu: false,
            custom_frameworks: vec![],
            selected_model: "gpt-4o-mini".into(),
            sdd_description: String::new(),
            sdd_running: false,
            playground_response: String::new(),
            playground_loading: false,
            editing_block_idx: None,
            terminal_output: String::new(),
            terminal_running: false,
            terminal_input_tx: None,
            terminal_input_buf: String::new(),
            terminal_input_entity: None,
            undo_stack: vec![],
            confirm_delete: None,
            search_query: String::new(),
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
