use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName, Theme, ThemeMode};
use crate::state::*;
use inkwell_core::types::BlockType;

// Actions for keyboard shortcuts
actions!(inkwell, [NewProject, ToggleTerminal, RunPrompt, ToggleSettings, Undo, SaveNow]);

/// Drag payload for panel resize handles
#[derive(Clone)]
struct ResizeDrag {
    side: ResizeSide,
    start_x: f32,
    start_width: f32,
}

#[derive(Clone, Copy)]
enum ResizeSide { Left, Right }

impl Render for ResizeDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(4.0)).h(px(40.0)).bg(accent()).rounded(px(2.0))
    }
}

// Import shared UI modules
use crate::ui::colors::*;

// Global tokio runtime — reused by all async operations (avoids creating 25+ runtimes)
pub fn rt() -> &'static tokio::runtime::Runtime {
    use once_cell::sync::Lazy;
    static RT: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
    });
    &RT
}

/// Build an LLM request with proper API routing and auth headers.
/// Routes to OpenAI/Anthropic/Google directly if API keys are set, else falls back to server proxy.
pub fn llm_post(client: &reqwest::Client, model: &str, server_url: &str, body: serde_json::Value) -> reqwest::RequestBuilder {
    let (ko, ka, kg, _) = crate::llm::load_local_keys();
    let (url, hdrs) = crate::llm::llm_endpoint(model, &ko, &ka, &kg, server_url);
    let msgs = body["messages"].as_array().cloned().unwrap_or_default();
    let rebuilt = crate::llm::build_llm_body(model, &msgs,
        body["temperature"].as_f64().unwrap_or(0.7) as f32,
        body["max_tokens"].as_u64().unwrap_or(4096) as u32,
        body["stream"].as_bool().unwrap_or(false),
    );
    let mut req = client.post(&url).json(&rebuilt);
    for (k, v) in &hdrs { req = req.header(k.as_str(), v.as_str()); }
    req
}

pub struct InkwellApp {
    pub state: AppState,
    pub store: Entity<crate::store::AppStore>,
    pub header: Entity<crate::components::header_bar::HeaderBar>,
    pub bottom_bar: Entity<crate::components::bottom_bar::BottomBar>,
    pub editor: Entity<crate::components::editor_pane::EditorPane>,
    pub left_panel: Entity<crate::components::left_panel::LeftPanel>,
    pub right_panel: Entity<crate::components::right_panel::RightPanel>,
}

impl InkwellApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (msg_tx, msg_rx) = std::sync::mpsc::channel();
        let store = cx.new(|_cx| crate::store::AppStore::new(msg_tx.clone()));
        let header = cx.new(|cx| crate::components::header_bar::HeaderBar::new(store.clone(), cx));
        let bottom_bar = cx.new(|cx| crate::components::bottom_bar::BottomBar::new(store.clone(), cx));
        let editor = cx.new(|cx| crate::components::editor_pane::EditorPane::new(store.clone(), window, cx));
        let left_panel = cx.new(|cx| crate::components::left_panel::LeftPanel::new(store.clone(), window, cx));
        let right_panel = cx.new(|cx| crate::components::right_panel::RightPanel::new(store.clone(), window, cx));

        let mut state = AppState::new_with_channel(msg_tx.clone(), msg_rx);
        state.dark_mode = store.read(cx).dark_mode;

        // Sync store changes back to state (temporary bridge during migration)
        cx.subscribe(&store, |this: &mut Self, _, event: &crate::store::StoreEvent, cx| {
            match event {
                crate::store::StoreEvent::SettingsChanged => {
                    let s = this.store.read(cx);
                    this.state.dark_mode = s.dark_mode;
                    this.state.lang = s.lang.clone();
                    this.state.show_settings = s.show_settings;
                    this.state.show_profile = s.show_profile;
                    this.state.left_open = s.left_open;
                    this.state.right_open = s.right_open;
                }
                crate::store::StoreEvent::ProjectChanged => {
                    let s = this.store.read(cx);
                    this.state.project.name = s.project.name.clone();
                    this.state.save_pending = s.save_pending;
                }
                crate::store::StoreEvent::SwitchRightTab(tab) => {
                    this.state.right_tab = *tab;
                    this.state.right_open = true;
                }
                _ => {}
            }
        }).detach();

        Self { state, store, header, bottom_bar, editor, left_panel, right_panel }
    }

    fn t(&self) -> crate::theme::InkwellTheme {
        crate::theme::InkwellTheme::from_mode(self.state.dark_mode)
    }
}

impl InkwellApp {
    /// Start a periodic timer for background work (sync, timers, polling).
    /// This runs OUTSIDE of render — doesn't force re-renders.
    fn start_periodic_sync(cx: &mut Context<Self>) {
        // Run every 100ms (~10fps) instead of every frame (60fps)
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(std::time::Duration::from_millis(100)).await;
                let should_continue = this.update(cx, |this, cx| {
                    // Poll async messages
                    this.poll_messages(cx);

                    // Sync editor content to store
                    let changed = this.editor.update(cx, |e, cx| e.sync_content(cx));
                    if changed {
                        this.store.update(cx, |s, cx| {
                            if s.prompt_dirty {
                                s.refresh_cache();
                                cx.emit(crate::store::StoreEvent::PromptCacheUpdated);
                            }
                        });
                        this.state.save_pending = true;
                    }

                    // Sync old state inputs (bridge)
                    this.sync_block_content(cx);

                    // Timers
                    if this.state.copy_feedback > 0 { this.state.copy_feedback = this.state.copy_feedback.saturating_sub(6); }
                    if this.state.save_status_timer > 0 {
                        this.state.save_status_timer = this.state.save_status_timer.saturating_sub(6);
                        if this.state.save_status_timer == 0 && this.state.save_status == "saved" {
                            this.state.save_status = "idle";
                            this.store.update(cx, |s, cx| {
                                s.save_status = "idle";
                                cx.emit(crate::store::StoreEvent::SaveStatusChanged);
                            });
                        }
                    }

                    // Auto-save
                    if this.state.save_pending && this.state.save_timer == 0 {
                        this.state.save_timer = 5; // ~500ms
                    }
                    if this.state.save_timer > 0 {
                        this.state.save_timer -= 1;
                        if this.state.save_timer == 0 && this.state.save_pending {
                            this.state.save_status = "saved";
                            this.state.save_status_timer = 30;
                            this.state.save_pending = false;
                            this.save_to_backend();
                            this.store.update(cx, |s, cx| {
                                s.save_status = "saved";
                                cx.emit(crate::store::StoreEvent::SaveStatusChanged);
                            });
                        }
                    }
                }).ok();
                if should_continue.is_none() { break; }
            }
        }).detach();
    }
}

