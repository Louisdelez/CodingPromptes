use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::state::*;
use inkwell_core::types::BlockType;

// Actions for keyboard shortcuts
actions!(inkwell, [NewProject, ToggleTerminal, RunPrompt, ToggleSettings, Undo, SaveNow]);

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
fn llm_post(client: &reqwest::Client, model: &str, server_url: &str, body: serde_json::Value) -> reqwest::RequestBuilder {
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
}

impl InkwellApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (msg_tx, msg_rx) = std::sync::mpsc::channel();
        let store = cx.new(|_cx| crate::store::AppStore::new(msg_tx.clone()));
        let header = cx.new(|cx| crate::components::header_bar::HeaderBar::new(store.clone(), cx));
        let bottom_bar = cx.new(|cx| crate::components::bottom_bar::BottomBar::new(store.clone(), cx));
        let editor = cx.new(|cx| crate::components::editor_pane::EditorPane::new(store.clone(), window, cx));

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

        Self { state, store, header, bottom_bar, editor }
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
                            self.state.projects.push(ProjectSummary { id: sp.id.clone(), name: sp.name.clone() });
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

        div()
            .size_full().bg(bg_primary()).flex().items_center().justify_center()
            .child(
                div().w(px(400.0)).p(px(32.0)).bg(bg_secondary()).rounded(px(16.0))
                    .border_1().border_color(border_c()).flex().flex_col().gap(px(16.0))
                    .child(div().flex().flex_col().items_center().gap(px(8.0))
                        .child(div().text_xl().text_color(text_primary()).child("Inkwell"))
                        .child(div().text_sm().text_color(text_muted()).child("GPU-Accelerated Prompt IDE"))
                    )
                    // Login/Register tabs
                    .child(
                        div().flex().rounded(px(8.0)).bg(bg_tertiary()).p(px(2.0))
                            .child(
                                div().flex_1().py(px(6.0)).rounded(px(6.0))
                                    .bg(if self.state.auth_mode == AuthMode::Login { accent() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                                    .text_xs().text_color(if self.state.auth_mode == AuthMode::Login { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                                    .flex().items_center().justify_center().child("Sign in")
                                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.auth_mode = AuthMode::Login; }))
                            )
                            .child(
                                div().flex_1().py(px(6.0)).rounded(px(6.0))
                                    .bg(if self.state.auth_mode == AuthMode::Register { accent() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                                    .text_xs().text_color(if self.state.auth_mode == AuthMode::Register { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                                    .flex().items_center().justify_center().child("Sign up")
                                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.auth_mode = AuthMode::Register; }))
                            )
                    )
                    // Server URL
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("Server"))
                        .child(Input::new(&server_input))
                    )
                    // Email
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("Email"))
                        .child(Input::new(&email_input))
                    )
                    // Password
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("Password"))
                        .child(Input::new(&password_input))
                    )
                    // Error
                    .children(self.state.auth_error.clone().map(|e| {
                        div().text_xs().text_color(danger()).child(e)
                    }))
                    // Login button
                    .child(
                        div().py(px(10.0)).bg(if self.state.auth_loading { text_muted() } else { accent() }).rounded(px(8.0))
                            .flex().items_center().justify_center()
                            .text_sm().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                            .child(if self.state.auth_loading { "Connecting..." }
                                else if self.state.auth_mode == AuthMode::Register { "Sign up" }
                                else { "Sign in" })
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
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
                    // Skip auth (for dev)
                    .child(
                        div().py(px(6.0)).flex().items_center().justify_center()
                            .text_xs().text_color(text_muted()).child("Skip (offline mode)")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.state.screen = Screen::Ide;
                            }))
                    )
            )
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
        let t = crate::theme::InkwellTheme::from_mode(dark_mode);
        let mut main_row = div().flex_1().flex().overflow_hidden();
        if left_open { main_row = main_row.child(self.render_sidebar(cx)); }
        main_row = main_row.child(self.editor.clone());
        if right_open { main_row = main_row.child(self.render_right_panel(cx)); }

        div().size_full().bg(t.bg_primary).flex().flex_col()
            .on_action(cx.listener(|this, _: &NewProject, _, _| {
                this.state.project = Project::default_prompt();
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

    fn render_header(&self, cx: &mut Context<Self>) -> Div {
        div().h(px(40.0)).px(px(12.0)).flex().items_center().gap(px(8.0))
            .border_b_1().border_color(border_c()).bg(bg_secondary())
            // Toggle left sidebar
            .child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(if self.state.left_open { text_secondary() } else { text_muted() })
                    .child(if self.state.left_open { "[<]" } else { "[>]" })
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.left_open = !this.state.left_open;
                    }))
            )
            .child(div().text_sm().text_color(accent()).child("Inkwell"))
            .child(div().w(px(1.0)).h(px(16.0)).bg(border_c()))
            .child(if self.state.editing_name {
                if let Some(ref entity) = self.state.name_input_entity {
                    div().w(px(180.0)).child(Input::new(entity))
                } else { div() }
            } else {
                div().text_sm().text_color(text_primary())
                    .child(self.state.project.name.clone())
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.editing_name = true;
                        // Reset name input to current name
                        this.state.name_input_entity = None;
                    }))
            })
            .child(if self.state.editing_name {
                div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                    .text_xs().text_color(success()).child(Icon::new(IconName::Check))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        if let Some(ref entity) = this.state.name_input_entity {
                            let new_name = entity.read(cx).value().to_string();
                            if !new_name.trim().is_empty() {
                                this.state.project.name = new_name.trim().to_string();
                                // Update in project list
                                if let Some(p) = this.state.projects.iter_mut().find(|p| p.id == this.state.project.id) {
                                    p.name = this.state.project.name.clone();
                                }
                                this.state.save_pending = true;
                            }
                        }
                        this.state.editing_name = false;
                    }))
            } else { div() })
                    .child(match self.state.save_status {
                        "saving" => div().text_xs().text_color(hsla(50.0 / 360.0, 0.8, 0.5, 1.0)).child("Saving..."),
                        "saved" => div().text_xs().text_color(success()).child("Saved"),
                        _ => div(),
                    })
            .child(div().flex_1())
            // Framework badge
            .children(self.state.project.framework.as_ref().map(|f| {
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0))
                    .bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.1))
                    .text_xs().text_color(accent()).child(f.clone())
            }))
            // Session info (click for profile)
            .children(self.state.session.as_ref().map(|_s| {
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0))
                    .flex().items_center().gap(px(4.0))
                    .child(div().w(px(18.0)).h(px(18.0)).rounded(px(9.0)).bg(accent())
                        .flex().items_center().justify_center()
                        .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                        .child(_s.email.chars().next().unwrap_or('U').to_uppercase().to_string()))
                    .child(div().text_xs().text_color(text_muted()).child(_s.email.clone()))
                    .cursor_pointer().hover(|s| s.bg(bg_tertiary()))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.show_profile = !this.state.show_profile;
                    }))
            }))
            // Lang toggle
            .child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(text_muted())
                    .child(self.state.lang.to_uppercase())
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.lang = if this.state.lang == "fr" { "en".into() } else { "fr".into() };
                    }))
            )
            // Settings
            .child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(if self.state.show_settings { accent() } else { text_muted() })
                    .child(Icon::new(IconName::Settings))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.show_settings = !this.state.show_settings;
                    }))
            )
            // Theme toggle
            .child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(text_muted())
                    .child(Icon::new(if self.state.dark_mode { IconName::Moon } else { IconName::Sun }))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.dark_mode = !this.state.dark_mode;
                    }))
            )
            .child(div().text_xs().text_color(success()).child("GPUI"))
            // Toggle right panel
            .child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(if self.state.right_open { text_secondary() } else { text_muted() })
                    .child(if self.state.right_open { "[>]" } else { "[<]" })
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.right_open = !this.state.right_open;
                    }))
            )
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> Div {
        let is_library = self.state.left_tab == LeftTab::Library;

        div().w(px(250.0)).flex_shrink_0().border_r_1().border_color(border_c()).bg(bg_secondary())
            .flex().flex_col()
            // Tab bar
            .child(
                div().h(px(36.0)).px(px(8.0)).flex().items_center().gap(px(4.0)).border_b_1().border_color(border_c())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                            .text_color(if is_library { accent() } else { text_muted() })
                            .bg(if is_library { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) } else { hsla(0.0, 0.0, 0.0, 0.0) })
                            .child(Icon::new(IconName::FolderOpen)).child("Bibliotheque")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.left_tab = LeftTab::Library; }))
                    )
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                            .text_color(if !is_library { accent() } else { text_muted() })
                            .bg(if !is_library { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) } else { hsla(0.0, 0.0, 0.0, 0.0) })
                            .child(Icon::new(IconName::Frame)).child("Frameworks")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.left_tab = LeftTab::Frameworks; }))
                    )
            )
            // Content
            .child(if is_library { self.render_library(cx) } else { self.render_frameworks(cx) })
    }

    fn render_library(&self, cx: &mut Context<Self>) -> Div {
        let _lang = &self.state.lang;
        let search = &self.state.search_query;
        let mut content = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(4.0));

        // Search bar with real Input widget
        content = content.child({
            if let Some(ref entity) = self.state.search_input {
                div().child(Input::new(entity))
            } else {
                div().h(px(28.0)).px(px(8.0)).bg(bg_tertiary()).rounded(px(6.0))
                    .border_1().border_color(border_c())
                    .flex().items_center()
                    .text_xs().text_color(text_muted())
                    .child("Rechercher...")
            }
        });

        // Workspaces
        content = content.child(
            div().flex().items_center().gap(px(4.0))
                .child(Icon::new(IconName::FolderOpen))
                .child(div().text_xs().text_color(text_muted()).child("Espaces de travail"))
                .child(div().flex_1())
                .child(
                    div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                        .child(Icon::new(IconName::Plus))
                        .text_color(text_muted())
                        .cursor_pointer().hover(|s| s.bg(accent_bg()))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            let name = format!("Workspace {}", this.state.workspaces.len() + 1);
                            let color = this.state.selected_workspace_color.clone();
                            let ws = inkwell_core::types::Workspace {
                                id: uuid::Uuid::new_v4().to_string(),
                                name: name.clone(), description: String::new(), color,
                                constitution: None,
                                created_at: chrono::Utc::now().timestamp_millis(),
                                updated_at: chrono::Utc::now().timestamp_millis(),
                            };
                            this.state.workspaces.push(ws);
                        }))
                )
        );
        // Color picker swatches
        {
            const PALETTE: &[&str] = &["#6366f1","#8b5cf6","#ec4899","#22c55e","#06b6d4","#f97316","#ef4444","#eab308"];
            let mut swatch_row = div().flex().gap(px(3.0)).px(px(4.0));
            for hex in PALETTE {
                let hex_str = hex.to_string();
                let is_selected = self.state.selected_workspace_color == *hex;
                swatch_row = swatch_row.child(
                    div().w(px(14.0)).h(px(14.0)).rounded(px(7.0))
                        .bg(hex_to_hsla(hex))
                        .border_1().border_color(if is_selected { text_primary() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            this.state.selected_workspace_color = hex_str.clone();
                        }))
                );
            }
            content = content.child(swatch_row);
        }
        for ws in &self.state.workspaces {
            let color = hex_to_hsla(&ws.color);
            let ws_id = ws.id.clone();
            let rename_id = ws.id.clone();
            let is_editing = self.state.editing_workspace_id.as_deref() == Some(&ws.id);
            content = content.child(
                div().px(px(8.0)).py(px(6.0)).rounded(px(4.0))
                    .flex().items_center().gap(px(6.0))
                    .hover(|s| s.bg(bg_tertiary()))
                    .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(color))
                    .child(if is_editing {
                        if let Some(ref entity) = self.state.workspace_name_input {
                            div().flex_1().child(Input::new(entity))
                        } else {
                            div().flex_1().text_xs().text_color(text_primary()).child(ws.name.clone())
                        }
                    } else {
                        div().flex_1().text_xs().text_color(text_primary())
                            .child(ws.name.clone())
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                this.state.editing_workspace_id = Some(rename_id.clone());
                                this.state.workspace_name_input = None; // Will recreate next frame
                            }))
                    })
                    .children(if is_editing {
                        Some(div().text_xs().text_color(success()).child(Icon::new(IconName::Check))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                if let Some(ref entity) = this.state.workspace_name_input {
                                    let new_name = entity.read(cx).value().to_string();
                                    if !new_name.trim().is_empty() {
                                        if let Some(w) = this.state.workspaces.iter_mut().find(|w| Some(w.id.as_str()) == this.state.editing_workspace_id.as_deref()) {
                                            w.name = new_name.trim().to_string();
                                        }
                                    }
                                }
                                this.state.editing_workspace_id = None;
                            })))
                    } else { None })
                    .child(
                        div().text_xs().text_color(danger()).child(Icon::new(IconName::Close))
                            .cursor_pointer().hover(|s| s.bg(hsla(0.0, 0.75, 0.55, 0.15)))
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                this.state.workspaces.retain(|w| w.id != ws_id);
                            }))
                    )
            );
            // Show constitution if present
            if let Some(ref constitution) = ws.constitution {
                if !constitution.is_empty() {
                    content = content.child(
                        div().pl(px(22.0)).pr(px(8.0)).py(px(2.0))
                            .text_xs().text_color(text_muted())
                            .child(constitution.chars().take(80).collect::<String>())
                    );
                }
            }
        }
        if !self.state.workspaces.is_empty() {
            content = content.child(div().h(px(1.0)).bg(border_c()));
        }

        // New project button
        content = content.child(
            div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                .bg(bg_tertiary()).text_xs().text_color(accent())
                .flex().items_center().justify_center().child(Icon::new(IconName::Plus)).child("Nouveau prompt")
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                    let new_proj = Project::default_prompt();
                    let name = new_proj.name.clone();
                    let id = new_proj.id.clone();
                    this.state.project = new_proj;
                    this.state.block_inputs.clear();
                    this.state.variable_inputs.clear();
                    this.state.prompt_dirty = true;
                    this.state.projects.push(ProjectSummary { id: id.clone(), name: name.clone() });
                    // Save locally immediately
                    this.state.save_pending = true;
                    this.state.save_timer = 1;
                }))
        );

        // Project list
        let search_lower = search.to_lowercase();
        let filtered: Vec<_> = self.state.projects.iter()
            .filter(|p| search_lower.is_empty() || p.name.to_lowercase().contains(&search_lower))
            .collect();

        for p in &filtered {
            let id = p.id.clone();
            let delete_id = p.id.clone();
            let is_active = self.state.project.id == p.id;
            content = content.child(
                div().px(px(8.0)).py(px(6.0)).rounded(px(4.0))
                    .flex().items_center().gap(px(4.0))
                    .bg(if is_active { bg_tertiary() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                    .hover(|s| s.bg(bg_tertiary()))
                    // Project name
                    .child(
                        div().flex_1().text_xs()
                            .text_color(if is_active { text_primary() } else { text_secondary() })
                            .child(p.name.clone())
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                // Load from local storage (instant, no network)
                                let local_projects = crate::persistence::load_all_projects();
                                if let Some(lp) = local_projects.iter().find(|p| p.id == id) {
                                    this.state.project.id = lp.id.clone();
                                    this.state.project.name = lp.name.clone();
                                    this.state.project.framework = lp.framework.clone();
                                    this.state.project.tags = lp.tags.clone();
                                    this.state.project.variables = lp.variables.clone();
                                    this.state.project.blocks = lp.blocks.iter().map(|b| {
                                        Block { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled, editing: false }
                                    }).collect();
                                    this.state.block_inputs.clear();
                                    this.state.variable_inputs.clear();
                                    this.state.prompt_dirty = true;
                                }
                            }))
                    )
                    // Delete button
                    .child(
                        div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                            .text_xs().text_color(danger()).child("x")
                            .hover(|s| s.bg(hsla(0.0, 0.75, 0.55, 0.15)))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                this.state.confirm_delete = Some(delete_id.clone());
                            }))
                    )
            );
        }

        // Confirm delete dialog
        if let Some(ref del_id) = self.state.confirm_delete {
            let del = del_id.clone();
            content = content.child(
                div().p(px(10.0)).rounded(px(8.0)).bg(hsla(0.0, 0.75, 0.55, 0.1))
                    .border_1().border_color(danger())
                    .flex().flex_col().gap(px(6.0))
                    .child(div().text_xs().text_color(danger()).child("Supprimer ce projet ?"))
                    .child(
                        div().flex().gap(px(6.0))
                            .child(
                                div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(danger())
                                    .text_xs().text_color(white()).child("Supprimer")
                                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                        let id = del.clone();
                                        this.state.projects.retain(|p| p.id != id);
                                        this.state.confirm_delete = None;
                                        // Delete locally
                                        crate::persistence::delete_project(&id);
                                        // Also delete on backend (best-effort)
                                        let server = this.state.server_url.clone();
                                        let token = this.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                                        if !token.is_empty() {
                                            rt().spawn(async move {
                                                let mut client = inkwell_core::api_client::ApiClient::new(&server);
                                                client.set_token(token);
                                                let _ = client.delete_project(&id).await;
                                            });
                                        }
                                    }))
                            )
                            .child(
                                div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(bg_tertiary())
                                    .text_xs().text_color(text_secondary()).child("Annuler")
                                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                        this.state.confirm_delete = None;
                                    }))
                            )
                    )
            );
        }

        if filtered.is_empty() && self.state.projects.is_empty() {
            content = content.child(div().text_xs().text_color(text_muted()).child("Rien ici encore"));
        } else if filtered.is_empty() {
            content = content.child(div().text_xs().text_color(text_muted()).child("Aucun projet correspondant"));
        }

        content
    }

    fn render_frameworks(&self, cx: &mut Context<Self>) -> Div {
        const FRAMEWORKS: &[(&str, &str)] = &[
            ("CO-STAR", "co-star"), ("RISEN", "risen"), ("RACE", "race"),
            ("SDD (Spec-Driven)", "sdd"), ("APE", "ape"), ("STOKE", "stoke"),
        ];
        let mut content = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(4.0));

        // Save current as framework with name input
        content = content.child(
            div().flex().items_center().gap(px(4.0))
                .child({
                    if let Some(ref entity) = self.state.framework_name_input {
                        div().flex_1().child(Input::new(entity))
                    } else {
                        div().flex_1()
                    }
                })
                .child(
                    div().px(px(8.0)).py(px(6.0)).rounded(px(6.0)).bg(accent())
                        .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                        .child(Icon::new(IconName::Save)).child("Save")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            let name = this.state.framework_name_input.as_ref()
                                .map(|i| i.read(cx).value().to_string())
                                .unwrap_or_default();
                            let name = if name.trim().is_empty() {
                                format!("Custom {}", this.state.custom_frameworks.len() + 1)
                            } else { name.trim().to_string() };
                            let blocks: Vec<(BlockType, String)> = this.state.project.blocks.iter()
                                .map(|b| (b.block_type, b.content.clone()))
                                .collect();
                            this.state.custom_frameworks.push(CustomFramework { name, blocks });
                            this.state.framework_name_input = None; // Reset
                        }))
                )
        );
        content = content.child(div().h(px(1.0)).bg(border_c()));
        for &(name, id) in FRAMEWORKS {
            let id_str = id.to_string();
            let is_active = self.state.project.framework.as_deref() == Some(id);
            content = content.child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0))
                    .border_1().border_color(if is_active { accent() } else { border_c() })
                    .bg(if is_active { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) } else { bg_tertiary() })
                    .text_xs().text_color(if is_active { accent() } else { text_secondary() })
                    .child(name.to_string())
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                        this.state.project.framework = Some(id_str.clone());
                        this.apply_framework(&id_str.clone());
                    }))
            );
        }

        // Custom frameworks
        if !self.state.custom_frameworks.is_empty() {
            content = content.child(div().h(px(1.0)).bg(border_c()));
            content = content.child(div().text_xs().text_color(text_muted()).child("Custom"));
            for (fw_idx, fw) in self.state.custom_frameworks.iter().enumerate() {
                let block_count = fw.blocks.len();
                content = content.child(
                    div().px(px(10.0)).py(px(8.0)).rounded(px(6.0))
                        .border_1().border_color(border_c()).bg(bg_tertiary())
                        .flex().items_center().gap(px(6.0))
                        .hover(|s| s.bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.1)))
                        .child(div().flex_1().text_xs().text_color(text_secondary())
                            .child(format!("{} ({} blocks)", fw.name, block_count))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                if let Some(fw) = this.state.custom_frameworks.get(fw_idx) {
                                    this.state.undo_stack.push_back(this.state.project.blocks.clone());
                                    this.state.project.blocks = fw.blocks.iter().map(|(bt, content)| {
                                        let mut b = Block::new(*bt);
                                        b.content = content.clone();
                                        b
                                    }).collect();
                                    this.state.block_inputs.clear();
                                }
                            }))
                        )
                        // Rename button
                        .child(
                            div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                                .text_xs().text_color(accent()).child(Icon::new(IconName::Redo))
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                    if let Some(fw) = this.state.custom_frameworks.get_mut(fw_idx) {
                                        // Simple rename: append " (edited)"
                                        let new_name = if fw.name.ends_with(" (edited)") {
                                            fw.name.trim_end_matches(" (edited)").to_string()
                                        } else {
                                            // Update blocks to match current project
                                            fw.blocks = this.state.project.blocks.iter()
                                                .map(|b| (b.block_type, b.content.clone()))
                                                .collect();
                                            format!("{} (edited)", fw.name)
                                        };
                                        fw.name = new_name;
                                    }
                                }))
                        )
                        // Delete button
                        .child(
                            div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                                .text_xs().text_color(danger()).child(Icon::new(IconName::Close))
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                    if fw_idx < this.state.custom_frameworks.len() {
                                        this.state.custom_frameworks.remove(fw_idx);
                                    }
                                }))
                        )
                );
            }
        }

        content
    }

    fn apply_framework(&mut self, id: &str) {
        // Save undo snapshot before replacing blocks
        self.state.undo_stack.push_back(self.state.project.blocks.clone());
        while self.state.undo_stack.len() > 50 { self.state.undo_stack.pop_front(); }
        self.state.block_inputs.clear();

        let blocks: Vec<(BlockType, &str)> = match id {
            "co-star" => vec![
                (BlockType::Context, "## Contexte\n"), (BlockType::Task, "## Objectif\n"),
                (BlockType::Role, "## Style\n"), (BlockType::Constraints, "## Ton\n"),
                (BlockType::Format, "## Format\n"),
            ],
            "sdd" => vec![
                (BlockType::SddConstitution, "# Project Constitution\n\n## Core Principles\n"),
                (BlockType::SddSpecification, "# Feature Specification\n\n## User Scenarios\n"),
                (BlockType::SddPlan, "# Implementation Plan\n\n## Summary\n"),
                (BlockType::SddTasks, "# Task Breakdown\n\n## Phase 1: Setup\n"),
                (BlockType::SddImplementation, "# Implementation Notes\n"),
            ],
            "risen" => vec![
                (BlockType::Role, "## Role\n"), (BlockType::Task, "## Instructions\n"),
                (BlockType::Format, "## Objectif final\n"), (BlockType::Constraints, "## Restrictions\n"),
            ],
            "ape" => vec![
                (BlockType::Task, "## Action\n"),
                (BlockType::Context, "## Purpose\n"),
                (BlockType::Format, "## Expectation\n"),
            ],
            _ => vec![(BlockType::Role, ""), (BlockType::Context, ""), (BlockType::Task, "")],
        };
        self.state.project.blocks = blocks.into_iter().map(|(bt, c)| {
            let mut b = Block::new(bt); b.content = c.into(); b
        }).collect();
    }

    fn render_editor(&self, cx: &mut Context<Self>) -> Div {
        let has_sdd = self.state.project.blocks.iter().any(|b| b.block_type.is_sdd());

        let mut block_list = div().flex().flex_col().gap(px(12.0));

        // SDD toolbar
        if has_sdd {
            block_list = block_list.child(
                div().p(px(12.0)).rounded(px(8.0))
                    .border_1().border_color(hsla(239.0 / 360.0, 0.84, 0.67, 0.2))
                    .bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.05))
                    .flex().items_center().gap(px(8.0))
                    .child(div().text_xs().text_color(accent()).child("SDD"))
                    .child(div().flex_1().h(px(28.0)).rounded(px(4.0)).border_1().border_color(border_c()).bg(bg_tertiary()))
                    .child(
                        div().px(px(12.0)).py(px(6.0)).rounded(px(4.0))
                            .bg(if self.state.sdd_running { text_muted() } else { accent() })
                            .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                            .child(if self.state.sdd_running { "Running..." } else { "Generate all" })
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                if this.state.sdd_running { return; }
                                this.state.sdd_running = true;

                                let server = this.state.server_url.clone();
                                let tx = this.state.msg_tx.clone();
                                let _self_model_str = this.state.selected_model.clone();
                                let blocks: Vec<(usize, BlockType)> = this.state.project.blocks.iter().enumerate()
                                    .filter(|(_, b)| b.block_type.is_sdd() && b.enabled)
                                    .map(|(i, b)| (i, b.block_type))
                                    .collect();

                                rt().spawn(async move {
                                        let client = reqwest::Client::new();
                                        let mut context = String::new();

                                        for (idx, bt) in &blocks {
                                            let prompt = if context.is_empty() {
                                                format!("Generate the {:?} for a new software project. Use Spec Kit SDD format.", bt)
                                            } else {
                                                format!("Based on:\n{}\n\nGenerate the {:?} phase.", context, bt)
                                            };

                                            if let Ok(resp) = llm_post(&client, "gpt-4o-mini", &server, serde_json::json!({                                                    "model": "gpt-4o-mini",
                                                    "messages": [
                                                        {"role": "system", "content": "You are an expert software architect. Write in Spec Kit SDD format."},
                                                        {"role": "user", "content": prompt}
                                                    ],
                                                    "temperature": 0.3, "max_tokens": 4096, "stream": false,
                                                })).send().await {
                                                if let Ok(data) = resp.json::<serde_json::Value>().await {
                                                    let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                                    context.push_str(&format!("\n### {:?}\n{}\n", bt, text));
                                                    let _ = tx.send(AsyncMsg::SddBlockResult { idx: *idx, content: text });
                                                }
                                            }
                                        }
                                        let _ = tx.send(AsyncMsg::LlmDone);
                                    });
                            }))
                    )
                    .child(
                        div().px(px(8.0)).py(px(6.0)).text_xs().text_color(text_muted()).child(Icon::new(IconName::Check)).child("Validate")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                let blocks: Vec<inkwell_core::types::PromptBlock> = this.state.project.blocks.iter().map(|b| {
                                    inkwell_core::types::PromptBlock { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled }
                                }).collect();
                                let server = this.state.server_url.clone();
                                let tx = this.state.msg_tx.clone();
                                let mut all_content = String::new();
                                for b in &blocks { if b.enabled { all_content.push_str(&format!("\n### {:?}\n{}\n", b.block_type, b.content)); } }
                                rt().spawn(async move {
                                        let client = reqwest::Client::new();
                                        if let Ok(resp) = llm_post(&client, "gpt-4o-mini", &server, serde_json::json!({"model":"gpt-4o-mini","messages":[                                                {"role":"system","content":"Analyze cross-phase consistency. Check: coverage gaps, contradictions, underspecification, constitution alignment. Output: COHERENT/MISSING/CONTRADICTION/RECOMMENDATION items."},
                                                {"role":"user","content":format!("Validate these SDD phases:\n{all_content}")}
                                            ],"temperature":0.3,"max_tokens":4096,"stream":false})).send().await {
                                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                                let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                                let _ = tx.send(AsyncMsg::LlmResponse(format!("--- Validation ---\n{text}")));
                                            }
                                        }
                                    });
                                this.state.right_tab = RightTab::Playground;
                                this.state.right_open = true;
                            }))
                    )
                    // Checklist
                    .child(
                        div().px(px(8.0)).py(px(6.0)).text_xs().text_color(hsla(50.0 / 360.0, 0.8, 0.5, 1.0)).child(Icon::new(IconName::Check)).child("Checklist")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                let mut all_content = String::new();
                                for b in &this.state.project.blocks { if b.enabled { all_content.push_str(&format!("{}\n\n", b.content)); } }
                                let server = this.state.server_url.clone();
                                let tx = this.state.msg_tx.clone();
                                rt().spawn(async move {
                                        let client = reqwest::Client::new();
                                        if let Ok(resp) = llm_post(&client, "gpt-4o-mini", &server, serde_json::json!({"model":"gpt-4o-mini","messages":[                                                {"role":"system","content":"Generate a quality checklist (Unit Tests for English). Format: - [ ] CHK001 [Quality Dimension] Question. Validate requirements quality, not implementation."},
                                                {"role":"user","content":format!("Generate checklist for:\n{all_content}")}
                                            ],"temperature":0.3,"max_tokens":4096,"stream":false})).send().await {
                                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                                let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                                let _ = tx.send(AsyncMsg::LlmResponse(format!("--- Checklist ---\n{text}")));
                                            }
                                        }
                                    });
                                this.state.right_tab = RightTab::Playground;
                                this.state.right_open = true;
                            }))
                    )
                    // Git
                    .child(
                        div().px(px(6.0)).py(px(4.0)).rounded(px(4.0))
                            .text_xs().text_color(success())
                            .flex().items_center().gap(px(4.0))
                            .child(Icon::new(IconName::GitBranch))
                            .child("Branch")
                            .cursor_pointer().hover(|s| s.bg(hsla(0.0, 0.0, 0.5, 0.08)))
                    )
                    .child(
                        div().px(px(6.0)).py(px(4.0)).rounded(px(4.0))
                            .text_xs().text_color(success())
                            .flex().items_center().gap(px(4.0))
                            .child(Icon::new(IconName::Save))
                            .child("Commit")
                            .cursor_pointer().hover(|s| s.bg(hsla(0.0, 0.0, 0.5, 0.08)))
                    )
                    // GitHub Issues
                    .child(
                        div().px(px(6.0)).py(px(4.0)).rounded(px(4.0))
                            .text_xs().text_color(hsla(280.0 / 360.0, 0.7, 0.6, 1.0))
                            .flex().items_center().gap(px(4.0))
                            .child(Icon::new(IconName::Globe))
                            .child("GitHub")
                            .cursor_pointer().hover(|s| s.bg(hsla(0.0, 0.0, 0.5, 0.08)))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                // Extract tasks from SDD tasks block and push to GitHub Issues
                                let tasks_content = this.state.project.blocks.iter()
                                    .find(|b| b.block_type == BlockType::SddTasks && b.enabled)
                                    .map(|b| b.content.clone())
                                    .unwrap_or_default();
                                if tasks_content.is_empty() { return; }
                                let repo = this.state.github_repo_input.as_ref()
                                    .map(|i| i.read(cx).value().to_string())
                                    .unwrap_or_else(|| this.state.github_repo.clone());
                                if repo.is_empty() {
                                    this.state.playground_response = "Set a GitHub repo (owner/repo) in settings first".into();
                                    return;
                                }
                                let tx = this.state.msg_tx.clone();
                                std::thread::spawn(move || {
                                    // Parse task lines (lines starting with - [ ] or - [x])
                                    let tasks: Vec<String> = tasks_content.lines()
                                        .filter(|l| l.trim().starts_with("- [") || l.trim().starts_with("* ["))
                                        .map(|l| l.trim().trim_start_matches("- [ ] ").trim_start_matches("- [x] ")
                                            .trim_start_matches("* [ ] ").trim_start_matches("* [x] ").to_string())
                                        .collect();
                                    rt().block_on(async {
                                        let client = reqwest::Client::new();
                                        let mut created = 0;
                                        for task in &tasks {
                                            let res = client.post(format!("https://api.github.com/repos/{repo}/issues"))
                                                .header("Accept", "application/vnd.github+json")
                                                .header("User-Agent", "inkwell-gpui")
                                                .json(&serde_json::json!({"title": task, "labels": ["sdd"]}))
                                                .send().await;
                                            if let Ok(r) = res {
                                                if r.status().is_success() { created += 1; }
                                            }
                                        }
                                        let _ = tx.send(AsyncMsg::GitHubPushed(
                                            format!("Created {created}/{} GitHub issues on {repo}", tasks.len())
                                        ));
                                    });
                                });
                            }))
                    )
                    // Presets (functional)
                    .child({
                        let presets = vec![
                            ("React", "TypeScript, Next.js 15, Tailwind CSS 4, React hooks"),
                            ("Rust", "Rust stable, Axum 0.8, Tokio, SQLite, serde"),
                            ("Python", "Python 3.12+, FastAPI, SQLAlchemy 2.0, Pydantic v2"),
                        ];
                        let mut row = div().flex().gap(px(2.0));
                        for (name, stack) in presets {
                            let stack_str = stack.to_string();
                            row = row.child(
                                div().px(px(4.0)).py(px(4.0)).rounded(px(3.0)).bg(bg_tertiary())
                                    .text_xs().text_color(text_muted()).child(name.to_string())
                                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                        // Inject tech stack into constitution block
                                        if let Some(block) = this.state.project.blocks.iter_mut()
                                            .find(|b| b.block_type == BlockType::SddConstitution) {
                                            if !block.content.contains("Technical Stack") {
                                                block.content.push_str(&format!("\n\n## Technical Stack\n{}\n", stack_str));
                                            }
                                        }
                                        // Reset block inputs to reflect changes
                                        this.state.block_inputs.clear();
                                    }))
                            );
                        }
                        row
                    })
            );
        }

        // Blocks
        for (idx, block) in self.state.project.blocks.iter().enumerate() {
            let color = hex_to_hsla(block.block_type.color());
            let label = block.block_type.label(&self.state.lang).to_string();
            let content = if block.content.is_empty() { "Click to edit..." } else { &block.content };
            let is_sdd = block.block_type.is_sdd();

            let mut header = div().px(px(12.0)).py(px(8.0)).flex().items_center().gap(px(8.0))
                .border_b_1().border_color(border_c())
                .child(div().w(px(3.0)).h(px(14.0)).rounded(px(2.0)).bg(color))
                .child(div().text_sm().text_color(color).child(label))
                .child(div().flex_1());

            if is_sdd {
                let _block_type_str = format!("{:?}", block.block_type);
                let all_blocks: Vec<inkwell_core::types::PromptBlock> = self.state.project.blocks.iter().map(|b| {
                    inkwell_core::types::PromptBlock {
                        id: b.id.clone(), block_type: b.block_type,
                        content: b.content.clone(), enabled: b.enabled,
                    }
                }).collect();

                // Generate button
                let tx1 = self.state.msg_tx.clone();
                let server1 = self.state.server_url.clone();
                let blocks1 = all_blocks.clone();
                let bt1 = block.block_type;
                let idx1 = idx;
                header = header.child(
                    div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(accent()).child(Icon::new(IconName::Wand2))
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |_this, _, _, _| {
                            let tx = tx1.clone();
                            let server = server1.clone();
                            let blocks = blocks1.clone();
                            let bt = bt1;
                            let idx = idx1;
                            rt().spawn(async move {
                                    // Build context from previous phases
                                    let mut context = String::new();
                                    let phase_order = [
                                        BlockType::SddConstitution, BlockType::SddSpecification,
                                        BlockType::SddPlan, BlockType::SddTasks, BlockType::SddImplementation,
                                    ];
                                    for phase in &phase_order {
                                        if *phase == bt { break; }
                                        if let Some(b) = blocks.iter().find(|b| b.block_type == *phase && b.enabled) {
                                            if !b.content.is_empty() {
                                                context.push_str(&format!("\n### {:?}\n{}\n", phase, b.content));
                                            }
                                        }
                                    }
                                    let prompt = if context.is_empty() {
                                        format!("Generate the content for the {:?} phase. Use the Spec Kit SDD format.", bt)
                                    } else {
                                        format!("Based on these previous phases:\n{}\n\nGenerate the content for the {:?} phase.", context, bt)
                                    };

                                    let client = reqwest::Client::new();
                                    if let Ok(resp) = llm_post(&client, "gpt-4o-mini", &server, serde_json::json!({                                            "model": "gpt-4o-mini",
                                            "messages": [
                                                {"role": "system", "content": "You are an expert software architect writing SDD specifications."},
                                                {"role": "user", "content": prompt}
                                            ],
                                            "temperature": 0.3, "max_tokens": 4096, "stream": false,
                                        })).send().await {
                                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                                            let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                            let _ = tx.send(AsyncMsg::SddBlockResult { idx, content: text });
                                        }
                                    }
                                });
                        }))
                );

                // Improve button
                let tx2 = self.state.msg_tx.clone();
                let server2 = self.state.server_url.clone();
                let blocks2 = all_blocks.clone();
                let bt2 = block.block_type;
                let idx2 = idx;
                let current_content = block.content.clone();
                header = header.child(
                    div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(hsla(280.0 / 360.0, 0.7, 0.6, 1.0)).child(Icon::new(IconName::Sparkles))
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |_this, _, _, _| {
                            let tx = tx2.clone();
                            let server = server2.clone();
                            let content = current_content.clone();
                            let blocks = blocks2.clone();
                            let bt = bt2;
                            let idx = idx2;
                            rt().spawn(async move {
                                    let mut context = String::new();
                                    let phase_order = [BlockType::SddConstitution, BlockType::SddSpecification, BlockType::SddPlan, BlockType::SddTasks, BlockType::SddImplementation];
                                    for phase in &phase_order {
                                        if *phase == bt { break; }
                                        if let Some(b) = blocks.iter().find(|b| b.block_type == *phase && b.enabled) {
                                            if !b.content.is_empty() { context.push_str(&format!("\n{}\n", b.content)); }
                                        }
                                    }
                                    let prompt = format!("Improve the following content. Make it more precise, complete, and well-structured. Keep the same format.\n\nContext:\n{context}\n\nContent to improve:\n{content}");
                                    let client = reqwest::Client::new();
                                    if let Ok(resp) = llm_post(&client, "gpt-4o-mini", &server, serde_json::json!({"model":"gpt-4o-mini","messages":[{"role":"system","content":"You improve SDD specifications. Keep the format strict."},{"role":"user","content":prompt}],"temperature":0.3,"max_tokens":4096,"stream":false}))                                        .send().await {
                                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                                            let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                            let _ = tx.send(AsyncMsg::SddBlockResult { idx, content: text });
                                        }
                                    }
                                });
                        }))
                );

                // Clarify button
                let tx3 = self.state.msg_tx.clone();
                let server3 = self.state.server_url.clone();
                let content3 = block.content.clone();
                header = header.child(
                    div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(hsla(50.0 / 360.0, 0.8, 0.5, 1.0)).child(Icon::new(IconName::CircleHelp))
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |_this, _, _, _| {
                            let tx = tx3.clone();
                            let server = server3.clone();
                            let content = content3.clone();
                            rt().spawn(async move {
                                    let prompt = format!("Analyze this specification and identify underspecified, ambiguous, or missing areas. Ask max 5 precise questions.\n\nContent:\n{content}");
                                    let client = reqwest::Client::new();
                                    if let Ok(resp) = llm_post(&client, "gpt-4o-mini", &server, serde_json::json!({"model":"gpt-4o-mini","messages":[{"role":"system","content":"You are a technical reviewer. Identify underspecified areas."},{"role":"user","content":prompt}],"temperature":0.5,"max_tokens":2048,"stream":false}))                                        .send().await {
                                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                                            let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                            let _ = tx.send(AsyncMsg::LlmResponse(format!("--- Clarify ---\n{text}")));
                                        }
                                    }
                                });
                        }))
                );
            }

            let block_count = self.state.project.blocks.len();
            let is_recording_this = self.state.stt_recording && self.state.stt_target_block == Some(idx);
            header = header
                // Mic (STT)
                .child(
                    div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(if is_recording_this { danger() } else { text_muted() })
                        .child(Icon::new(if is_recording_this { IconName::Circle } else { IconName::Mic }))
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            if this.state.stt_recording {
                                // Stop any existing recording first
                                if let Some(stop_tx) = this.state.stt_stop_tx.take() {
                                    let _ = stop_tx.send(());
                                }
                                this.state.stt_recording = false;
                            } else {
                                // Stop any orphaned recording before starting new
                                if let Some(old_tx) = this.state.stt_stop_tx.take() {
                                    let _ = old_tx.send(());
                                }
                                this.state.stt_recording = true;
                                this.state.stt_target_block = Some(idx);
                                let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
                                this.state.stt_stop_tx = Some(stop_tx);
                                let tx = this.state.msg_tx.clone();
                                let server = this.state.server_url.clone();
                                let stt_provider = this.state.stt_provider;

                                std::thread::spawn(move || {
                                    // Record audio via cpal
                                    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
                                    let host = cpal::default_host();
                                    let device = match host.default_input_device() {
                                        Some(d) => d,
                                        None => { let _ = tx.send(AsyncMsg::SttError("No microphone found".into())); return; }
                                    };
                                    let config = cpal::StreamConfig { channels: 1, sample_rate: cpal::SampleRate(16000), buffer_size: cpal::BufferSize::Default };
                                    let samples = std::sync::Arc::new(std::sync::Mutex::new(Vec::<f32>::new()));
                                    let samples_clone = samples.clone();

                                    let stream = device.build_input_stream(
                                        &config,
                                        move |data: &[f32], _| { samples_clone.lock().unwrap().extend_from_slice(data); },
                                        |err| { eprintln!("Audio error: {err}"); },
                                        None,
                                    );

                                    match stream {
                                        Ok(s) => {
                                            let _ = s.play();
                                            // Wait for stop signal (max 30s)
                                            let _ = stop_rx.recv_timeout(std::time::Duration::from_secs(30));
                                            drop(s);

                                            // Encode to WAV
                                            let pcm = samples.lock().unwrap();
                                            if pcm.is_empty() { return; }
                                            let mut wav_buf = Vec::new();
                                            {
                                                let cursor = std::io::Cursor::new(&mut wav_buf);
                                                let spec = hound::WavSpec { channels: 1, sample_rate: 16000, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
                                                let mut writer = hound::WavWriter::new(cursor, spec).unwrap();
                                                for &s in pcm.iter() {
                                                    let val = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
                                                    writer.write_sample(val).unwrap();
                                                }
                                                writer.finalize().unwrap();
                                            }
                                            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &wav_buf);

                                            // Send to STT (routes via API keys or server fallback)
                                            rt().block_on(async {
                                                let settings = crate::persistence::load_settings();
                                                let (stt_url, stt_hdrs) = crate::llm::stt_endpoint(&stt_provider, &settings.api_key_openai, &server);
                                                let client = reqwest::Client::new();

                                                let resp = if stt_url.contains("openai.com") || stt_url.contains("groq.com") {
                                                    // OpenAI Whisper / Groq API — multipart form
                                                    let wav_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &b64).unwrap_or_default();
                                                    let part = reqwest::multipart::Part::bytes(wav_bytes).file_name("audio.wav").mime_str("audio/wav").unwrap();
                                                    let form = reqwest::multipart::Form::new()
                                                        .part("file", part)
                                                        .text("model", "whisper-1");
                                                    let mut req = client.post(&stt_url).multipart(form);
                                                    for (k, v) in &stt_hdrs { req = req.header(k.as_str(), v.as_str()); }
                                                    req.send().await
                                                } else {
                                                    // Local server — JSON with base64
                                                    let mut req = client.post(&stt_url)
                                                        .json(&serde_json::json!({"audio": b64, "language": "auto"}));
                                                    for (k, v) in &stt_hdrs { req = req.header(k.as_str(), v.as_str()); }
                                                    req.send().await
                                                };

                                                if let Ok(resp) = resp {
                                                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                                                        let text = data["text"].as_str().unwrap_or("").to_string();
                                                        if !text.is_empty() {
                                                            let _ = tx.send(AsyncMsg::SttResult { block_idx: idx, text });
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                        Err(e) => { let _ = tx.send(AsyncMsg::SttError(format!("Mic error: {e}"))); }
                                    }
                                });
                            }
                        }))
                )
                // Move up
                .child(
                    div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(if idx > 0 { text_secondary() } else { hsla(0.0, 0.0, 0.2, 1.0) })
                        .child(Icon::new(IconName::ChevronUp))
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            if idx > 0 {
                                this.state.project.blocks.swap(idx, idx - 1);
                                this.state.block_inputs.swap(idx, idx - 1);
                            }
                        }))
                )
                // Move down
                .child(
                    div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(if idx < block_count - 1 { text_secondary() } else { hsla(0.0, 0.0, 0.2, 1.0) })
                        .child(Icon::new(IconName::ChevronDown))
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            if idx + 1 < this.state.project.blocks.len() {
                                this.state.project.blocks.swap(idx, idx + 1);
                                this.state.block_inputs.swap(idx, idx + 1);
                            }
                        }))
                )
                // Toggle
                .child(
                    div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(if block.enabled { success() } else { text_muted() })
                        .child(Icon::new(if block.enabled { IconName::Eye } else { IconName::EyeOff }))
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            if let Some(b) = this.state.project.blocks.get_mut(idx) { b.enabled = !b.enabled; }
                        }))
                )
                // Delete
                .child(
                    div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(danger()).child("x")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            if idx < this.state.project.blocks.len() {
                                // Save undo snapshot
                                this.state.undo_stack.push_back(this.state.project.blocks.clone());
                                while this.state.undo_stack.len() > 50 { this.state.undo_stack.pop_front(); }
                                this.state.project.blocks.remove(idx);
                                if idx < this.state.block_inputs.len() { this.state.block_inputs.remove(idx); }
                            }
                        }))
                );

            // Use Input widget for block content
            let block_input = self.state.block_inputs.get(idx).and_then(|i| i.clone());

            let mut block_content = div().p(px(4.0)).min_h(px(60.0));
            if let Some(input_entity) = block_input {
                block_content = block_content.child(Input::new(&input_entity));
            } else {
                block_content = block_content
                    .text_sm().text_color(text_secondary())
                    .child(content.to_string());
            }

            let block_div = div().rounded(px(8.0))
                .border_1().border_color(border_c())
                .bg(bg_secondary()).overflow_hidden()
                .child(header)
                .child(block_content);

            block_list = block_list.child(block_div);
        }

        // Add block
        let all_types = vec![
            BlockType::Role, BlockType::Context, BlockType::Task,
            BlockType::Examples, BlockType::Constraints, BlockType::Format,
            BlockType::SddConstitution, BlockType::SddSpecification,
            BlockType::SddPlan, BlockType::SddTasks, BlockType::SddImplementation,
        ];

        block_list = block_list.child(
            div().py(px(14.0)).flex().items_center().justify_center()
                .rounded(px(8.0)).border_2().border_color(hsla(0.0, 0.0, 0.5, 0.2))
                .bg(hsla(0.0, 0.0, 0.5, 0.03))
                .text_sm().text_color(text_muted())
                .child(Icon::new(IconName::Plus)).child(" Ajouter un bloc")
                .cursor_pointer().hover(|s| s.bg(hsla(0.0, 0.0, 0.5, 0.06)))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                    this.state.show_add_menu = !this.state.show_add_menu;
                }))
        );

        if self.state.show_add_menu {
            let mut menu = div().p(px(8.0)).rounded(px(8.0)).bg(bg_secondary())
                .border_1().border_color(border_c()).flex().flex_col().gap(px(2.0));
            for bt in all_types {
                let label = bt.label(&self.state.lang).to_string();
                let color = hex_to_hsla(bt.color());
                menu = menu.child(
                    div().px(px(10.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(8.0))
                        .text_xs().text_color(text_secondary())
                        .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(color))
                        .child(label)
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            this.state.project.blocks.push(Block::new(bt));
                            this.state.show_add_menu = false;
                        }))
                );
            }
            block_list = block_list.child(menu);
        }

        // Variables panel (uses cached vars — zero per-frame overhead)
        let vars = self.state.cached_vars.clone();
        if !vars.is_empty() {
            let mut var_panel = div().p(px(12.0)).rounded(px(8.0)).bg(bg_secondary())
                .border_1().border_color(border_c()).flex().flex_col().gap(px(6.0))
                .child(div().text_xs().text_color(text_muted()).child(Icon::new(IconName::Asterisk)).child("Variables"));
            for var in &vars {
                let input_entity = self.state.variable_inputs.get(var).cloned();
                var_panel = var_panel.child(
                    div().flex().items_center().gap(px(8.0))
                        .child(div().w(px(80.0)).text_xs().text_color(accent()).child(format!("{{{{{var}}}}}")))
                        .child(if let Some(entity) = input_entity {
                            div().flex_1().child(Input::new(&entity))
                        } else {
                            div().flex_1().h(px(28.0)).px(px(8.0)).bg(bg_tertiary())
                                .rounded(px(4.0)).border_1().border_color(border_c())
                                .flex().items_center().text_xs().text_color(text_muted())
                                .child("loading...")
                        })
                );
            }
            block_list = block_list.child(var_panel);
        }

        // Variable hint (like web app)
        block_list = block_list.child(
            div().px(px(12.0)).py(px(10.0)).rounded(px(8.0))
                .bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.08))
                .border_1().border_color(hsla(239.0 / 360.0, 0.84, 0.67, 0.15))
                .flex().items_center()
                .child(div().text_xs().text_color(text_muted())
                    .child("Utilisez ")
                )
                .child(div().px(px(4.0)).py(px(1.0)).rounded(px(3.0)).bg(accent_bg())
                    .text_xs().text_color(accent()).child("{{variable}}")
                )
                .child(div().text_xs().text_color(text_muted())
                    .child(" dans vos blocs pour creer des variables.")
                )
        );

        // Tags section
        if !self.state.project.tags.is_empty() || true {
            let mut tags_row = div().flex().flex_wrap().gap(px(4.0));
            for tag in &self.state.project.tags {
                let tag_name = tag.clone();
                tags_row = tags_row.child(
                    div().px(px(8.0)).py(px(3.0)).rounded(px(12.0))
                        .bg(accent_bg()).text_xs().text_color(accent())
                        .flex().items_center().gap(px(4.0))
                        .child(tag.clone())
                        .child(
                            div().text_xs().text_color(text_muted()).child("x")
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                    this.state.project.tags.retain(|t| t != &tag_name);
                                }))
                        )
                );
            }
            // Add tag with real Input widget
            if let Some(ref entity) = self.state.tag_input {
                tags_row = tags_row.child(
                    div().flex().items_center().gap(px(4.0))
                        .child(div().w(px(100.0)).child(Input::new(entity)))
                        .child(
                            div().px(px(8.0)).py(px(3.0)).rounded(px(12.0))
                                .bg(accent()).text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                                .child("+")
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                    let name = this.state.tag_input.as_ref()
                                        .map(|i| i.read(cx).value().to_string())
                                        .unwrap_or_default();
                                    let tag = if name.trim().is_empty() { format!("tag-{}", this.state.project.tags.len() + 1) } else { name.trim().to_string() };
                                    this.state.project.tags.push(tag);
                                    this.state.tag_input = None; // Reset input
                                }))
                        )
                );
            } else {
                tags_row = tags_row.child(
                    div().px(px(8.0)).py(px(3.0)).rounded(px(12.0))
                        .border_1().border_color(border_c())
                        .text_xs().text_color(text_muted())
                        .child("+ tag")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            this.state.project.tags.push(format!("tag-{}", this.state.project.tags.len() + 1));
                        }))
                );
            }
            block_list = block_list.child(
                div().p(px(8.0)).rounded(px(8.0)).bg(bg_secondary()).border_1().border_color(border_c())
                    .flex().flex_col().gap(px(4.0))
                    .child(div().flex().items_center().gap(px(4.0))
                        .child(Icon::new(IconName::Star))
                        .child(div().text_xs().text_color(text_muted()).child("Tags")))
                    .child(tags_row)
            );
        }

        div().flex_1().flex().flex_col().min_w_0().overflow_hidden()
            .child(div().flex_1().p(px(16.0)).flex().flex_col().gap(px(12.0)).child(block_list))
    }

    fn render_right_panel(&mut self, cx: &mut Context<Self>) -> Div {
        let tabs = vec![
            ("Preview", RightTab::Preview), ("Playground", RightTab::Playground),
            ("Chat", RightTab::Chat), ("STT", RightTab::Stt),
            ("Optimize", RightTab::Optimize), ("Lint", RightTab::Lint),
            ("GPU", RightTab::Fleet), ("Terminal", RightTab::Terminal),
            ("Export", RightTab::Export), ("History", RightTab::History),
            ("Analytics", RightTab::Analytics), ("Chain", RightTab::Chain),
            ("Collab", RightTab::Collab),
        ];

        let mut tab_bar = div().h(px(36.0)).px(px(6.0)).flex().items_center().gap(px(2.0))
            .border_b_1().border_color(border_c());
        for (label, tab) in tabs {
            let is_active = self.state.right_tab == tab;
            tab_bar = tab_bar.child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(if is_active { accent() } else { text_muted() })
                    .bg(if is_active { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) } else { hsla(0.0, 0.0, 0.0, 0.0) })
                    .child(label.to_string())
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                        this.state.right_tab = tab;
                    }))
            );
        }

        div().w(px(380.0)).flex_shrink_0().border_l_1().border_color(border_c()).bg(bg_secondary())
            .flex().flex_col()
            .child(tab_bar)
            .child(match self.state.right_tab {
                RightTab::Preview => self.render_preview(cx),
                RightTab::Playground => self.render_playground(cx),
                RightTab::Fleet => self.render_fleet(cx),
                RightTab::Export => self.render_export(cx),
                RightTab::History => self.render_history(cx),
                RightTab::Terminal => self.render_terminal(cx),
                RightTab::Stt => self.render_stt(cx),
                RightTab::Optimize => self.render_optimize(cx),
                RightTab::Lint => self.render_lint(),
                RightTab::Chat => self.render_chat(cx),
                RightTab::Analytics => self.render_analytics(cx),
                RightTab::Collab => self.render_collab(cx),
                RightTab::Chain => self.render_chain(cx),
            })
    }

    fn render_preview(&self, cx: &mut Context<Self>) -> Div {
        let compiled = &self.state.cached_prompt;
        let lines = self.state.cached_lines;
        let chars = self.state.cached_chars;
        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0))
            .child(
                div().flex().items_center().gap(px(8.0))
                    .child(div().flex().items_center().gap(px(8.0))
                    .child(Icon::new(IconName::Eye))
                    .child(div().text_xs().text_color(text_muted()).child("Prompt compile"))
                    .child(div().flex_1())
                    .child({
                        let compiled_copy = compiled.clone();
                        let is_copied = self.state.copy_feedback > 0;
                        div().px(px(6.0)).py(px(2.0)).rounded(px(4.0))
                            .text_xs().text_color(if is_copied { success() } else { accent() })
                            .flex().items_center().gap(px(4.0))
                            .child(Icon::new(if is_copied { IconName::Check } else { IconName::Copy }))
                            .child(if is_copied { "Copie !" } else { "Copier" })
                            .cursor_pointer().hover(|s| s.bg(accent_bg()))
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(compiled_copy.clone()));
                                this.state.copy_feedback = 120; // ~2s at 60fps
                            }))
                    })
                )
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(text_muted()).child(format!("{lines} lines / {chars} chars")))
            )
            .child(
                div().flex_1().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary())
                    .border_1().border_color(border_c())
                    .text_xs().text_color(text_primary())
                    .child(if compiled.is_empty() { "Commencez a ecrire dans les blocs pour voir le prompt compile...".to_string() } else { compiled.clone() })
            )
    }

    fn render_playground(&self, cx: &mut Context<Self>) -> Div {
        const MODELS: &[&str] = &[
            "gpt-4o-mini", "gpt-4o", "gpt-4.1", "claude-sonnet-4-6", "claude-opus-4-6",
            "gemini-2.5-pro", "gemini-2.5-flash",
        ];

        let mut model_list = div().flex().flex_col().gap(px(2.0));
        for model in MODELS {
            let model_str = model.to_string();
            let is_selected = self.state.playground_selected_models.contains(&model_str);
            let is_active = self.state.selected_model == *model;
            model_list = model_list.child(
                div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                    .flex().items_center().gap(px(6.0))
                    .text_xs().text_color(if is_active { accent() } else { text_secondary() })
                    .bg(if is_active { accent_bg() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                    .hover(|s| s.bg(bg_tertiary()))
                    .child(div().w(px(12.0)).h(px(12.0)).rounded(px(2.0))
                        .border_1().border_color(if is_selected { accent() } else { border_c() })
                        .bg(if is_selected { accent() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                        .flex().items_center().justify_center()
                        .child(if is_selected { Icon::new(IconName::Check).text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)) } else { Icon::new(IconName::Check).text_color(hsla(0.0, 0.0, 0.0, 0.0)) })
                    )
                    .child(model_str.clone())
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                        this.state.selected_model = model_str.clone();
                        // Toggle in selected models list
                        let ms = model_str.clone();
                        if this.state.playground_selected_models.contains(&ms) {
                            this.state.playground_selected_models.retain(|m| m != &ms);
                        } else {
                            this.state.playground_selected_models.push(ms);
                        }
                    }))
            );
        }

        // Temperature + Max tokens
        model_list = model_list
            .child(div().h(px(1.0)).bg(border_c()).my(px(4.0)))
            .child(div().flex().items_center().gap(px(6.0))
                .child(div().text_xs().text_color(text_muted()).child("Temp:"))
                .child(
                    div().px(px(4.0)).py(px(2.0)).rounded(px(3.0)).text_xs().text_color(text_secondary()).child("-")
                        .cursor_pointer().hover(|s| s.bg(bg_tertiary()))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            this.state.playground_temperature = (this.state.playground_temperature - 0.1).max(0.0);
                        }))
                )
                .child(div().text_xs().text_color(accent()).child(format!("{:.1}", self.state.playground_temperature)))
                .child(
                    div().px(px(4.0)).py(px(2.0)).rounded(px(3.0)).text_xs().text_color(text_secondary()).child("+")
                        .cursor_pointer().hover(|s| s.bg(bg_tertiary()))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            this.state.playground_temperature = (this.state.playground_temperature + 0.1).min(2.0);
                        }))
                )
            )
            .child(div().flex().items_center().gap(px(6.0))
                .child(div().text_xs().text_color(text_muted()).child("Tokens:"))
                .child(
                    div().px(px(4.0)).py(px(2.0)).rounded(px(3.0)).text_xs().text_color(text_secondary()).child("-")
                        .cursor_pointer().hover(|s| s.bg(bg_tertiary()))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            this.state.playground_max_tokens = this.state.playground_max_tokens.saturating_sub(256).max(256);
                        }))
                )
                .child(div().text_xs().text_color(accent()).child(format!("{}", self.state.playground_max_tokens)))
                .child(
                    div().px(px(4.0)).py(px(2.0)).rounded(px(3.0)).text_xs().text_color(text_secondary()).child("+")
                        .cursor_pointer().hover(|s| s.bg(bg_tertiary()))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            this.state.playground_max_tokens = (this.state.playground_max_tokens + 256).min(16384);
                        }))
                )
            );

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().text_xs().text_color(text_muted()).child(Icon::new(IconName::Bot)).child("Select Model"))
            .child(model_list)
            .child(div().h(px(1.0)).bg(border_c()))
            // Run all selected models
            .child(
                div().py(px(6.0))
                    .bg(if self.state.multi_model_loading { text_muted() } else { hsla(280.0 / 360.0, 0.7, 0.5, 1.0) })
                    .rounded(px(6.0)).flex().items_center().justify_center()
                    .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                    .child(format!("Run all ({} models)", self.state.playground_selected_models.len()))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        if this.state.multi_model_loading || this.state.playground_selected_models.is_empty() { return; }
                        this.state.multi_model_loading = true;
                        this.state.multi_model_responses.clear();
                        let prompt = this.state.project.compiled_prompt();
                        let selected_models = this.state.playground_selected_models.clone();
                        let server = this.state.server_url.clone();
                        let tx = this.state.msg_tx.clone();
                        let temp = this.state.playground_temperature;
                        let max_tok = this.state.playground_max_tokens;
                        rt().spawn(async move {
                                for model in &selected_models {
                                    let client = reqwest::Client::new();
                                    let start = std::time::Instant::now();
                                    if let Ok(resp) = llm_post(&client, &model, &server, serde_json::json!({"model":model,"messages":[{"role":"user","content":prompt}],"temperature":temp,"max_tokens":max_tok,"stream":false}))                                        .send().await {
                                        let latency = start.elapsed().as_millis() as u64;
                                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                                            let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                            let tokens_out = (text.len() as f64 / 4.0).ceil() as u64;
                                            let _ = tx.send(AsyncMsg::MultiModelResult { model: model.to_string(), response: text.clone() });
                                            let _ = tx.send(AsyncMsg::ExecutionRecorded(crate::state::Execution {
                                                model: model.to_string(),
                                                tokens_in: (prompt.len() as f64 / 4.0).ceil() as u64,
                                                tokens_out, latency_ms: latency,
                                                cost: 0.0, timestamp: chrono::Utc::now().timestamp_millis(),
                                                prompt_preview: prompt.chars().take(80).collect(),
                                                response_preview: text.chars().take(100).collect(),
                                            }));
                                        }
                                    }
                                }
                                let _ = tx.send(AsyncMsg::MultiModelDone);
                            });
                    }))
            )
            .child(
                div().py(px(10.0))
                    .bg(if self.state.playground_loading { text_muted() } else { accent() })
                    .rounded(px(8.0)).flex().items_center().justify_center()
                    .text_sm().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                    .child(if self.state.playground_loading { "Running..." } else { "Run prompt" })
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        if this.state.playground_loading { return; }
                        this.state.playground_loading = true;
                        this.state.playground_response.clear();

                        let prompt = this.state.project.compiled_prompt();
                        let model = this.state.selected_model.clone();
                        let server_url = this.state.server_url.clone();
                        let tx = this.state.msg_tx.clone();
                        let temp = this.state.playground_temperature;
                        let max_tok = this.state.playground_max_tokens;

                        rt().spawn(async move {
                                let start = std::time::Instant::now();
                                let client = reqwest::Client::new();
                                let resp = llm_post(&client, &model, &server_url, serde_json::json!({                                        "model": model,
                                        "messages": [{"role": "user", "content": prompt}],
                                        "temperature": temp,
                                        "max_tokens": max_tok,
                                        "stream": true,
                                    }))
                                    .send().await;

                                match resp {
                                    Ok(r) if r.status().is_success() => {
                                        use futures_util::StreamExt;
                                        let mut stream = r.bytes_stream();
                                        let mut buffer = String::new();
                                        while let Some(chunk) = stream.next().await {
                                            if let Ok(bytes) = chunk {
                                                let text = String::from_utf8_lossy(&bytes);
                                                for line in text.lines() {
                                                    if let Some(data) = line.strip_prefix("data: ") {
                                                        if data == "[DONE]" { continue; }
                                                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                                            if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                                                buffer.push_str(content);
                                                                let _ = tx.send(AsyncMsg::LlmChunk(buffer.clone()));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        let latency = start.elapsed().as_millis() as u64;
                                        let tokens_in = (prompt.len() as f64 / 4.0).ceil() as u64;
                                        let tokens_out = (buffer.len() as f64 / 4.0).ceil() as u64;
                                        if buffer.is_empty() {
                                            let _ = tx.send(AsyncMsg::LlmResponse("(empty response)".into()));
                                        }
                                        // Record execution
                                        let _ = tx.send(AsyncMsg::ExecutionRecorded(crate::state::Execution {
                                            model: model.clone(),
                                            tokens_in, tokens_out, latency_ms: latency,
                                            cost: (tokens_in as f64 * 0.000003) + (tokens_out as f64 * 0.000006),
                                            timestamp: chrono::Utc::now().timestamp_millis(),
                                            prompt_preview: prompt.chars().take(100).collect(),
                                            response_preview: buffer.chars().take(100).collect(),
                                        }));
                                        let _ = tx.send(AsyncMsg::LlmDone);
                                    }
                                    Ok(r) => {
                                        let err = r.text().await.unwrap_or_default();
                                        let _ = tx.send(AsyncMsg::LlmError(err));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(AsyncMsg::LlmError(e.to_string()));
                                    }
                                }
                            });
                    }))
            )
            .child(
                div().flex_1().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary())
                    .border_1().border_color(border_c())
                    .text_xs().text_color(if self.state.playground_response.is_empty() { text_muted() } else { text_primary() })
                    .child(if self.state.playground_response.is_empty() {
                        "Response will appear here...".to_string()
                    } else {
                        self.state.playground_response.clone()
                    })
            )
            // Multi-model results
            .children(if self.state.multi_model_responses.is_empty() { None } else {
                let mut results = div().flex().flex_col().gap(px(4.0));
                for (model, resp) in &self.state.multi_model_responses {
                    results = results.child(
                        div().p(px(8.0)).rounded(px(6.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                            .flex().flex_col().gap(px(2.0))
                            .child(div().text_xs().text_color(accent()).child(model.clone()))
                            .child(div().text_xs().text_color(text_primary()).child(
                                resp.chars().take(300).collect::<String>()
                            ))
                    );
                }
                Some(results)
            })
            // Response stats
            .child({
                let last = self.state.executions.last();
                div().flex().items_center().gap(px(8.0)).flex_wrap()
                    .child(div().text_xs().text_color(text_muted()).child(format!("Model: {}", self.state.selected_model)))
                    .child(div().text_xs().text_color(text_muted()).child(format!("~{} tokens in", self.state.cached_tokens)))
                    .children(last.map(|e| {
                        div().flex().items_center().gap(px(6.0))
                            .child(div().text_xs().text_color(accent()).child(format!("{}ms", e.latency_ms)))
                            .child(div().text_xs().text_color(success()).child(format!("{} in / {} out", e.tokens_in, e.tokens_out)))
                            .child(div().text_xs().text_color(hsla(50.0 / 360.0, 0.8, 0.5, 1.0)).child(format!("${:.6}", e.cost)))
                    }))
            })
    }

    fn render_fleet(&self, cx: &mut Context<Self>) -> Div {
        let mut content = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0));

        content = content.child(
            div().flex().items_center().gap(px(8.0))
                .child(div().text_xs().text_color(text_muted()).child("GPU Nodes"))
                .child(div().flex_1())
                .child(
                    div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                        .text_xs().text_color(text_muted()).child(Icon::new(IconName::Redo)).child("Refresh")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            let token = this.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                            if !token.is_empty() {
                                let server = this.state.server_url.clone();
                                let tx = this.state.msg_tx.clone();
                                rt().spawn(async move {
                                    let mut client = inkwell_core::api_client::ApiClient::new(&server);
                                    client.set_token(token);
                                    if let Ok(nodes) = client.list_nodes().await {
                                        let _ = tx.send(AsyncMsg::NodesLoaded(nodes));
                                    }
                                });
                            }
                        }))
                )
        );

        if self.state.gpu_nodes.is_empty() {
            content = content.child(
                div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .flex().flex_col().gap(px(4.0))
                    .child(div().flex().items_center().gap(px(6.0))
                        .child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0)).bg(success()))
                        .child(div().text_xs().text_color(text_primary()).child("Local server"))
                    )
                    .child(div().text_xs().text_color(text_muted()).child(self.state.server_url.clone()))
            );
        } else {
            for node in &self.state.gpu_nodes {
                let is_online = node.status == "online";
                content = content.child(
                    div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                        .flex().flex_col().gap(px(4.0))
                        .child(div().flex().items_center().gap(px(6.0))
                            .child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0))
                                .bg(if is_online { success() } else { text_muted() }))
                            .child(div().text_xs().text_color(text_primary()).child(node.name.clone()))
                            .child(div().flex_1())
                            .child(div().text_xs().text_color(
                                if is_online { success() } else { danger() }
                            ).child(node.status.clone()))
                        )
                        .child(div().text_xs().text_color(text_muted()).child(
                            if node.gpu_info.is_empty() { node.address.clone() } else { node.gpu_info.clone() }
                        ))
                        .child(div().text_xs().text_color(text_muted()).child(node.address.clone()))
                );
            }
        }

        content = content.child(
            div().text_xs().text_color(text_muted()).child("Connect GPU servers via the Inkwell GPU Server app")
        );

        content
    }

    fn render_export(&self, cx: &mut Context<Self>) -> Div {
        let compiled = self.state.cached_prompt.clone();

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0))
            .child(div().text_xs().text_color(text_muted()).child(Icon::new(IconName::Download)).child("Export"))
            // Export Markdown
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child(Icon::new(IconName::File)).child("Markdown (.md)")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        let content = this.state.project.compiled_prompt();
                        let name = this.state.project.name.clone();
                        std::thread::spawn(move || {
                            let path = format!("{}.md", name.replace(' ', "-").to_lowercase());
                            let _ = std::fs::write(&path, &content);
                        });
                    }))
            )
            // Export JSON
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child(Icon::new(IconName::File)).child("JSON")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        let blocks: Vec<inkwell_core::types::PromptBlock> = this.state.project.blocks.iter().map(|b| {
                            inkwell_core::types::PromptBlock {
                                id: b.id.clone(), block_type: b.block_type,
                                content: b.content.clone(), enabled: b.enabled,
                            }
                        }).collect();
                        let name = this.state.project.name.clone();
                        std::thread::spawn(move || {
                            let json = serde_json::to_string_pretty(&blocks).unwrap_or_default();
                            let path = format!("{}.json", name.replace(' ', "-").to_lowercase());
                            let _ = std::fs::write(&path, &json);
                        });
                    }))
            )
            // Export .specify/
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child(Icon::new(IconName::FolderOpen)).child(".specify/ (Spec Kit)")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        let blocks = this.state.project.blocks.clone();
                        let name = this.state.project.name.clone();
                        std::thread::spawn(move || {
                            let dir = format!(".specify/001-{}", name.replace(' ', "-").to_lowercase());
                            let _ = std::fs::create_dir_all(&dir);
                            let file_map = vec![
                                (BlockType::SddConstitution, "constitution.md"),
                                (BlockType::SddSpecification, "spec.md"),
                                (BlockType::SddPlan, "plan.md"),
                                (BlockType::SddTasks, "tasks.md"),
                                (BlockType::SddImplementation, "implementation.md"),
                            ];
                            for (bt, filename) in file_map {
                                if let Some(b) = blocks.iter().find(|b| b.block_type == bt && b.enabled) {
                                    let _ = std::fs::write(format!("{dir}/{filename}"), &b.content);
                                }
                            }
                            // Agent config files
                            let all_content: String = blocks.iter().filter(|b| b.enabled).map(|b| b.content.as_str()).collect::<Vec<_>>().join("\n\n");
                            let constitution = blocks.iter().find(|b| b.block_type == BlockType::SddConstitution).map(|b| b.content.as_str()).unwrap_or("");
                            let _ = std::fs::write(format!("{dir}/../CLAUDE.md"), format!("# {}\n\n{}\n", name, constitution));
                            let _ = std::fs::write(format!("{dir}/../AGENTS.md"), format!("# {} Agent Instructions\n\n{}\n", name, all_content));
                        });
                    }))
            )
            // Export Anthropic format
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child(Icon::new(IconName::File)).child("Anthropic (Messages API)")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        let blocks = &this.state.project.blocks;
                        let system_parts: Vec<String> = blocks.iter()
                            .filter(|b| b.enabled && (b.block_type == BlockType::Role || b.block_type == BlockType::Context || b.block_type == BlockType::Constraints || b.block_type == BlockType::Format))
                            .map(|b| b.content.clone())
                            .collect();
                        let user_parts: Vec<String> = blocks.iter()
                            .filter(|b| b.enabled && (b.block_type == BlockType::Task || b.block_type == BlockType::Examples))
                            .map(|b| b.content.clone())
                            .collect();
                        let json = serde_json::json!({
                            "model": "claude-sonnet-4-6-20250514",
                            "max_tokens": 4096,
                            "system": system_parts.join("\n\n"),
                            "messages": [{"role": "user", "content": user_parts.join("\n\n")}]
                        });
                        let name = this.state.project.name.clone();
                        std::thread::spawn(move || {
                            let content = serde_json::to_string_pretty(&json).unwrap_or_default();
                            let path = format!("{}-anthropic.json", name.replace(' ', "-").to_lowercase());
                            let _ = std::fs::write(&path, &content);
                        });
                    }))
            )
            // Export OpenAI format
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child(Icon::new(IconName::File)).child("OpenAI (Chat API)")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        let blocks = &this.state.project.blocks;
                        let system_content: String = blocks.iter()
                            .filter(|b| b.enabled && (b.block_type == BlockType::Role || b.block_type == BlockType::Context || b.block_type == BlockType::Constraints || b.block_type == BlockType::Format))
                            .map(|b| b.content.as_str())
                            .collect::<Vec<_>>()
                            .join("\n\n");
                        let user_content: String = blocks.iter()
                            .filter(|b| b.enabled && (b.block_type == BlockType::Task || b.block_type == BlockType::Examples))
                            .map(|b| b.content.as_str())
                            .collect::<Vec<_>>()
                            .join("\n\n");
                        let json = serde_json::json!({
                            "model": "gpt-4o",
                            "messages": [
                                {"role": "system", "content": system_content},
                                {"role": "user", "content": user_content}
                            ]
                        });
                        let name = this.state.project.name.clone();
                        std::thread::spawn(move || {
                            let content = serde_json::to_string_pretty(&json).unwrap_or_default();
                            let path = format!("{}-openai.json", name.replace(' ', "-").to_lowercase());
                            let _ = std::fs::write(&path, &content);
                        });
                    }))
            )
            // Copy to clipboard
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs()
                    .text_color(if self.state.copy_feedback > 0 { success() } else { accent() })
                    .child(Icon::new(if self.state.copy_feedback > 0 { IconName::Check } else { IconName::Copy }))
                    .child(if self.state.copy_feedback > 0 { "Copied!" } else { "Copier" })
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(compiled.clone()));
                        this.state.copy_feedback = 120;
                    }))
            )
            // Export as ZIP
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child(Icon::new(IconName::Download)).child("All projects (.zip)")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        let projects = this.state.projects.clone();
                        let current_blocks = this.state.project.blocks.clone();
                        let current_name = this.state.project.name.clone();
                        std::thread::spawn(move || {
                            let path = "inkwell-export.zip";
                            let file = std::fs::File::create(path).unwrap();
                            let mut zip = zip::ZipWriter::new(file);
                            let options = zip::write::SimpleFileOptions::default();
                            // Export current project
                            let blocks: Vec<inkwell_core::types::PromptBlock> = current_blocks.iter().map(|b| {
                                inkwell_core::types::PromptBlock {
                                    id: b.id.clone(), block_type: b.block_type,
                                    content: b.content.clone(), enabled: b.enabled,
                                }
                            }).collect();
                            let json = serde_json::to_string_pretty(&blocks).unwrap_or_default();
                            let _ = zip.start_file(format!("{}.json", current_name.replace(' ', "-")), options);
                            use std::io::Write;
                            let _ = zip.write_all(json.as_bytes());
                            // Export list of all projects as index
                            let index: Vec<serde_json::Value> = projects.iter().map(|p| {
                                serde_json::json!({"id": p.id, "name": p.name})
                            }).collect();
                            let _ = zip.start_file("index.json", options);
                            let _ = zip.write_all(serde_json::to_string_pretty(&index).unwrap_or_default().as_bytes());
                            let _ = zip.finish();
                        });
                    }))
            )
            .child(div().h(px(1.0)).bg(border_c()).my(px(4.0)))
            // Import from JSON
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(success())
                    .child(Icon::new(IconName::Upload)).child("Import from JSON file")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        let tx = this.state.msg_tx.clone();
                        std::thread::spawn(move || {
                            // Try to read from default import path
                            let home = dirs::home_dir().unwrap_or_default();
                            let import_path = home.join("inkwell-import.json");
                            if let Ok(data) = std::fs::read_to_string(&import_path) {
                                let _ = tx.send(AsyncMsg::LlmResponse(format!("__IMPORT__{data}")));
                            } else {
                                let _ = tx.send(AsyncMsg::LlmResponse("Place a JSON file at ~/inkwell-import.json to import".into()));
                            }
                        });
                    }))
            )
            // Import from clipboard
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(success())
                    .child(Icon::new(IconName::Copy)).child("Import from clipboard")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        if let Some(item) = cx.read_from_clipboard() {
                            let text = item.text().unwrap_or_default().to_string();
                            if let Ok(blocks) = serde_json::from_str::<Vec<inkwell_core::types::PromptBlock>>(&text) {
                                this.state.undo_stack.push_back(this.state.project.blocks.clone());
                                this.state.project.blocks = blocks.into_iter().map(|b| {
                                    Block { id: b.id, block_type: b.block_type, content: b.content, enabled: b.enabled, editing: false }
                                }).collect();
                                this.state.block_inputs.clear();
                            }
                        }
                    }))
            )
    }

    fn render_history(&self, cx: &mut Context<Self>) -> Div {
        let mut content = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0));

        content = content.child(
            div().flex().items_center().gap(px(8.0))
                .child(div().text_xs().text_color(text_muted()).child("Version History"))
                .child(div().flex_1())
                .child({
                    if let Some(ref entity) = self.state.version_label_input {
                        div().w(px(100.0)).child(Input::new(entity))
                    } else {
                        div()
                    }
                })
                .child(
                    div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(accent())
                        .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)).child(Icon::new(IconName::Save)).child("Save version")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            let project_id = this.state.project.id.clone();
                            let blocks: Vec<inkwell_core::types::PromptBlock> = this.state.project.blocks.iter().map(|b| {
                                inkwell_core::types::PromptBlock {
                                    id: b.id.clone(), block_type: b.block_type,
                                    content: b.content.clone(), enabled: b.enabled,
                                }
                            }).collect();
                            let server = this.state.server_url.clone();
                            let token = this.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                            let tx = this.state.msg_tx.clone();
                            let custom_label = this.state.version_label_input.as_ref()
                                .map(|i| i.read(cx).value().to_string())
                                .unwrap_or_default();
                            let label = if custom_label.trim().is_empty() {
                                format!("v{}", chrono::Utc::now().format("%H:%M"))
                            } else {
                                custom_label.trim().to_string()
                            };
                            this.state.version_label_input = None; // Reset

                            // Save version locally
                            let blocks_json = serde_json::to_string(&blocks).unwrap_or_default();
                            let version = inkwell_core::types::Version {
                                id: uuid::Uuid::new_v4().to_string(),
                                project_id: project_id.clone(),
                                blocks_json: blocks_json.clone(),
                                variables_json: "{}".into(),
                                label: label.clone(),
                                created_at: chrono::Utc::now().timestamp_millis(),
                            };
                            this.state.versions.push(version);
                            // Background sync to server (optional)
                            if !token.is_empty() {
                                rt().spawn(async move {
                                    let mut client = inkwell_core::api_client::ApiClient::new(&server);
                                    client.set_token(token);
                                    let _ = client.create_version(&project_id, &blocks_json, "{}", &label).await;
                                });
                            }
                        }))
                )
                .child(
                    div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                        .text_xs().text_color(text_muted()).child("Refresh")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            // Refresh from server only if connected
                            let token = this.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                            if !token.is_empty() {
                                let project_id = this.state.project.id.clone();
                                let server = this.state.server_url.clone();
                                let tx = this.state.msg_tx.clone();
                                rt().spawn(async move {
                                    let mut client = inkwell_core::api_client::ApiClient::new(&server);
                                    client.set_token(token);
                                    if let Ok(versions) = client.list_versions(&project_id).await {
                                        let _ = tx.send(AsyncMsg::VersionsLoaded(versions));
                                    }
                                });
                            }
                        }))
                )
        );

        if self.state.versions.is_empty() {
            content = content.child(div().text_xs().text_color(text_muted()).child("No versions saved yet."));
        } else {
            for v in &self.state.versions {
                let blocks_json = v.blocks_json.clone();
                content = content.child(
                    div().px(px(10.0)).py(px(6.0)).rounded(px(6.0))
                        .border_1().border_color(border_c()).bg(bg_tertiary())
                        .flex().items_center().gap(px(8.0))
                        .child(div().text_xs().text_color(text_primary()).child(v.label.clone()))
                        .child(div().flex_1())
                        .child(div().text_xs().text_color(text_muted()).child(
                            chrono::DateTime::from_timestamp_millis(v.created_at)
                                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                                .unwrap_or_default()
                        ))
                        .child({
                            let diff_json = blocks_json.clone();
                            div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                                .text_xs().text_color(hsla(50.0 / 360.0, 0.8, 0.5, 1.0)).child("Diff")
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                    // Show diff between current and version
                                    if let Ok(v_blocks) = serde_json::from_str::<Vec<inkwell_core::types::PromptBlock>>(&diff_json) {
                                        let current = this.state.project.compiled_prompt();
                                        let mut version_text = String::new();
                                        for b in &v_blocks {
                                            if b.enabled { version_text.push_str(&format!("{}\n", b.content)); }
                                        }
                                        // Line-by-line diff
                                        let cur_lines: Vec<&str> = current.lines().collect();
                                        let ver_lines: Vec<&str> = version_text.lines().collect();
                                        let mut diff = String::from("--- Diff ---\n");
                                        let max = cur_lines.len().max(ver_lines.len());
                                        for i in 0..max {
                                            let cur = cur_lines.get(i).copied().unwrap_or("");
                                            let ver = ver_lines.get(i).copied().unwrap_or("");
                                            if cur != ver {
                                                if !ver.is_empty() { diff.push_str(&format!("- {ver}\n")); }
                                                if !cur.is_empty() { diff.push_str(&format!("+ {cur}\n")); }
                                            } else if !cur.is_empty() {
                                                diff.push_str(&format!("  {cur}\n"));
                                            }
                                        }
                                        this.state.playground_response = diff;
                                        this.state.right_tab = RightTab::Playground;
                                    }
                                }))
                        })
                        .child(
                            div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                                .text_xs().text_color(accent()).child(Icon::new(IconName::Undo)).child("Restore")
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                    if let Ok(blocks) = serde_json::from_str::<Vec<inkwell_core::types::PromptBlock>>(&blocks_json) {
                                        this.state.project.blocks = blocks.into_iter().map(|b| {
                                            Block { id: b.id, block_type: b.block_type, content: b.content, enabled: b.enabled, editing: false }
                                        }).collect();
                                        this.state.block_inputs.clear();
                                    }
                                }))
                        )
                );
            }
        }

        // Execution history
        content = content.child(div().h(px(1.0)).bg(border_c()).my(px(4.0)));
        content = content.child(
            div().text_xs().text_color(text_muted()).child(format!("Executions ({})", self.state.executions.len()))
        );
        if self.state.executions.is_empty() {
            content = content.child(div().text_xs().text_color(text_muted()).child("No executions yet. Run a prompt in Playground."));
        } else {
            let execs: Vec<_> = self.state.executions.iter().rev().take(20).cloned().collect();
            for exec in execs {
                let preview = if exec.response_preview.is_empty() { "...".to_string() } else { exec.response_preview };
                content = content.child(
                    div().px(px(8.0)).py(px(6.0)).rounded(px(6.0))
                        .border_1().border_color(border_c()).bg(bg_tertiary())
                        .flex().flex_col().gap(px(2.0))
                        .child(div().flex().items_center().gap(px(6.0))
                            .child(div().text_xs().text_color(accent()).child(exec.model))
                            .child(div().flex_1())
                            .child(div().text_xs().text_color(text_muted()).child(
                                chrono::DateTime::from_timestamp_millis(exec.timestamp)
                                    .map(|d| d.format("%H:%M:%S").to_string())
                                    .unwrap_or_default()
                            ))
                        )
                        .child(div().flex().items_center().gap(px(8.0))
                            .child(div().text_xs().text_color(success()).child(format!("{}ms", exec.latency_ms)))
                            .child(div().text_xs().text_color(text_secondary()).child(format!("{}/{} tok", exec.tokens_in, exec.tokens_out)))
                            .child(div().text_xs().text_color(hsla(50.0 / 360.0, 0.8, 0.5, 1.0)).child(format!("${:.6}", exec.cost)))
                        )
                        .child(div().text_xs().text_color(text_muted()).child(preview))
                );
            }
        }

        content
    }

    fn render_stt(&self, cx: &mut Context<Self>) -> Div {
        let providers = vec![
            ("Local (Whisper)", SttProvider::Local),
            ("OpenAI Whisper", SttProvider::OpenaiWhisper),
            ("Groq", SttProvider::Groq),
            ("Deepgram", SttProvider::Deepgram),
        ];
        let mut provider_row = div().flex().flex_wrap().gap(px(4.0));
        for (label, provider) in providers {
            let is_active = self.state.stt_provider == provider;
            provider_row = provider_row.child(
                div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                    .bg(if is_active { accent() } else { bg_tertiary() })
                    .text_xs().text_color(if is_active { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                    .child(label.to_string())
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                        this.state.stt_provider = provider;
                    }))
            );
        }

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().text_xs().text_color(text_muted()).child(Icon::new(IconName::Mic)).child("Speech-to-Text"))
            // Provider selection
            .child(
                div().flex().flex_col().gap(px(4.0))
                    .child(div().text_xs().text_color(text_muted()).child("Provider"))
                    .child(provider_row)
            )
            .child(
                div().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .flex().flex_col().gap(px(6.0))
                    .child(div().text_xs().text_color(text_primary()).child("How to use"))
                    .child(div().text_xs().text_color(text_secondary()).child(
                        "Click the Mic button on any block header to start recording."
                    ))
                    .child(div().text_xs().text_color(text_secondary()).child(
                        "Click again to stop. The transcription will be appended to the block."
                    ))
            )
            .child(
                div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .flex().flex_col().gap(px(4.0))
                    .child(div().flex().items_center().gap(px(6.0))
                        .child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0))
                            .bg(if self.state.stt_recording { danger() } else { success() }))
                        .child(div().text_xs().text_color(text_primary()).child(
                            if self.state.stt_recording { "Recording..." } else { "Ready" }
                        ))
                    )
                    .child(div().text_xs().text_color(text_muted()).child(
                        format!("Server: {} | Provider: {:?}", self.state.server_url,
                            match self.state.stt_provider { SttProvider::Local => "Local", SttProvider::OpenaiWhisper => "OpenAI", SttProvider::Groq => "Groq", SttProvider::Deepgram => "Deepgram" })
                    ))
            )
    }

    fn render_optimize(&self, cx: &mut Context<Self>) -> Div {
        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::Sparkles))
                .child(div().text_xs().text_color(text_muted()).child("Prompt Optimizer")))
            .child(div().text_xs().text_color(text_secondary()).child("Improve your prompt using AI. The optimizer rewrites for clarity, specificity, and effectiveness."))
            .child(
                div().py(px(8.0)).px(px(12.0)).rounded(px(6.0)).bg(accent())
                    .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)).child("Optimize prompt")
                    .flex().items_center().justify_center().gap(px(6.0))
                    .child(Icon::new(IconName::Sparkles))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        let prompt = this.state.project.compiled_prompt();
                        let server = this.state.server_url.clone();
                        let tx = this.state.msg_tx.clone();
                        rt().spawn(async move {
                                let client = reqwest::Client::new();
                                if let Ok(resp) = llm_post(&client, "gpt-4o-mini", &server, serde_json::json!({"model":"gpt-4o-mini","messages":[                                        {"role":"system","content":"You are a prompt engineering expert. Rewrite the following prompt to be clearer, more specific, and more effective. Keep the same intent."},
                                        {"role":"user","content":format!("Optimize this prompt:\n\n{prompt}")}
                                    ],"temperature":0.3,"max_tokens":4096,"stream":false})).send().await {
                                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                                        let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                        let _ = tx.send(AsyncMsg::LlmResponse(format!("--- Optimized ---\n{text}")));
                                    }
                                }
                            });
                        this.state.right_tab = RightTab::Playground;
                    }))
            )
            .child(div().text_xs().text_color(text_muted()).child("The optimized prompt will appear in the Playground tab. Copy it back to your blocks manually.")
            )
    }

    fn render_lint(&self) -> Div {
        let blocks = &self.state.project.blocks;
        let enabled = blocks.iter().filter(|b| b.enabled).count();
        let empty = blocks.iter().filter(|b| b.enabled && b.content.trim().is_empty()).count();
        let has_task = blocks.iter().any(|b| b.enabled && b.block_type == BlockType::Task);
        let compiled = &self.state.cached_prompt;
        let unresolved = compiled.matches("{{").count();
        let too_short = self.state.cached_chars < 50 && enabled > 0;
        let too_long = self.state.cached_chars > 10000;

        let mut checks = div().flex().flex_col().gap(px(6.0));

        // Check: no blocks enabled
        if enabled == 0 {
            checks = checks.child(lint_item("error", "No blocks enabled", IconName::TriangleAlert));
        }
        // Check: empty blocks
        if empty > 0 {
            checks = checks.child(lint_item("warning", &format!("{empty} empty block(s)"), IconName::TriangleAlert));
        }
        // Check: no task block
        if !has_task && enabled > 0 {
            checks = checks.child(lint_item("warning", "No task/directive block", IconName::Info));
        }
        // Check: unresolved variables
        if unresolved > 0 {
            checks = checks.child(lint_item("warning", &format!("{unresolved} unresolved variable(s)"), IconName::Info));
        }
        // Check: too short
        if too_short {
            checks = checks.child(lint_item("info", "Prompt seems very short", IconName::Info));
        }
        // Check: too long
        if too_long {
            checks = checks.child(lint_item("warning", "Prompt is very long (>10K chars)", IconName::TriangleAlert));
        }
        // Check: negative instructions
        let has_negative = compiled.contains("don't") || compiled.contains("never") || compiled.contains("avoid") || compiled.contains("do not");
        if has_negative {
            checks = checks.child(lint_item("info", "Contains negative instructions — consider positive framing", IconName::Info));
        }
        // Check: no examples for complex prompts
        let has_examples = self.state.project.blocks.iter().any(|b| b.block_type == BlockType::Examples && b.enabled);
        if !has_examples && compiled.len() > 800 {
            checks = checks.child(lint_item("info", "Complex prompt without examples — consider adding few-shot examples", IconName::Info));
        }
        // All good
        if enabled > 0 && empty == 0 && has_task && unresolved == 0 && !too_short && !too_long && !has_negative {
            checks = checks.child(lint_item("success", "All checks passed!", IconName::Check));
        }

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::TriangleAlert))
                .child(div().text_xs().text_color(text_muted()).child("Linting")))
            .child(checks)
    }

    fn render_chat(&self, cx: &mut Context<Self>) -> Div {
        let mut messages_view = div().flex().flex_col().gap(px(6.0));
        for (role, content) in &self.state.chat_messages {
            let is_user = role == "user";
            messages_view = messages_view.child(
                div().px(px(10.0)).py(px(6.0)).rounded(px(8.0))
                    .bg(if is_user { bg_tertiary() } else { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) })
                    .flex().flex_col().gap(px(2.0))
                    .child(div().text_xs().text_color(if is_user { text_muted() } else { accent() })
                        .child(if is_user { "You" } else { "Assistant" }))
                    .child(div().text_xs().text_color(text_primary()).child(content.clone()))
            );
        }

        div().flex_1().flex().flex_col()
            .child(div().px(px(12.0)).py(px(6.0)).flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::Bot))
                .child(div().text_xs().text_color(text_muted()).child("Conversation")))
            // Messages area
            .child(div().flex_1().p(px(8.0)).flex().flex_col().gap(px(4.0))
                .child(messages_view)
                .child(if self.state.chat_messages.is_empty() {
                    div().text_xs().text_color(text_muted()).child("Start a conversation...")
                } else { div() })
            )
            // Input area
            .child(
                div().h(px(36.0)).px(px(8.0)).border_t_1().border_color(border_c())
                    .flex().items_center().gap(px(6.0))
                    .child({
                        if let Some(ref entity) = self.state.chat_input_entity {
                            div().flex_1().child(Input::new(entity))
                        } else {
                            div().flex_1().text_xs().text_color(text_muted()).child("Type a message...")
                        }
                    })
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(accent())
                            .child(Icon::new(IconName::Play))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                let msg = if let Some(ref entity) = this.state.chat_input_entity {
                                    entity.read(cx).value().to_string()
                                } else { String::new() };
                                if msg.is_empty() { return; }
                                this.state.chat_messages.push(("user".into(), msg.clone()));
                                this.state.chat_input_entity = None;
                                // Call LLM
                                let server = this.state.server_url.clone();
                                let tx = this.state.msg_tx.clone();
                                let messages: Vec<serde_json::Value> = this.state.chat_messages.iter()
                                    .map(|(role, content)| serde_json::json!({"role": role, "content": content}))
                                    .collect();
                                rt().spawn(async move {
                                        let client = reqwest::Client::new();
                                        if let Ok(resp) = llm_post(&client, "gpt-4o-mini", &server, serde_json::json!({"model":"gpt-4o-mini","messages":messages,"temperature":0.7,"max_tokens":2048,"stream":false}))                                            .send().await {
                                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                                let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                                let _ = tx.send(AsyncMsg::LlmResponse(format!("__CHAT__{text}")));
                                            }
                                        }
                                    });
                            }))
                    )
            )
    }

    fn render_analytics(&self, cx: &mut Context<Self>) -> Div {
        let now = chrono::Utc::now().timestamp_millis();
        let range_ms: i64 = match self.state.analytics_range {
            AnalyticsRange::Week => 7 * 24 * 3600 * 1000,
            AnalyticsRange::Month => 30 * 24 * 3600 * 1000,
            AnalyticsRange::All => i64::MAX,
        };
        let filtered: Vec<&Execution> = self.state.executions.iter()
            .filter(|e| now - e.timestamp < range_ms)
            .collect();
        let exec_count = filtered.len();
        let total_tokens_in: u64 = filtered.iter().map(|e| e.tokens_in).sum();
        let total_tokens_out: u64 = filtered.iter().map(|e| e.tokens_out).sum();
        let total_cost: f64 = filtered.iter().map(|e| e.cost).sum();
        let avg_latency = if exec_count > 0 { filtered.iter().map(|e| e.latency_ms).sum::<u64>() / exec_count as u64 } else { 0 };
        let token_count = self.state.cached_tokens;

        let ranges = vec![
            ("7d", AnalyticsRange::Week), ("30d", AnalyticsRange::Month), ("All", AnalyticsRange::All),
        ];
        let mut range_row = div().flex().gap(px(4.0));
        for (label, range) in ranges {
            let is_active = self.state.analytics_range == range;
            range_row = range_row.child(
                div().px(px(8.0)).py(px(3.0)).rounded(px(4.0))
                    .bg(if is_active { accent() } else { bg_tertiary() })
                    .text_xs().text_color(if is_active { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                    .child(label.to_string())
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                        this.state.analytics_range = range;
                    }))
            );
        }

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(12.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::ChartPie))
                .child(div().text_xs().text_color(text_muted()).child("Analytics"))
                .child(div().flex_1())
                .child(range_row))
            // KPI cards
            .child(div().flex().gap(px(8.0))
                .child(kpi_card("Executions", &exec_count.to_string(), accent()))
                .child(kpi_card("Tokens", &format!("~{token_count}"), success()))
                .child(kpi_card("Blocks", &self.state.project.blocks.len().to_string(), text_secondary()))
            )
            .child(div().flex().gap(px(8.0))
                .child(kpi_card("Tokens In", &total_tokens_in.to_string(), accent()))
                .child(kpi_card("Tokens Out", &total_tokens_out.to_string(), success()))
                .child(kpi_card("Avg Latency", &format!("{}ms", avg_latency), hsla(50.0 / 360.0, 0.8, 0.5, 1.0)))
            )
            .child(
                div().p(px(8.0)).rounded(px(6.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .flex().items_center().gap(px(8.0))
                    .child(div().text_xs().text_color(text_muted()).child("Total cost:"))
                    .child(div().text_xs().text_color(hsla(50.0 / 360.0, 0.8, 0.5, 1.0)).child(format!("${:.6}", total_cost)))
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(text_muted()).child(format!("{} versions", self.state.versions.len())))
            )
    }

    fn render_collab(&self, cx: &mut Context<Self>) -> Div {
        let colors = vec![accent(), success(), hsla(280.0 / 360.0, 0.7, 0.6, 1.0), hsla(50.0 / 360.0, 0.8, 0.5, 1.0), danger()];

        let mut content = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::User))
                .child(div().text_xs().text_color(text_muted()).child("Collaboration"))
                .child(div().flex_1())
                .child(
                    div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(text_muted()).child(Icon::new(IconName::Redo)).child("Refresh")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            // Poll collab users from backend
                            let server = this.state.server_url.clone();
                            let token = this.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                            let project_id = this.state.project.id.clone();
                            let tx = this.state.msg_tx.clone();
                            rt().spawn(async move {
                                    let client = reqwest::Client::new();
                                    if let Ok(resp) = client.get(format!("{server}/api/projects/{project_id}/presence"))
                                        .header("Authorization", format!("Bearer {token}"))
                                        .send().await {
                                        if let Ok(users) = resp.json::<Vec<serde_json::Value>>().await {
                                            let collab_users: Vec<crate::state::CollabUser> = users.iter().map(|u| {
                                                crate::state::CollabUser {
                                                    name: u["display_name"].as_str().unwrap_or("").to_string(),
                                                    email: u["email"].as_str().unwrap_or("").to_string(),
                                                    online: u["online"].as_bool().unwrap_or(false),
                                                }
                                            }).collect();
                                            let _ = tx.send(AsyncMsg::CollabUsersLoaded(collab_users));
                                        }
                                    }
                                });
                        }))
                ));

        // Current user
        content = content.child(
            div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                .flex().items_center().gap(px(8.0))
                .child(div().w(px(24.0)).h(px(24.0)).rounded(px(12.0)).bg(accent())
                    .flex().items_center().justify_center()
                    .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                    .child(self.state.session.as_ref().map(|s| s.email.chars().next().unwrap_or('U').to_uppercase().to_string()).unwrap_or("U".into())))
                .child(div().flex().flex_col()
                    .child(div().text_xs().text_color(text_primary()).child(
                        self.state.session.as_ref().map(|s| s.display_name.clone()).unwrap_or("You".into())))
                    .child(div().text_xs().text_color(success()).child("Online (you)"))
                )
        );

        // Other collaborators
        for (i, user) in self.state.collab_users.iter().enumerate() {
            let color = colors[i % colors.len()];
            content = content.child(
                div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .flex().items_center().gap(px(8.0))
                    .child(div().w(px(24.0)).h(px(24.0)).rounded(px(12.0)).bg(color)
                        .flex().items_center().justify_center()
                        .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                        .child(user.name.chars().next().unwrap_or('?').to_uppercase().to_string()))
                    .child(div().flex().flex_col()
                        .child(div().text_xs().text_color(text_primary()).child(user.name.clone()))
                        .child(div().text_xs().text_color(if user.online { success() } else { text_muted() })
                            .child(if user.online { "Online" } else { "Offline" }))
                    )
            );
        }

        if self.state.collab_users.is_empty() {
            content = content.child(div().text_xs().text_color(text_muted()).child("Click Refresh to check for collaborators."));
        }

        content
    }

    fn render_chain(&self, cx: &mut Context<Self>) -> Div {
        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::Network))
                .child(div().text_xs().text_color(text_muted()).child("Prompt Chain")))
            .child(div().text_xs().text_color(text_secondary()).child("Execute multiple prompts sequentially. The output of each step feeds into the next."))
            .child(
                div().py(px(8.0)).px(px(12.0)).rounded(px(6.0)).bg(accent())
                    .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                    .flex().items_center().justify_center().gap(px(6.0))
                    .child(Icon::new(IconName::Play))
                    .child("Run chain")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        // Chain: execute each block sequentially, passing output as context
                        let blocks: Vec<String> = this.state.project.blocks.iter()
                            .filter(|b| b.enabled && !b.content.is_empty())
                            .map(|b| b.content.clone())
                            .collect();
                        let server = this.state.server_url.clone();
                        let tx = this.state.msg_tx.clone();
                        rt().spawn(async move {
                                let client = reqwest::Client::new();
                                let mut chain_output = String::new();
                                for (i, block_content) in blocks.iter().enumerate() {
                                    let prompt = if chain_output.is_empty() {
                                        block_content.clone()
                                    } else {
                                        format!("Previous output:\n{chain_output}\n\nNow:\n{block_content}")
                                    };
                                    if let Ok(resp) = llm_post(&client, "gpt-4o-mini", &server, serde_json::json!({"model":"gpt-4o-mini","messages":[{"role":"user","content":prompt}],"temperature":0.7,"max_tokens":2048,"stream":false}))                                        .send().await {
                                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                                            let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                            chain_output = text.clone();
                                            let _ = tx.send(AsyncMsg::LlmResponse(format!("--- Step {} ---\n{text}", i + 1)));
                                        }
                                    }
                                }
                                let _ = tx.send(AsyncMsg::LlmDone);
                            });
                        this.state.right_tab = RightTab::Playground;
                    }))
            )
    }

    fn render_settings(&self, cx: &mut Context<Self>) -> Div {
        let lang = self.state.lang.clone();
        div().h(px(280.0)).flex_shrink_0()
            .border_t_1().border_color(border_c()).bg(bg_secondary())
            .p(px(16.0)).flex().flex_col().gap(px(12.0))
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
            )
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

    fn render_terminal(&mut self, cx: &mut Context<Self>) -> Div {
        div().flex_1().flex().flex_col()
            .child(
                div().px(px(12.0)).py(px(6.0)).flex().items_center().gap(px(8.0))
                    .border_b_1().border_color(border_c())
                    .child(div().text_xs().text_color(text_muted()).child("Terminal"))
                    // Session tabs
                    .child({
                        let mut tabs = div().flex().gap(px(2.0));
                        for (i, session) in self.state.terminal_sessions.iter().enumerate() {
                            let is_active = i == self.state.active_terminal;
                            tabs = tabs.child(
                                div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                                    .text_xs().text_color(if is_active { accent() } else { text_muted() })
                                    .bg(if is_active { accent_bg() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                                    .child(session.label.clone())
                                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                        this.state.active_terminal = i;
                                    }))
                            );
                        }
                        tabs
                    })
                    // SSH button
                    .child(
                        div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                            .text_xs().text_color(text_muted())
                            .flex().items_center().gap(px(4.0))
                            .child(Icon::new(IconName::Globe))
                            .child("SSH")
                            .cursor_pointer().hover(|s| s.bg(accent_bg()))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.state.show_ssh_modal = !this.state.show_ssh_modal;
                            }))
                    )
                    .child(div().flex_1())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(if self.state.terminal_sessions.get(self.state.active_terminal).map(|s| s.running).unwrap_or(false) { danger() } else { success() })
                            .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                            .child(if self.state.terminal_sessions.get(self.state.active_terminal).map(|s| s.running).unwrap_or(false) { "Stop" } else { "Start" })
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                if this.state.terminal_sessions.get(this.state.active_terminal).map(|s| s.running).unwrap_or(false) {
                                    if let Some(session) = this.state.terminal_sessions.get_mut(this.state.active_terminal) {
                                        session.running = false;
                                    }
                                } else {
                                    let (input_tx, input_rx) = std::sync::mpsc::channel::<String>();
                                    this.state.terminal_sessions.push(crate::state::TerminalSession {
                                        label: format!("Shell {}", this.state.terminal_sessions.len() + 1),
                                        output: String::new(),
                                        running: true,
                                        input_tx: Some(input_tx),
                                    });
                                    this.state.active_terminal = this.state.terminal_sessions.len() - 1;

                                    let tx = this.state.msg_tx.clone();


                                    std::thread::spawn(move || {
                                        use std::io::{Read, Write};
                                        let pty_system = portable_pty::native_pty_system();
                                        let pair = pty_system.openpty(portable_pty::PtySize {
                                            rows: 24, cols: 80, pixel_width: 0, pixel_height: 0,
                                        }).unwrap();

                                        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
                                        let mut cmd = portable_pty::CommandBuilder::new(&shell);
                                        cmd.env("TERM", "dumb");
                                        let _child = pair.slave.spawn_command(cmd).unwrap();
                                        drop(pair.slave);

                                        let mut reader = pair.master.try_clone_reader().unwrap();
                                        let mut writer = pair.master.take_writer().unwrap();

                                        // Writer thread — reads from input channel and writes to PTY
                                        std::thread::spawn(move || {
                                            while let Ok(input) = input_rx.recv() {
                                                let _ = writer.write_all(input.as_bytes());
                                                let _ = writer.flush();
                                            }
                                        });

                                        // Reader thread — reads PTY output
                                        let mut buf = [0u8; 4096];
                                        loop {
                                            match reader.read(&mut buf) {
                                                Ok(0) => break,
                                                Ok(n) => {
                                                    let text = String::from_utf8_lossy(&buf[..n]).to_string();
                                                    let clean = strip_ansi(&text);
                                                    if tx.send(AsyncMsg::TerminalOutput(clean)).is_err() { break; }
                                                }
                                                Err(_) => break,
                                            }
                                        }
                                    });
                                }
                            }))
                    )
            )
            // Output area
            .child(
                div().flex_1().p(px(8.0)).bg(hsla(0.0, 0.0, 0.04, 1.0))
                    .text_xs().text_color(hsla(120.0 / 360.0, 0.8, 0.6, 1.0))
                    .child(if self.state.terminal_sessions.get(self.state.active_terminal).map(|s| s.output.as_str()).unwrap_or("").is_empty() {
                        "Click Start to open a terminal session".to_string()
                    } else {
                        let lines: Vec<&str> = self.state.terminal_sessions.get(self.state.active_terminal).map(|s| s.output.as_str()).unwrap_or("").lines().collect();
                        let start = if lines.len() > 50 { lines.len() - 50 } else { 0 };
                        lines[start..].join("\n")
                    })
            )
            // Input area with real Input widget
            .child(
                div().h(px(36.0)).px(px(8.0)).bg(hsla(0.0, 0.0, 0.06, 1.0))
                    .border_t_1().border_color(border_c())
                    .flex().items_center().gap(px(6.0))
                    .child(div().text_xs().text_color(hsla(120.0 / 360.0, 0.8, 0.6, 1.0)).child(Icon::new(IconName::SquareTerminal)))
                    .child({
                        if let Some(ref entity) = self.state.terminal_input_entity {
                            div().flex_1().child(Input::new(entity))
                        } else {
                            div().flex_1().text_xs().text_color(text_muted()).child("...")
                        }
                    })
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(success())
                            .text_xs().text_color(hsla(0.0, 0.0, 0.0, 1.0)).child(Icon::new(IconName::Play)).child("Run")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                let cmd = if let Some(ref entity) = this.state.terminal_input_entity {
                                    let val = entity.read(cx).value().to_string();
                                    val
                                } else {
                                    String::new()
                                };
                                if !cmd.is_empty() {
                                    if let Some(session) = this.state.terminal_sessions.get(this.state.active_terminal) {
                                        if let Some(ref tx) = session.input_tx {
                                            let _ = tx.send(format!("{cmd}\n"));
                                        }
                                    }
                                    this.state.terminal_input_entity = None;
                                }
                            }))
                    )
            )
            // SSH Modal
            .children(if self.state.show_ssh_modal {
                Some(div().p(px(12.0)).bg(bg_secondary()).border_t_1().border_color(border_c())
                    .flex().flex_col().gap(px(8.0))
                    .child(div().flex().items_center().gap(px(6.0))
                        .child(Icon::new(IconName::Globe))
                        .child(div().text_xs().text_color(text_primary()).child("SSH Connection"))
                        .child(div().flex_1())
                        .child(div().text_xs().text_color(text_muted()).child("Close")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.state.show_ssh_modal = false;
                            })))
                    )
                    .child(div().flex().gap(px(6.0))
                        .child({
                            if let Some(ref entity) = self.state.ssh_host_input {
                                div().flex_1().child(Input::new(entity))
                            } else {
                                div().flex_1().h(px(28.0)).px(px(8.0)).bg(bg_tertiary()).rounded(px(4.0))
                                    .border_1().border_color(border_c()).flex().items_center()
                                    .text_xs().text_color(text_muted()).child("host")
                            }
                        })
                        .child({
                            if let Some(ref entity) = self.state.ssh_port_input {
                                div().w(px(60.0)).child(Input::new(entity))
                            } else {
                                div().w(px(60.0)).h(px(28.0)).px(px(8.0)).bg(bg_tertiary()).rounded(px(4.0))
                                    .border_1().border_color(border_c()).flex().items_center()
                                    .text_xs().text_color(text_secondary()).child("22")
                            }
                        })
                    )
                    .child({
                        if let Some(ref entity) = self.state.ssh_user_input {
                            div().child(Input::new(entity))
                        } else {
                            div().h(px(28.0)).px(px(8.0)).bg(bg_tertiary()).rounded(px(4.0))
                                .border_1().border_color(border_c()).flex().items_center()
                                .text_xs().text_color(text_muted()).child("username")
                        }
                    })
                    .child(
                        div().py(px(6.0)).bg(accent()).rounded(px(6.0))
                            .flex().items_center().justify_center().gap(px(6.0))
                            .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                            .child(Icon::new(IconName::Globe))
                            .child("Connect")
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                let host = this.state.ssh_host_input.as_ref()
                                    .map(|i| i.read(cx).value().to_string())
                                    .filter(|s| !s.is_empty())
                                    .unwrap_or_else(|| "localhost".into());
                                let user = this.state.ssh_user_input.as_ref()
                                    .map(|i| i.read(cx).value().to_string())
                                    .filter(|s| !s.is_empty())
                                    .unwrap_or_else(|| "root".into());
                                let port = this.state.ssh_port_input.as_ref()
                                    .map(|i| i.read(cx).value().to_string())
                                    .filter(|s| !s.is_empty())
                                    .unwrap_or_else(|| "22".into());
                                let (input_tx, input_rx) = std::sync::mpsc::channel::<String>();
                                this.state.terminal_sessions.push(crate::state::TerminalSession {
                                    label: format!("{}@{}", user, host),
                                    output: String::new(), running: true, input_tx: Some(input_tx),
                                });
                                this.state.active_terminal = this.state.terminal_sessions.len() - 1;
                                this.state.show_ssh_modal = false;
                                let tx = this.state.msg_tx.clone();
                                std::thread::spawn(move || {
                                    use std::io::{Read, Write};
                                    let pty_system = portable_pty::native_pty_system();
                                    let pair = pty_system.openpty(portable_pty::PtySize {
                                        rows: 24, cols: 80, pixel_width: 0, pixel_height: 0,
                                    }).unwrap();
                                    let mut cmd = portable_pty::CommandBuilder::new("ssh");
                                    cmd.arg("-o"); cmd.arg("StrictHostKeyChecking=accept-new");
                                    cmd.arg("-p"); cmd.arg(&port);
                                    cmd.arg(format!("{user}@{host}"));
                                    cmd.env("TERM", "dumb");
                                    let _child = pair.slave.spawn_command(cmd).unwrap();
                                    drop(pair.slave);
                                    let mut reader = pair.master.try_clone_reader().unwrap();
                                    let mut writer = pair.master.take_writer().unwrap();
                                    std::thread::spawn(move || {
                                        while let Ok(input) = input_rx.recv() {
                                            let _ = writer.write_all(input.as_bytes());
                                            let _ = writer.flush();
                                        }
                                    });
                                    let mut buf = [0u8; 4096];
                                    loop {
                                        match reader.read(&mut buf) {
                                            Ok(0) => break,
                                            Ok(n) => {
                                                let text = String::from_utf8_lossy(&buf[..n]).to_string();
                                                let clean = crate::ui::colors::strip_ansi(&text);
                                                if tx.send(AsyncMsg::TerminalOutput(clean)).is_err() { break; }
                                            }
                                            Err(_) => break,
                                        }
                                    }
                                });
                            }))
                    )
                )
            } else { None })
    }

    fn render_bottom_bar(&self, cx: &mut Context<Self>) -> Div {
        let chars = self.state.cached_chars;
        let words = self.state.cached_words;
        let lines = self.state.cached_lines;
        let tokens = self.state.cached_tokens;
        let cost = tokens as f64 * 0.000003;
        let enabled = self.state.project.blocks.iter().filter(|b| b.enabled).count();
        let total = self.state.project.blocks.len();

        div().h(px(28.0)).px(px(12.0)).flex().items_center().gap(px(10.0))
            .border_t_1().border_color(border_c()).bg(bg_secondary())
            .child(div().text_xs().text_color(text_muted()).child(format!("{chars} car.")))
            .child(div().text_xs().text_color(text_muted()).child(format!("{words} mots")))
            .child(div().text_xs().text_color(text_muted()).child(format!("{lines} lignes")))
            .child(div().text_xs().text_color(text_muted()).child(format!("~{tokens} tokens")))
            .child(div().text_xs().text_color(text_muted()).child(format!("~${cost:.6}")))
            .child({
                let max_ctx = 128000u64;
                let pct = (tokens as f64 / max_ctx as f64 * 100.0).min(100.0);
                let bar_color = if pct > 80.0 { danger() } else if pct > 50.0 { hsla(50.0 / 360.0, 0.8, 0.5, 1.0) } else { accent() };
                div().w(px(40.0)).h(px(4.0)).rounded(px(2.0)).bg(bg_tertiary())
                    .child(div().h(px(4.0)).rounded(px(2.0)).bg(bar_color)
                        .w(px(pct as f32 / 100.0 * 40.0)))
            })
            .child(div().text_xs().text_color(text_muted()).child(format!("{:.1}%", tokens as f64 / 128000.0 * 100.0)))
            .child(div().w(px(1.0)).h(px(12.0)).bg(border_c()))
            .child(div().text_xs().text_color(text_muted()).child(format!("{enabled}/{total} blocs")))
            .child(div().flex_1())
            // Terminal button
            .child(
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0)).text_xs()
                    .text_color(text_muted()).child(Icon::new(IconName::SquareTerminal)).child("Terminal")
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.right_tab = RightTab::Terminal;
                        this.state.right_open = true;
                    }))
            )
            .child(div().w(px(1.0)).h(px(12.0)).bg(border_c()))
            .child(div().text_xs().text_color(text_secondary()).child(self.state.selected_model.clone()))
    }
}

// hex_to_hsla and strip_ansi are in ui::colors

fn lint_item(severity: &str, message: &str, icon: IconName) -> Div {
    let color = match severity {
        "error" => danger(),
        "warning" => hsla(50.0 / 360.0, 0.8, 0.5, 1.0),
        "success" => success(),
        _ => text_muted(),
    };
    div().px(px(10.0)).py(px(6.0)).rounded(px(6.0))
        .bg(hsla(color.h, color.s, color.l, 0.1))
        .border_1().border_color(hsla(color.h, color.s, color.l, 0.2))
        .flex().items_center().gap(px(8.0))
        .child(Icon::new(icon).text_color(color))
        .child(div().text_xs().text_color(color).child(message.to_string()))
}

fn kpi_card(label: &str, value: &str, color: Hsla) -> Div {
    div().flex_1().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary())
        .border_1().border_color(border_c())
        .flex().flex_col().items_center().gap(px(4.0))
        .child(div().text_xl().text_color(color).child(value.to_string()))
        .child(div().text_xs().text_color(text_muted()).child(label.to_string()))
}
