use gpui::*;
use crate::state::*;
use inkwell_core::types::Workspace;
use std::sync::mpsc;

/// Events emitted by AppStore — children subscribe to these
#[derive(Clone, Debug)]
pub enum StoreEvent {
    ProjectChanged,
    PromptCacheUpdated,
    PlaygroundUpdated,
    ChatMessageReceived,
    TerminalOutput,
    SettingsChanged,
    SaveStatusChanged,
    SessionChanged,
    BlockContentChanged(usize),
    SwitchRightTab(RightTab),
    CloseAllMenus,
}

/// Shared application data — all entities read from this
pub struct AppStore {
    // Core
    pub screen: Screen,
    pub lang: String,
    pub server_url: String,
    pub dark_mode: bool,

    // Auth
    pub auth_error: Option<String>,
    pub auth_loading: bool,
    pub auth_mode: AuthMode,
    pub session: Option<inkwell_core::types::AuthSession>,

    // Project
    pub project: Project,
    pub projects: Vec<ProjectSummary>,
    pub workspaces: Vec<Workspace>,
    pub custom_frameworks: Vec<CustomFramework>,
    pub selected_model: String,

    // Cached prompt
    pub cached_prompt: String,
    pub cached_tokens: usize,
    pub cached_chars: usize,
    pub cached_words: usize,
    pub cached_lines: usize,
    pub cached_vars: Vec<String>,
    pub prompt_dirty: bool,

    // Playground
    pub playground_response: String,
    pub playground_loading: bool,
    pub playground_temperature: f32,
    pub playground_max_tokens: u32,
    pub playground_selected_models: Vec<String>,
    pub multi_model_responses: Vec<(String, String)>,
    pub multi_model_loading: bool,

    // Chat
    pub chat_messages: Vec<(String, String)>,
    pub chat_system_prompt: String,

    // Terminal
    pub terminal_sessions: Vec<TerminalSession>,
    pub active_terminal: usize,

    // SDD
    pub sdd_running: bool,

    // STT
    pub stt_recording: bool,
    pub stt_target_block: Option<usize>,
    pub stt_stop_tx: Option<mpsc::Sender<()>>,
    pub stt_provider: SttProvider,

    // Settings
    pub show_settings: bool,
    pub show_profile: bool,
    pub api_key_openai: String,
    pub api_key_anthropic: String,
    pub api_key_google: String,
    pub github_repo: String,

    // History
    pub versions: Vec<inkwell_core::types::Version>,
    pub executions: Vec<Execution>,
    pub gpu_nodes: Vec<inkwell_core::types::GpuNode>,

    // UI state
    pub left_tab: LeftTab,
    pub right_tab: RightTab,
    pub left_open: bool,
    pub right_open: bool,
    pub terminal_open: bool,
    pub left_width: f32,
    pub right_width: f32,
    pub show_add_menu: bool,
    pub show_ssh_modal: bool,
    pub editing_workspace_id: Option<String>,
    pub selected_workspace_color: String,
    pub confirm_delete: Option<String>,
    pub confirm_delete_block: Option<usize>,
    pub search_query: String,
    pub analytics_range: AnalyticsRange,
    pub collab_users: Vec<CollabUser>,

    // Save
    pub save_status: &'static str,
    pub save_status_timer: u32,
    pub save_pending: bool,
    pub save_timer: u32,

    // Undo
    pub undo_stack: std::collections::VecDeque<Vec<Block>>,

    // Timers
    pub copy_feedback: u32,
    pub fleet_poll_timer: u32,
    pub collab_poll_timer: u32,
    pub frame_count: u32,

    // Async
    pub msg_tx: mpsc::Sender<AsyncMsg>,
}

impl EventEmitter<StoreEvent> for AppStore {}