impl Render for InkwellApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // PURE render — zero state mutation

        // One-time init
        if !self.state.inputs_initialized {
            self.ensure_block_inputs(window, cx);
            self.ensure_terminal_input(window, cx);
            self.state.inputs_initialized = true;
            Self::start_periodic_sync(cx);
        }

        match self.state.screen {
            Screen::Auth => self.render_auth(window, cx),
            Screen::Ide => self.render_ide(cx),
        }
    }
}

impl InkwellApp {
    fn poll_messages(&mut self, cx: &mut Context<Self>) {
        // Limit messages per frame to avoid blocking render
        let mut count = 0;
        while count < 50 {
            let msg = match self.state.msg_rx.try_recv() {
                Ok(m) => m,
                Err(_) => break,
            };
            count += 1;
            match msg {
                AsyncMsg::AuthSuccess { session, projects, workspaces } => {
                    self.state.auth_loading = false;
                    crate::persistence::save_session(&crate::persistence::SavedSession {
                        server_url: self.state.server_url.clone(),
                        token: session.token.clone(),
                        email: session.email.clone(),
                        dark_mode: self.state.dark_mode,
                        lang: self.state.lang.clone(),
                        last_project_id: None,
                        left_open: self.state.left_open,
                        right_open: self.state.right_open,
                    });
                    self.state.session = Some(session);
                    self.state.screen = Screen::Ide;
                    // MERGE server projects with local — never overwrite local work
                    for sp in &projects {
                        if !self.state.projects.iter().any(|p| p.id == sp.id) {
                            // New project from server — save locally + add to list
                            let local = crate::persistence::LocalProject {
                                id: sp.id.clone(), name: sp.name.clone(),
                                workspace_id: None,
                                blocks: sp.blocks.clone(),
                                variables: std::collections::HashMap::new(),
                                tags: vec![], framework: sp.framework.clone(),
                                updated_at: chrono::Utc::now().timestamp_millis(),
                            };
                            crate::persistence::save_project(&local);
                            self.state.projects.push(ProjectSummary { id: sp.id.clone(), name: sp.name.clone(), workspace_id: None });
                        }
                    }
                    // Merge workspaces
                    for sw in &workspaces {
                        if !self.state.workspaces.iter().any(|w| w.id == sw.id) {
                            self.state.workspaces.push(sw.clone());
                        }
                    }
                    // Push local projects to server that server doesn't have
                    let local_projects = crate::persistence::load_all_projects();
                    let server_url = self.state.server_url.clone();
                    let token = self.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                    for lp in &local_projects {
                        if !projects.iter().any(|sp| sp.id == lp.id) {
                            crate::persistence::sync_project_to_server(&server_url, &token, lp);
                        }
                    }
                    // Don't touch current project — user keeps working on what they had open
                }
                AsyncMsg::AuthError(e) => {
                    self.state.auth_loading = false;
                    self.state.auth_error = Some(e);
                }
                AsyncMsg::LlmResponse(text) => {
                    if text.starts_with("__CHAT__") {
                        self.state.chat_messages.push(("assistant".into(), text[8..].to_string()));
                    } else if text.starts_with("__LOADPROJECT__") {
                        let json_str = &text[15..];
                        if let Ok(proj) = serde_json::from_str::<inkwell_core::types::PromptProject>(json_str) {
                            self.state.project.name = proj.name.clone();
                            self.state.project.id = proj.id.clone();
                            self.state.project.framework = proj.framework.clone();
                            self.state.project.blocks = proj.blocks.iter().map(|b| {
                                Block { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled, editing: false }
                            }).collect();
                            self.state.block_inputs.clear();
                            self.state.variable_inputs.clear();
                        }
                    } else if text.starts_with("__IMPORT__") {
                        let json_str = &text[10..];
                        if let Ok(blocks) = serde_json::from_str::<Vec<inkwell_core::types::PromptBlock>>(json_str) {
                            self.state.undo_stack.push_back(self.state.project.blocks.clone());
                            self.state.project.blocks = blocks.into_iter().map(|b| {
                                Block { id: b.id, block_type: b.block_type, content: b.content, enabled: b.enabled, editing: false }
                            }).collect();
                            self.state.block_inputs.clear();
                            self.state.playground_response = "Imported successfully!".into();
                        } else {
                            self.state.playground_response = "Invalid JSON format for import".into();
                        }
                    } else {
                        self.state.playground_response = text;
                    }
                }
                AsyncMsg::LlmChunk(text) => {
                    self.state.playground_response = text;
                }
                AsyncMsg::LlmDone => {
                    self.state.playground_loading = false;
                    self.state.sdd_running = false;
                    // Execution already tracked via ExecutionRecorded message (local).
                    // Optionally sync to server in background.
                    if !self.state.playground_response.is_empty() {
                        let token = self.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                        if !token.is_empty() {
                            let server = self.state.server_url.clone();
                            let project_id = self.state.project.id.clone();
                            let model = self.state.selected_model.clone();
                            let prompt = self.state.cached_prompt.clone();
                            let response = self.state.playground_response.clone();
                            rt().spawn(async move {
                                let mut client = inkwell_core::api_client::ApiClient::new(&server);
                                client.set_token(token);
                                let _ = client.create_execution(&project_id, &serde_json::json!({
                                    "model": model, "provider": "local", "prompt": prompt,
                                    "response": response, "tokens_in": 0, "tokens_out": 0,
                                    "cost": 0.0, "latency_ms": 0,
                                })).await;
                            });
                        }
                    }
                }
                AsyncMsg::LlmError(e) => {
                    self.state.playground_loading = false;
                    self.state.playground_response = format!("Error: {e}");
                }
                AsyncMsg::SddBlockResult { idx, content } => {
                    if let Some(block) = self.state.project.blocks.get_mut(idx) {
                        block.content = content.clone();
                    }
                    // Reset the input state for this block so it picks up new content
                    if idx < self.state.block_inputs.len() {
                        self.state.block_inputs[idx] = None; // Will be recreated next frame
                    }
                }
                AsyncMsg::ExportReady(path) => {
                    self.state.playground_response = format!("Exported to {path}");
                }
                AsyncMsg::VersionsLoaded(versions) => {
                    self.state.versions = versions;
                }
                AsyncMsg::NodesLoaded(nodes) => {
                    self.state.gpu_nodes = nodes;
                }
                AsyncMsg::SttResult { block_idx, text } => {
                    self.state.stt_recording = false;
                    if let Some(block) = self.state.project.blocks.get_mut(block_idx) {
                        if !block.content.is_empty() && !block.content.ends_with(' ') && !block.content.ends_with('\n') {
                            block.content.push(' ');
                        }
                        block.content.push_str(&text);
                    }
                    // Reset input to pick up new content
                    if block_idx < self.state.block_inputs.len() {
                        self.state.block_inputs[block_idx] = None;
                    }
                }
                AsyncMsg::SttError(e) => {
                    self.state.stt_recording = false;
                    self.state.playground_response = format!("STT Error: {e}");
                }
                AsyncMsg::CustomFrameworkSaved => {}
                AsyncMsg::MultiModelResult { model, response } => {
                    self.state.multi_model_responses.push((model, response));
                }
                AsyncMsg::MultiModelDone => {
                    self.state.multi_model_loading = false;
                }
                AsyncMsg::ExecutionRecorded(exec) => {
                    self.state.executions.push(exec);
                    // Cap at 500 executions to prevent unbounded growth
                    if self.state.executions.len() > 500 {
                        self.state.executions.drain(..self.state.executions.len() - 500);
                    }
                }
                AsyncMsg::CollabUsersLoaded(users) => {
                    self.state.collab_users = users;
                }
                AsyncMsg::GitHubPushed(msg) => {
                    self.state.playground_response = msg;
                }
                AsyncMsg::TerminalOutput(text) => {
                    let idx = self.state.active_terminal;
                    if let Some(session) = self.state.terminal_sessions.get_mut(idx) {
                        session.output.push_str(&text);
                        if session.output.len() > 10_000 {
                            let mut start = session.output.len() - 8_000;
                            // Ensure we don't split a UTF-8 character
                            while start < session.output.len() && !session.output.is_char_boundary(start) {
                                start += 1;
                            }
                            session.output = session.output[start..].to_string();
                        }
                    }
                }
            }
        }
        // Sync key state → store after processing messages
        if count > 0 {
            self.store.update(cx, |s, cx| {
                if s.playground_response != self.state.playground_response {
                    s.playground_response = self.state.playground_response.clone();
                    s.playground_loading = self.state.playground_loading;
                    cx.emit(crate::store::StoreEvent::PlaygroundUpdated);
                }
                if s.save_status != self.state.save_status {
                    s.save_status = self.state.save_status;
                    cx.emit(crate::store::StoreEvent::SaveStatusChanged);
                }
                if s.session.is_some() != self.state.session.is_some() {
                    s.session = self.state.session.clone();
                    s.screen = self.state.screen;
                    cx.emit(crate::store::StoreEvent::SessionChanged);
                }
                // Sync project blocks for SDD/import results
                if s.project.blocks.len() != self.state.project.blocks.len() || self.state.prompt_dirty {
                    s.project.blocks = self.state.project.blocks.iter().map(|b| {
                        Block { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled, editing: false }
                    }).collect();
                    s.prompt_dirty = true;
                    s.refresh_cache();
                    self.state.prompt_dirty = false;
                    cx.emit(crate::store::StoreEvent::PromptCacheUpdated);
                    cx.emit(crate::store::StoreEvent::ProjectChanged);
                }
            });
        }
    }
}

