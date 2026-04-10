// Re-export pure types so `use crate::state::*` still works everywhere
pub use crate::types::*;

use inkwell_core::types::*;
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;

#[allow(dead_code)]
pub struct AppState {
    pub screen: Screen,
    pub lang: String,
    // Auth
    pub server_url: String,
    pub auth_error: Option<String>,
    pub auth_loading: bool,
    pub auth_mode: AuthMode,
    pub session: Option<AuthSession>,
    pub server_url_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub email_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub password_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    // Block input states (one per block)
    pub block_inputs: Vec<Option<gpui::Entity<gpui_component::input::InputState>>>,
    // Chat
    pub chat_system_prompt: String,
    // Save status
    pub save_status: &'static str, // "idle" | "saving" | "saved"
    pub save_status_timer: u32,
    // Project name editing
    pub editing_name: bool,
    pub name_input_entity: Option<gpui::Entity<gpui_component::input::InputState>>,
    // Playground
    pub playground_temperature: f32,
    pub playground_max_tokens: u32,
    pub playground_selected_models: Vec<String>,
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
    pub api_key_openai_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub api_key_anthropic_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub api_key_google_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub ssh_port_input: Option<gpui::Entity<gpui_component::input::InputState>>,
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
    pub sdd_running: bool,
    // Playground
    pub playground_response: String,
    pub playground_loading: bool,
    // Chat
    pub chat_messages: Vec<(String, String)>, // (role, content)
    pub chat_input_entity: Option<gpui::Entity<gpui_component::input::InputState>>,
    // Terminal (multi-session)
    pub terminal_sessions: Vec<TerminalSession>,
    pub active_terminal: usize,
    pub terminal_input_entity: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub show_ssh_modal: bool,
    pub ssh_host: String,
    pub ssh_user: String,
    pub ssh_port: String,
    pub ssh_host_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub ssh_user_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub tag_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub version_label_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    // Cached computed values (invalidated on block change)
    pub cached_prompt: String,
    pub cached_tokens: usize,
    pub cached_chars: usize,
    pub cached_words: usize,
    pub cached_lines: usize,
    pub prompt_dirty: bool,
    pub cached_vars: Vec<String>,
    // Undo
    pub undo_stack: VecDeque<Vec<Block>>,
    // Persistence
    pub confirm_delete: Option<String>, // project id to confirm delete
    pub search_query: String,
    pub search_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    pub variable_inputs: HashMap<String, gpui::Entity<gpui_component::input::InputState>>,
    // Execution history
    pub executions: Vec<Execution>,
    // Multi-model
    pub multi_model_responses: Vec<(String, String)>, // (model, response)
    pub multi_model_loading: bool,
    // STT provider
    pub stt_provider: SttProvider,
    // Analytics
    pub analytics_range: AnalyticsRange,
    // GitHub
    pub github_repo: String,
    pub github_repo_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    // Collab
    pub collab_users: Vec<CollabUser>,
    // Workspace rename
    pub editing_workspace_id: Option<String>,
    pub workspace_name_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    // Profile
    pub show_profile: bool,
    // Framework name input
    pub framework_name_input: Option<gpui::Entity<gpui_component::input::InputState>>,
    // Workspace color
    pub selected_workspace_color: String,
    // Copy feedback
    pub copy_feedback: u32, // countdown frames for "Copied!" display
    // Auto-poll timers
    pub fleet_poll_timer: u32,
    pub collab_poll_timer: u32,
    // Frame counter for throttling
    pub frame_count: u32,
    pub inputs_initialized: bool,
    // Async message channel
    pub msg_rx: mpsc::Receiver<AsyncMsg>,
    pub msg_tx: mpsc::Sender<AsyncMsg>,
}

impl AppState {
    pub fn new() -> Self {
        let (msg_tx, msg_rx) = mpsc::channel();
        Self::new_with_channel(msg_tx, msg_rx)
    }

    pub fn new_with_channel(msg_tx: mpsc::Sender<AsyncMsg>, msg_rx: mpsc::Receiver<AsyncMsg>) -> Self {
        let saved = crate::persistence::load_session();
        let server_url = if saved.server_url.is_empty() { "http://localhost:8910".to_string() } else { saved.server_url };
        let local_projects = crate::persistence::load_all_projects();
        let local_settings = crate::persistence::load_settings();
        let local_frameworks = crate::persistence::load_frameworks();

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
            let summaries: Vec<ProjectSummary> = local_projects.iter()
                .map(|p| ProjectSummary { id: p.id.clone(), name: p.name.clone(), workspace_id: None })
                .collect();
            (proj, summaries)
        } else {
            (Project::default_prompt(), vec![])
        };

        let custom_frameworks: Vec<CustomFramework> = local_frameworks.iter()
            .map(|f| CustomFramework { name: f.name.clone(), blocks: f.blocks.clone() })
            .collect();

        Self {
            screen: Screen::Ide, lang: if saved.lang.is_empty() { "fr".into() } else { saved.lang },
            server_url, server_url_input: None, email_input: None, password_input: None,
            block_inputs: vec![], chat_system_prompt: String::new(),
            save_status: "idle", save_status_timer: 0, editing_name: false, name_input_entity: None,
            playground_temperature: 0.7, playground_max_tokens: 2048,
            playground_selected_models: vec!["gpt-4o-mini".into()],
            save_pending: false, save_timer: 0, versions: vec![], gpu_nodes: vec![],
            stt_recording: false, stt_target_block: None, stt_stop_tx: None,
            show_settings: false,
            api_key_openai: local_settings.api_key_openai,
            api_key_anthropic: local_settings.api_key_anthropic,
            api_key_google: local_settings.api_key_google,
            api_key_openai_input: None, api_key_anthropic_input: None, api_key_google_input: None,
            ssh_port_input: None, auth_error: None, auth_loading: false, auth_mode: AuthMode::Login,
            session: None, project, projects: project_summaries, workspaces: vec![],
            left_tab: LeftTab::Library, right_tab: RightTab::Preview,
            left_open: saved.left_open, right_open: saved.right_open,
            dark_mode: saved.dark_mode, show_add_menu: false, custom_frameworks,
            selected_model: if local_settings.selected_model.is_empty() { "gpt-4o-mini".into() } else { local_settings.selected_model },
            sdd_running: false, playground_response: String::new(), playground_loading: false,
            chat_messages: vec![], chat_input_entity: None,
            terminal_sessions: vec![], active_terminal: 0, terminal_input_entity: None,
            show_ssh_modal: false, ssh_host: String::new(), ssh_user: String::new(), ssh_port: "22".into(),
            ssh_host_input: None, ssh_user_input: None, tag_input: None, version_label_input: None,
            cached_prompt: String::new(), cached_tokens: 0, cached_chars: 0, cached_words: 0, cached_lines: 0,
            prompt_dirty: true, cached_vars: vec![], undo_stack: VecDeque::new(),
            confirm_delete: None, search_query: String::new(), search_input: None,
            variable_inputs: HashMap::new(), executions: vec![],
            multi_model_responses: vec![], multi_model_loading: false,
            stt_provider: SttProvider::Local, analytics_range: AnalyticsRange::All,
            github_repo: local_settings.github_repo, github_repo_input: None,
            collab_users: vec![], editing_workspace_id: None, workspace_name_input: None,
            show_profile: false, framework_name_input: None, selected_workspace_color: "#6366f1".into(),
            copy_feedback: 0, fleet_poll_timer: 0, collab_poll_timer: 0,
            frame_count: 0, inputs_initialized: false, msg_rx, msg_tx,
        }
    }
}