impl AppStore {
    pub fn new(msg_tx: mpsc::Sender<AsyncMsg>) -> Self {
        let saved = crate::persistence::load_session();
        let local_projects = crate::persistence::load_all_projects();
        let local_settings = crate::persistence::load_settings();
        let local_frameworks = crate::persistence::load_frameworks();
        let server_url = if saved.server_url.is_empty() { "http://localhost:8910".into() } else { saved.server_url };

        let (project, project_summaries) = if let Some(first) = local_projects.first() {
            let proj = Project {
                id: first.id.clone(), name: first.name.clone(),
                workspace_id: first.workspace_id.clone(),
                blocks: first.blocks.iter().map(|b| Block {
                    id: b.id.clone(), block_type: b.block_type,
                    content: b.content.clone(), enabled: b.enabled, editing: false,
                }).collect(),
                variables: first.variables.clone(), tags: first.tags.clone(),
                framework: first.framework.clone(),
            };
            let summaries = local_projects.iter().map(|p| ProjectSummary { id: p.id.clone(), name: p.name.clone(), workspace_id: None }).collect();
            (proj, summaries)
        } else {
            let default = Project::default_prompt();
            let summary = ProjectSummary { id: default.id.clone(), name: default.name.clone(), workspace_id: None };
            // Save default project to disk so it persists
            crate::persistence::save_project(&crate::persistence::LocalProject {
                id: default.id.clone(), name: default.name.clone(),
                workspace_id: None,
                blocks: default.blocks.iter().map(|b| inkwell_core::types::PromptBlock {
                    id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled,
                }).collect(),
                variables: std::collections::HashMap::new(), tags: vec![],
                framework: None, updated_at: chrono::Utc::now().timestamp_millis(),
            });
            (default, vec![summary])
        };

        let custom_frameworks = local_frameworks.iter()
            .map(|f| CustomFramework { name: f.name.clone(), blocks: f.blocks.clone() })
            .collect();

        Self {
            screen: Screen::Ide,
            lang: if saved.lang.is_empty() { "fr".into() } else { saved.lang },
            server_url, dark_mode: saved.dark_mode,
            auth_error: None, auth_loading: false, auth_mode: AuthMode::Login, session: None,
            project, projects: project_summaries, workspaces: vec![],
            custom_frameworks,
            selected_model: if local_settings.selected_model.is_empty() { "gpt-4o-mini".into() } else { local_settings.selected_model },
            cached_prompt: String::new(), cached_tokens: 0, cached_chars: 0,
            cached_words: 0, cached_lines: 0, cached_vars: vec![], prompt_dirty: true,
            playground_response: String::new(), playground_loading: false,
            playground_temperature: 0.7, playground_max_tokens: 2048,
            playground_selected_models: vec!["gpt-4o-mini".into()],
            multi_model_responses: vec![], multi_model_loading: false,
            chat_messages: vec![], chat_system_prompt: String::new(),
            terminal_sessions: vec![], active_terminal: 0,
            sdd_running: false,
            stt_recording: false, stt_target_block: None, stt_stop_tx: None, stt_provider: SttProvider::Local,
            show_settings: false, show_profile: false,
            api_key_openai: local_settings.api_key_openai,
            api_key_anthropic: local_settings.api_key_anthropic,
            api_key_google: local_settings.api_key_google,
            github_repo: local_settings.github_repo,
            versions: vec![], executions: vec![], gpu_nodes: vec![],
            left_tab: LeftTab::Library, right_tab: RightTab::Preview,
            left_open: true, right_open: true, terminal_open: false,
            left_width: 288.0, right_width: 384.0,
            show_add_menu: false, show_ssh_modal: false,
            editing_workspace_id: None, selected_workspace_color: "#6366f1".into(),
            confirm_delete: None, confirm_delete_block: None, search_query: String::new(),
            analytics_range: AnalyticsRange::All, collab_users: vec![],
            save_status: "idle", save_status_timer: 0, save_pending: false, save_timer: 0,
            undo_stack: std::collections::VecDeque::new(),
            copy_feedback: 0, fleet_poll_timer: 0, collab_poll_timer: 0, frame_count: 0,
            msg_tx,
        }
    }

    /// Refresh the cached prompt and related stats
    pub fn refresh_cache(&mut self) {
        let core_blocks: Vec<inkwell_core::types::PromptBlock> = self.project.blocks.iter().map(|b| {
            inkwell_core::types::PromptBlock { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled }
        }).collect();
        self.cached_prompt = inkwell_core::prompt::compile_prompt(&core_blocks, &self.project.variables);
        self.cached_tokens = (self.cached_prompt.len() as f64 / 4.0).ceil() as usize;
        self.cached_chars = self.cached_prompt.len();
        self.cached_words = if self.cached_prompt.is_empty() { 0 } else { self.cached_prompt.split_whitespace().count() };
        self.cached_lines = self.cached_prompt.lines().count();
        self.cached_vars = inkwell_core::prompt::extract_variables(&core_blocks);
        self.prompt_dirty = false;
    }
}