impl InkwellApp {
    fn render_auth(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        // Initialize input entities
        if self.state.server_url_input.is_none() {
            self.state.server_url_input = Some(cx.new(|cx| {
                InputState::new(window, cx).default_value("http://localhost:8910")
            }));
        }
        if self.state.email_input.is_none() {
            self.state.email_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("email@example.com")
            }));
        }
        if self.state.password_input.is_none() {
            self.state.password_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Password").masked(true)
            }));
        }

        let (Some(server_input), Some(email_input), Some(password_input)) = (
            self.state.server_url_input.clone(),
            self.state.email_input.clone(),
            self.state.password_input.clone(),
        ) else {
            return div().size_full().bg(bg_primary());
        };

        let is_login = self.state.auth_mode == AuthMode::Login;
        let lang = self.state.lang.clone();
        let is_fr = lang == "fr";

        div()
            .size_full().bg(bg_primary()).flex().flex_col()
            // Top bar: theme + lang toggles (matching web)
            .child(div().flex().justify_end().px(px(16.0)).py(px(8.0)).gap(px(8.0))
                .child(div().px(px(8.0)).py(px(4.0)).rounded(px(6.0)).bg(bg_tertiary())
                    .flex().items_center().gap(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(if self.state.dark_mode { IconName::Moon } else { IconName::Sun }).text_color(text_muted()))
                    .child(Icon::new(IconName::ChevronDown).text_color(text_muted()))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                        this.state.dark_mode = !this.state.dark_mode;
                        set_dark_mode(this.state.dark_mode);
                        Theme::change(if this.state.dark_mode { ThemeMode::Dark } else { ThemeMode::Light }, Some(window), cx);
                    })))
                .child(div().px(px(8.0)).py(px(4.0)).rounded(px(6.0)).bg(bg_tertiary())
                    .flex().items_center().gap(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(IconName::Globe).text_color(text_muted()))
                    .child(div().text_xs().text_color(text_secondary()).child(if is_fr { "FR" } else { "EN" }))
                    .child(Icon::new(IconName::ChevronDown).text_color(text_muted()))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _cx| {
                        this.state.lang = if this.state.lang == "fr" { "en".into() } else { "fr".into() };
                    }))))
            // Centered auth form
            .child(div().flex_1().flex().items_center().justify_center()
                .child(div().w(px(420.0)).flex().flex_col().gap(px(20.0))
                    // Logo + title (matching web)
                    .child(div().flex().flex_col().items_center().gap(px(8.0))
                        .child(div().w(px(56.0)).h(px(56.0)).rounded(px(16.0)).bg(bg_tertiary())
                            .flex().items_center().justify_center()
                            .child(div().text_xl().text_color(accent()).child("I")))
                        .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(text_primary())
                            .child(if is_fr { "Bienvenue sur Inkwell" } else { "Welcome to Inkwell" }))
                        .child(div().text_sm().text_color(text_muted())
                            .child(if is_fr { "Votre atelier de creation de prompts IA" } else { "Your AI prompt creation workshop" })))
                    // Connexion / Inscription tabs (matching web exactly)
                    .child(div().flex().rounded(px(8.0)).bg(bg_tertiary()).p(px(2.0))
                        .child(div().flex_1().py(px(8.0)).rounded(px(6.0))
                            .bg(if is_login { accent() } else { transparent() })
                            .text_sm().text_color(if is_login { ink_white() } else { text_secondary() })
                            .flex().items_center().justify_center()
                            .child(if is_fr { "Connexion" } else { "Sign in" })
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.auth_mode = AuthMode::Login; })))
                        .child(div().flex_1().py(px(8.0)).rounded(px(6.0))
                            .bg(if !is_login { accent() } else { transparent() })
                            .text_sm().text_color(if !is_login { ink_white() } else { text_secondary() })
                            .flex().items_center().justify_center()
                            .child(if is_fr { "Inscription" } else { "Sign up" })
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.auth_mode = AuthMode::Register; }))))
                    // Form fields (matching web: label + icon-prefixed input)
                    .child(div().flex().flex_col().gap(px(12.0))
                        // Server URL
                        .child(div().flex().flex_col().gap(px(4.0))
                            .child(div().text_xs().text_color(text_muted()).child(if is_fr { "Serveur" } else { "Server" }))
                            .child(Input::new(&server_input)))
                        // Email
                        .child(div().flex().flex_col().gap(px(4.0))
                            .child(div().text_xs().text_color(text_muted()).child("Email"))
                            .child(Input::new(&email_input)))
                        // Password
                        .child(div().flex().flex_col().gap(px(4.0))
                            .child(div().text_xs().text_color(text_muted()).child(if is_fr { "Mot de passe" } else { "Password" }))
                            .child(Input::new(&password_input))))
                    // Error message
                    .children(self.state.auth_error.clone().map(|e| {
                        div().px(px(12.0)).py(px(8.0)).rounded(px(8.0))
                            .bg(hsla(0.0, 0.75, 0.5, 0.1))
                            .flex().items_center().gap(px(6.0))
                            .child(Icon::new(IconName::TriangleAlert).text_color(danger()))
                            .child(div().text_xs().text_color(danger()).child(e))
                    }))
                    // Submit button (matching web)
                    .child(div().py(px(10.0)).bg(if self.state.auth_loading { text_muted() } else { accent() }).rounded(px(8.0))
                        .flex().items_center().justify_center()
                        .text_sm().text_color(ink_white())
                        .child(if self.state.auth_loading { if is_fr { "Connexion..." } else { "Connecting..." } }
                            else if is_login { if is_fr { "Se connecter" } else { "Sign in" } }
                            else { if is_fr { "S'inscrire" } else { "Sign up" } })
                        .cursor_pointer().hover(|s| s.bg(accent_hover()))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                if this.state.auth_loading { return; }
                                this.state.auth_error = None;

                                let server_url = this.state.server_url_input.as_ref()
                                    .map(|i| i.read(cx).value().to_string())
                                    .unwrap_or_else(|| this.state.server_url.clone());
                                let email = this.state.email_input.as_ref()
                                    .map(|i| i.read(cx).value().to_string())
                                    .unwrap_or_default();
                                let password = this.state.password_input.as_ref()
                                    .map(|i| i.read(cx).value().to_string())
                                    .unwrap_or_default();
                                // Validate inputs
                                if email.trim().is_empty() || password.trim().is_empty() {
                                    this.state.auth_error = Some("Email et mot de passe requis".into());
                                    return;
                                }
                                this.state.auth_loading = true;
                                let tx = this.state.msg_tx.clone();
                                let is_register = this.state.auth_mode == AuthMode::Register;
                                let display_name = email.split('@').next().unwrap_or("User").to_string();

                                rt().spawn(async move {
                                        let mut client = inkwell_core::api_client::ApiClient::new(&server_url);
                                        let result = if is_register {
                                            client.register(&email, &password, &display_name).await
                                        } else {
                                            client.login(&email, &password).await
                                        };
                                        match result {
                                            Ok(session) => {
                                                client.set_token(session.token.clone());
                                                let projects = client.list_projects().await.unwrap_or_default();
                                                let workspaces = client.list_workspaces().await.unwrap_or_default();
                                                let _ = tx.send(AsyncMsg::AuthSuccess { session, projects, workspaces });
                                            }
                                            Err(e) => { let _ = tx.send(AsyncMsg::AuthError(e)); }
                                        }
                                    });
                            }))
                    )
                    // "ou" divider (matching web)
                    .child(div().flex().items_center().gap(px(12.0))
                        .child(div().flex_1().h(px(1.0)).bg(border_c()))
                        .child(div().text_xs().text_color(text_muted()).child("ou"))
                        .child(div().flex_1().h(px(1.0)).bg(border_c())))
                    // OAuth buttons (matching web: Google + GitHub)
                    .child(div().flex().flex_col().gap(px(8.0))
                        .child(div().py(px(10.0)).rounded(px(8.0)).border_1().border_color(border_c()).bg(bg_secondary())
                            .flex().items_center().justify_center().gap(px(8.0))
                            .text_sm().text_color(text_primary())
                            .child(div().text_sm().text_color(text_secondary()).child("G"))
                            .child(if is_fr { "Continuer avec Google" } else { "Continue with Google" })
                            .cursor_pointer().hover(|s| s.bg(bg_hover())))
                        .child(div().py(px(10.0)).rounded(px(8.0)).border_1().border_color(border_c()).bg(bg_secondary())
                            .flex().items_center().justify_center().gap(px(8.0))
                            .text_sm().text_color(text_primary())
                            .child(Icon::new(IconName::Github).text_color(text_secondary()))
                            .child(if is_fr { "Continuer avec GitHub" } else { "Continue with GitHub" })
                            .cursor_pointer().hover(|s| s.bg(bg_hover()))))
                    // Bottom link (matching web: "Pas encore de compte? Inscription")
                    .child(div().flex().items_center().justify_center().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted())
                            .child(if is_fr {
                                if is_login { "Pas encore de compte ?" } else { "Deja un compte ?" }
                            } else {
                                if is_login { "Don't have an account?" } else { "Already have an account?" }
                            }))
                        .child(div().text_xs().text_color(accent()).cursor_pointer()
                            .child(if is_fr {
                                if is_login { "Inscription" } else { "Connexion" }
                            } else {
                                if is_login { "Sign up" } else { "Sign in" }
                            })
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.state.auth_mode = if this.state.auth_mode == AuthMode::Login { AuthMode::Register } else { AuthMode::Login };
                            }))))
                ))
    }

    fn sync_block_content(&mut self, cx: &mut Context<Self>) {
        // Read content from Input widgets — only allocate if value changed
        let mut changed = false;
        for (idx, block) in self.state.project.blocks.iter_mut().enumerate() {
            if let Some(Some(input)) = self.state.block_inputs.get(idx) {
                let val = input.read(cx).value();
                if val != block.content.as_str() {
                    block.content = val.to_string();
                    changed = true;
                }
            }
        }
        // Read variable values — only allocate if changed
        let var_keys: Vec<String> = self.state.variable_inputs.keys().cloned().collect();
        for var_name in var_keys {
            if let Some(entity) = self.state.variable_inputs.get(&var_name) {
                let val = entity.read(cx).value();
                let old = self.state.project.variables.get(&var_name).map(|s| s.as_str()).unwrap_or("");
                if val != old && !val.is_empty() {
                    self.state.project.variables.insert(var_name, val.to_string());
                    changed = true;
                }
            }
        }
        // Refresh prompt cache if dirty
        if changed { self.state.prompt_dirty = true; }
        if self.state.prompt_dirty {
            let core_blocks: Vec<inkwell_core::types::PromptBlock> = self.state.project.blocks.iter().map(|b| {
                inkwell_core::types::PromptBlock { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled }
            }).collect();
            self.state.cached_prompt = inkwell_core::prompt::compile_prompt(&core_blocks, &self.state.project.variables);
            self.state.cached_tokens = (self.state.cached_prompt.len() as f64 / 4.0).ceil() as usize;
            self.state.cached_chars = self.state.cached_prompt.len();
            self.state.cached_words = if self.state.cached_prompt.is_empty() { 0 } else { self.state.cached_prompt.split_whitespace().count() };
            self.state.cached_lines = self.state.cached_prompt.lines().count();
            self.state.cached_vars = inkwell_core::prompt::extract_variables(&core_blocks);
            self.state.prompt_dirty = false;
        }
        // Read search query from input (only allocate if changed)
        if let Some(ref input) = self.state.search_input {
            let val = input.read(cx).value();
            if val != self.state.search_query.as_str() {
                self.state.search_query = val.to_string();
            }
        }
        // Mark save pending if content changed (actual save in periodic timer)
        if changed {
            self.state.save_pending = true;
        }
    }

    fn save_to_backend(&mut self) {
        self.state.save_status = "saving";

        // 1. Save locally FIRST (instant, no network)
        let local_project = crate::persistence::LocalProject {
            id: self.state.project.id.clone(),
            name: self.state.project.name.clone(),
            workspace_id: self.state.project.workspace_id.clone(),
            blocks: self.state.project.blocks.iter().map(|b| {
                inkwell_core::types::PromptBlock {
                    id: b.id.clone(), block_type: b.block_type,
                    content: b.content.clone(), enabled: b.enabled,
                }
            }).collect(),
            variables: self.state.project.variables.clone(),
            tags: self.state.project.tags.clone(),
            framework: self.state.project.framework.clone(),
            updated_at: chrono::Utc::now().timestamp_millis(),
        };
        crate::persistence::save_project(&local_project);

        // Also save custom frameworks locally
        let local_fws: Vec<crate::persistence::LocalFramework> = self.state.custom_frameworks.iter()
            .map(|f| crate::persistence::LocalFramework { name: f.name.clone(), blocks: f.blocks.clone() })
            .collect();
        crate::persistence::save_frameworks(&local_fws);

        // Also save settings locally
        crate::persistence::save_settings(&crate::persistence::LocalSettings {
            api_key_openai: self.state.api_key_openai.clone(),
            api_key_anthropic: self.state.api_key_anthropic.clone(),
            api_key_google: self.state.api_key_google.clone(),
            github_repo: self.state.github_repo.clone(),
            selected_model: self.state.selected_model.clone(),
        });

        // 2. Background sync to server (non-blocking, best-effort)
        let server_url = self.state.server_url.clone();
        let token = self.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
        crate::persistence::sync_project_to_server(&server_url, &token, &local_project);
    }

    fn ensure_terminal_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.terminal_input_entity.is_none() {
            self.state.terminal_input_entity = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Enter command...")
            }));
        }
        if self.state.chat_input_entity.is_none() {
            self.state.chat_input_entity = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Type a message...")
            }));
        }
        if self.state.ssh_host_input.is_none() {
            self.state.ssh_host_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("hostname or IP")
            }));
        }
        if self.state.ssh_user_input.is_none() {
            self.state.ssh_user_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("username")
            }));
        }
        if self.state.tag_input.is_none() {
            self.state.tag_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("tag name")
            }));
        }
        if self.state.version_label_input.is_none() {
            self.state.version_label_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("version label")
            }));
        }
        if self.state.api_key_openai_input.is_none() {
            self.state.api_key_openai_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("sk-...").masked(true)
            }));
        }
        if self.state.api_key_anthropic_input.is_none() {
            self.state.api_key_anthropic_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("sk-ant-...").masked(true)
            }));
        }
        if self.state.api_key_google_input.is_none() {
            self.state.api_key_google_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("AIza...").masked(true)
            }));
        }
        if self.state.ssh_port_input.is_none() {
            self.state.ssh_port_input = Some(cx.new(|cx| {
                InputState::new(window, cx).default_value("22")
            }));
        }
        if self.state.workspace_name_input.is_none() && self.state.editing_workspace_id.is_some() {
            let name = self.state.workspaces.iter()
                .find(|w| Some(w.id.as_str()) == self.state.editing_workspace_id.as_deref())
                .map(|w| w.name.clone()).unwrap_or_default();
            self.state.workspace_name_input = Some(cx.new(|cx| {
                InputState::new(window, cx).default_value(name)
            }));
        }
        if self.state.search_input.is_none() {
            self.state.search_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Rechercher...")
            }));
        }
        if self.state.name_input_entity.is_none() {
            let name = self.state.project.name.clone();
            self.state.name_input_entity = Some(cx.new(|cx| {
                InputState::new(window, cx).default_value(name)
            }));
        }
        if self.state.framework_name_input.is_none() {
            self.state.framework_name_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Framework name")
            }));
        }
        if self.state.github_repo_input.is_none() {
            self.state.github_repo_input = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("owner/repo")
            }));
        }
    }

    fn ensure_block_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Ensure we have an InputState for each block
        while self.state.block_inputs.len() < self.state.project.blocks.len() {
            let idx = self.state.block_inputs.len();
            let content = self.state.project.blocks.get(idx)
                .map(|b| b.content.clone()).unwrap_or_default();
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(content)
                    .multi_line(true)
                    .auto_grow(3, 20)
            });
            self.state.block_inputs.push(Some(input));
        }
        // Remove excess
        self.state.block_inputs.truncate(self.state.project.blocks.len());

        // Ensure variable input entities — only when vars changed
        let var_count = self.state.cached_vars.len();
        if var_count != self.state.variable_inputs.len() || self.state.variable_inputs.keys().any(|k| !self.state.cached_vars.contains(k)) {
            for var in &self.state.cached_vars.clone() {
                if !self.state.variable_inputs.contains_key(var) {
                    let val = self.state.project.variables.get(var).cloned().unwrap_or_default();
                    let entity = cx.new(|cx| {
                        InputState::new(window, cx)
                            .placeholder(format!("value for {var}"))
                            .default_value(val)
                    });
                    self.state.variable_inputs.insert(var.clone(), entity);
                }
            }
            let cached = self.state.cached_vars.clone();
            self.state.variable_inputs.retain(|k, _| cached.contains(k));
        }
    }

    fn render_ide(&mut self, cx: &mut Context<Self>) -> Div {
        // Read layout state from store (not self.state) to avoid bridge re-renders
        let s = self.store.read(cx);
        let left_open = s.left_open;
        let right_open = s.right_open;
        let show_settings = s.show_settings;
        let show_profile = s.show_profile;
        let dark_mode = s.dark_mode;
        drop(s);

        set_dark_mode(dark_mode);
        // Sync gpui-component theme so Input, Button, etc. follow dark/light mode
        Theme::change(if dark_mode { ThemeMode::Dark } else { ThemeMode::Light }, None, cx);
        let t = crate::theme::InkwellTheme::from_mode(dark_mode);
        let mut main_row = div().flex_1().flex().overflow_hidden();
        let left_w = self.store.read(cx).left_width;
        let right_w = self.store.read(cx).right_width;
        if left_open {
            main_row = main_row.child(self.left_panel.clone());
            // Left resize handle — drag to resize
            main_row = main_row.child(
                div().id("left-resize").w(px(4.0)).flex_shrink_0().cursor_pointer()
                    .hover(|s| s.bg(accent()))
                    .on_drag(ResizeDrag { side: ResizeSide::Left, start_x: 0.0, start_width: left_w },
                        |drag, _, _, cx| cx.new(|_| drag.clone()))
                    .on_drag_move(cx.listener(|this, ev: &DragMoveEvent<ResizeDrag>, _, cx| {
                        let new_w = f32::from(ev.event.position.x).clamp(180.0, 500.0);
                        this.store.update(cx, |s, _| { s.left_width = new_w; });
                        cx.notify();
                    }))
            );
        }
        main_row = main_row.child(self.editor.clone());
        if right_open {
            // Right resize handle — drag to resize
            main_row = main_row.child(
                div().id("right-resize").w(px(4.0)).flex_shrink_0().cursor_pointer()
                    .hover(|s| s.bg(accent()))
                    .on_drag(ResizeDrag { side: ResizeSide::Right, start_x: 0.0, start_width: right_w },
                        |drag, _, _, cx| cx.new(|_| drag.clone()))
                    .on_drag_move(cx.listener(|this, ev: &DragMoveEvent<ResizeDrag>, window, cx| {
                        // Right panel width = window_width - mouse_x
                        let mouse_x = f32::from(ev.event.position.x);
                        let win_w = f32::from(window.viewport_size().width);
                        let new_w = (win_w - mouse_x).clamp(250.0, 600.0);
                        this.store.update(cx, |s, _| { s.right_width = new_w; });
                        cx.notify();
                    }))
            );
            main_row = main_row.child(self.right_panel.clone());
        }

        div().size_full().bg(t.bg_primary).flex().flex_col()
            .on_action(cx.listener(|this, _: &NewProject, _, _| {
                let mut p = Project::default_prompt();
                p.name = "Nouveau Prompte".into();
                let now = chrono::Local::now();
                p.tags.push(now.format("%Y-%m-%d %H:%M").to_string());
                this.state.project = p;
                this.state.block_inputs.clear();
            }))
            .on_action(cx.listener(|this, _: &ToggleTerminal, _, cx| {
                this.state.right_tab = RightTab::Terminal;
                this.state.right_open = true;
                this.store.update(cx, |s, cx| { s.right_tab = RightTab::Terminal; s.right_open = true; cx.emit(crate::store::StoreEvent::SwitchRightTab(RightTab::Terminal)); });
            }))
            .on_action(cx.listener(|this, _: &RunPrompt, _, cx| {
                this.state.right_tab = RightTab::Playground;
                this.state.right_open = true;
                this.store.update(cx, |s, cx| { s.right_tab = RightTab::Playground; s.right_open = true; cx.emit(crate::store::StoreEvent::SwitchRightTab(RightTab::Playground)); });
            }))
            .on_action(cx.listener(|this, _: &ToggleSettings, _, cx| {
                this.state.show_settings = !this.state.show_settings;
                this.store.update(cx, |s, cx| { s.show_settings = !s.show_settings; cx.emit(crate::store::StoreEvent::SettingsChanged); });
            }))
            .on_action(cx.listener(|this, _: &Undo, _, _| {
                if let Some(prev_blocks) = this.state.undo_stack.pop_back() {
                    this.state.project.blocks = prev_blocks;
                    this.state.block_inputs.clear();
                }
            }))
            .on_action(cx.listener(|this, _: &SaveNow, _, _| {
                this.state.save_pending = true;
                this.state.save_timer = 1; // Save next frame
            }))
            .child(self.header.clone()) // Entity<HeaderBar> — only re-renders when store emits relevant event
            .child(main_row)
            .children(if show_settings { Some(self.render_settings(cx)) } else { None })
            .children(if show_profile { Some(self.render_profile(cx)) } else { None })
            .child(self.bottom_bar.clone()) // Entity<BottomBar> — only re-renders on PromptCacheUpdated
    }


    fn render_settings(&self, cx: &mut Context<Self>) -> Div {
        let lang = self.state.lang.clone();
        // Modal overlay (matching web: centered card over backdrop)
        div().size_full().absolute().top_0().left_0()
            .bg(hsla(0.0, 0.0, 0.0, 0.4))
            .flex().items_center().justify_center()
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.state.show_settings = false;
                this.store.update(cx, |s, cx| { s.show_settings = false; cx.emit(crate::store::StoreEvent::SettingsChanged); });
            }))
            .child(div().w(px(480.0)).max_h(px(600.0))
                .rounded(px(12.0)).bg(bg_secondary())
                .border_1().border_color(border_c())
                .p(px(24.0)).flex().flex_col().gap(px(16.0))
                .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, _| { /* stop propagation */ }))
            .child(
                div().flex().items_center().gap(px(8.0))
                    .child(div().text_sm().text_color(text_primary()).child(Icon::new(IconName::Settings)))
                    .child(div().flex_1())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                            .text_xs().text_color(text_muted()).child(Icon::new(IconName::Close)).child("Close")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.state.show_settings = false;
                            }))
                    )
            )
            .child(
                div().flex().gap(px(16.0))
                    // Language
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("Language"))
                        .child(div().flex().gap(px(4.0))
                            .child(
                                div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                                    .bg(if lang == "fr" { accent() } else { bg_tertiary() })
                                    .text_xs().text_color(if lang == "fr" { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                                    .child("Francais")
                                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.lang = "fr".into(); }))
                            )
                            .child(
                                div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                                    .bg(if lang == "en" { accent() } else { bg_tertiary() })
                                    .text_xs().text_color(if lang == "en" { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                                    .child("English")
                                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.lang = "en".into(); }))
                            )
                        )
                    )
                    // Server URL
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("Server URL"))
                        .child(div().h(px(28.0)).px(px(8.0)).bg(bg_tertiary()).rounded(px(4.0))
                            .border_1().border_color(border_c())
                            .flex().items_center().text_xs().text_color(text_secondary())
                            .child(self.state.server_url.clone()))
                    )
                    // API Keys
                    .child(div().flex().flex_col().gap(px(6.0))
                        .child(div().text_xs().text_color(text_muted()).child("API Keys"))
                        .child(div().flex().items_center().gap(px(6.0))
                            .child(div().w(px(60.0)).text_xs().text_color(text_muted()).child("OpenAI"))
                            .child({
                                if let Some(ref entity) = self.state.api_key_openai_input {
                                    div().flex_1().child(Input::new(entity))
                                } else {
                                    div().flex_1().text_xs().text_color(text_muted()).child("not set")
                                }
                            })
                        )
                        .child(div().flex().items_center().gap(px(6.0))
                            .child(div().w(px(60.0)).text_xs().text_color(text_muted()).child("Anthropic"))
                            .child({
                                if let Some(ref entity) = self.state.api_key_anthropic_input {
                                    div().flex_1().child(Input::new(entity))
                                } else {
                                    div().flex_1().text_xs().text_color(text_muted()).child("not set")
                                }
                            })
                        )
                        .child(div().flex().items_center().gap(px(6.0))
                            .child(div().w(px(60.0)).text_xs().text_color(text_muted()).child("Google"))
                            .child({
                                if let Some(ref entity) = self.state.api_key_google_input {
                                    div().flex_1().child(Input::new(entity))
                                } else {
                                    div().flex_1().text_xs().text_color(text_muted()).child("not set")
                                }
                            })
                        )
                        .child(
                            div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(accent())
                                .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                                .flex().items_center().justify_center().child("Save keys")
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                    if let Some(ref e) = this.state.api_key_openai_input {
                                        let v = e.read(cx).value().to_string();
                                        if !v.is_empty() { this.state.api_key_openai = v; }
                                    }
                                    if let Some(ref e) = this.state.api_key_anthropic_input {
                                        let v = e.read(cx).value().to_string();
                                        if !v.is_empty() { this.state.api_key_anthropic = v; }
                                    }
                                    if let Some(ref e) = this.state.api_key_google_input {
                                        let v = e.read(cx).value().to_string();
                                        if !v.is_empty() { this.state.api_key_google = v; }
                                    }
                                }))
                        )
                    )
                    // GitHub repo
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("GitHub Repo"))
                        .child({
                            if let Some(ref entity) = self.state.github_repo_input {
                                div().child(Input::new(entity))
                            } else {
                                div().text_xs().text_color(text_muted()).child("owner/repo")
                            }
                        })
                    )
            )
            .child(
                div().flex().gap(px(8.0))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+, settings"))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+N new"))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+` terminal"))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+Enter run"))
                    .child(div().flex_1())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                            .bg(if self.state.session.is_some() { danger() } else { accent() })
                            .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                            .child(if self.state.session.is_some() { "Deconnecter sync" } else { "Connecter sync" })
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                if this.state.session.is_some() {
                                    // Disconnect sync (keep local data)
                                    this.state.session = None;
                                } else {
                                    // Go to auth to connect
                                    this.state.screen = Screen::Auth;
                                }
                                crate::persistence::save_session(&crate::persistence::SavedSession {
                                    server_url: this.state.server_url.clone(),
                                    token: String::new(),
                                    email: String::new(),
                                    dark_mode: this.state.dark_mode,
                                    lang: this.state.lang.clone(),
                                    last_project_id: None,
                                    left_open: this.state.left_open,
                                    right_open: this.state.right_open,
                                });
                            }))
                    )
            ))
    }

    fn render_profile(&self, cx: &mut Context<Self>) -> Div {
        let session = self.state.session.as_ref();
        let email = session.map(|s| s.email.clone()).unwrap_or_default();
        let display_name = session.map(|s| s.display_name.clone()).unwrap_or("User".into());
        let initial = email.chars().next().unwrap_or('U').to_uppercase().to_string();

        div().h(px(180.0)).flex_shrink_0()
            .border_t_1().border_color(border_c()).bg(bg_secondary())
            .p(px(16.0)).flex().flex_col().gap(px(12.0))
            .child(
                div().flex().items_center().gap(px(8.0))
                    .child(div().text_sm().text_color(text_primary()).child(Icon::new(IconName::User)).child("Profile"))
                    .child(div().flex_1())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                            .text_xs().text_color(text_muted()).child(Icon::new(IconName::Close)).child("Close")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.state.show_profile = false;
                            }))
                    )
            )
            .child(
                div().flex().items_center().gap(px(16.0))
                    .child(
                        div().w(px(48.0)).h(px(48.0)).rounded(px(24.0)).bg(accent())
                            .flex().items_center().justify_center()
                            .text_xl().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                            .child(initial)
                    )
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_sm().text_color(text_primary()).child(display_name))
                        .child(div().text_xs().text_color(text_secondary()).child(email))
                        .child(div().text_xs().text_color(success()).child("Connected"))
                    )
            )
            .child(
                div().flex().gap(px(8.0))
                    .child(div().text_xs().text_color(text_muted()).child(format!("Server: {}", self.state.server_url)))
                    .child(div().text_xs().text_color(text_muted()).child(format!("{} projects", self.state.projects.len())))
                    .child(div().text_xs().text_color(text_muted()).child(format!("{} workspaces", self.state.workspaces.len())))
                    .child(div().text_xs().text_color(text_muted()).child(format!("{} executions", self.state.executions.len())))
                )
    }

}
