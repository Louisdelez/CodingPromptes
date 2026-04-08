use gpui::*;
use gpui_component::input::{Input, InputState};
use crate::state::*;
use inkwell_core::types::BlockType;

// Actions for keyboard shortcuts
actions!(inkwell, [NewProject, ToggleTerminal, RunPrompt, ToggleSettings, Undo, SaveNow]);

use crate::theme::InkwellTheme;

// i18n helper
fn tr<'a>(key: &'a str, lang: &str) -> &'a str {
    inkwell_core::i18n::t(key, lang)
}

// Theme-aware color helpers
// These use a thread-local to avoid passing theme everywhere
use std::cell::RefCell;
thread_local! {
    static DARK_MODE: RefCell<bool> = const { RefCell::new(true) };
}
fn set_dark_mode(dark: bool) { DARK_MODE.with(|d| *d.borrow_mut() = dark); }
fn is_dark() -> bool { DARK_MODE.with(|d| *d.borrow()) }

fn bg_primary() -> Hsla { if is_dark() { hsla(230.0/360.0, 0.15, 0.07, 1.0) } else { hsla(0.0, 0.0, 1.0, 1.0) } }
fn bg_secondary() -> Hsla { if is_dark() { hsla(230.0/360.0, 0.12, 0.10, 1.0) } else { hsla(220.0/360.0, 0.10, 0.97, 1.0) } }
fn bg_tertiary() -> Hsla { if is_dark() { hsla(230.0/360.0, 0.10, 0.14, 1.0) } else { hsla(220.0/360.0, 0.08, 0.93, 1.0) } }
fn border_c() -> Hsla { if is_dark() { hsla(230.0/360.0, 0.10, 0.20, 1.0) } else { hsla(220.0/360.0, 0.10, 0.85, 1.0) } }
fn text_primary() -> Hsla { if is_dark() { hsla(0.0, 0.0, 0.95, 1.0) } else { hsla(220.0/360.0, 0.15, 0.10, 1.0) } }
fn text_secondary() -> Hsla { if is_dark() { hsla(0.0, 0.0, 0.70, 1.0) } else { hsla(220.0/360.0, 0.10, 0.35, 1.0) } }
fn text_muted() -> Hsla { if is_dark() { hsla(0.0, 0.0, 0.50, 1.0) } else { hsla(220.0/360.0, 0.05, 0.55, 1.0) } }
fn accent() -> Hsla { hsla(239.0 / 360.0, 0.84, if is_dark() { 0.67 } else { 0.55 }, 1.0) }
fn danger() -> Hsla { hsla(0.0, 0.75, if is_dark() { 0.55 } else { 0.45 }, 1.0) }
fn success() -> Hsla { hsla(150.0 / 360.0, 0.65, if is_dark() { 0.45 } else { 0.35 }, 1.0) }

pub struct InkwellApp {
    pub state: AppState,
}

impl InkwellApp {
    pub fn new() -> Self {
        Self { state: AppState::new() }
    }

    fn t(&self) -> InkwellTheme {
        InkwellTheme::from_mode(self.state.dark_mode)
    }
}

impl Render for InkwellApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        set_dark_mode(self.state.dark_mode);
        self.poll_messages();

        match self.state.screen {
            Screen::Auth => self.render_auth(window, cx),
            Screen::Ide => {
                self.ensure_block_inputs(window, cx);
                self.ensure_terminal_input(window, cx);
                self.sync_block_content(cx);
                self.render_ide(cx)
            }
        }
    }
}

impl InkwellApp {
    fn poll_messages(&mut self) {
        while let Ok(msg) = self.state.msg_rx.try_recv() {
            match msg {
                AsyncMsg::AuthSuccess { session, projects, workspaces } => {
                    self.state.auth_loading = false;
                    self.state.session = Some(session);
                    self.state.screen = Screen::Ide;
                    self.state.projects = projects.iter().map(|p| {
                        ProjectSummary { id: p.id.clone(), name: p.name.clone() }
                    }).collect();
                    self.state.workspaces = workspaces;
                    // Load first project
                    if let Some(first) = projects.first() {
                        self.state.project.name = first.name.clone();
                        self.state.project.id = first.id.clone();
                        self.state.project.blocks = first.blocks.iter().map(|b| {
                            Block {
                                id: b.id.clone(), block_type: b.block_type,
                                content: b.content.clone(), enabled: b.enabled, editing: false,
                            }
                        }).collect();
                        self.state.project.framework = first.framework.clone();
                    }
                }
                AsyncMsg::AuthError(e) => {
                    self.state.auth_loading = false;
                    self.state.auth_error = Some(e);
                }
                AsyncMsg::LlmResponse(text) => {
                    self.state.playground_response = text;
                }
                AsyncMsg::LlmChunk(text) => {
                    self.state.playground_response = text;
                }
                AsyncMsg::LlmDone => {
                    self.state.playground_loading = false;
                    self.state.sdd_running = false;
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
                AsyncMsg::TerminalOutput(text) => {
                    self.state.terminal_output.push_str(&text);
                    // Cap at 10K chars
                    if self.state.terminal_output.len() > 10_000 {
                        let start = self.state.terminal_output.len() - 8_000;
                        self.state.terminal_output = self.state.terminal_output[start..].to_string();
                    }
                }
            }
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

        let server_input = self.state.server_url_input.clone().unwrap();
        let email_input = self.state.email_input.clone().unwrap();
        let password_input = self.state.password_input.clone().unwrap();

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
                                    .text_xs().text_color(if self.state.auth_mode == AuthMode::Login { hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                                    .flex().items_center().justify_center().child("Sign in")
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.auth_mode = AuthMode::Login; }))
                            )
                            .child(
                                div().flex_1().py(px(6.0)).rounded(px(6.0))
                                    .bg(if self.state.auth_mode == AuthMode::Register { accent() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                                    .text_xs().text_color(if self.state.auth_mode == AuthMode::Register { hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                                    .flex().items_center().justify_center().child("Sign up")
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.auth_mode = AuthMode::Register; }))
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
                            .text_sm().text_color(hsla(0.0, 0.0, 1.0, 1.0))
                            .child(if self.state.auth_loading { "Connecting..." }
                                else if self.state.auth_mode == AuthMode::Register { "Sign up" }
                                else { "Sign in" })
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                if this.state.auth_loading { return; }
                                this.state.auth_loading = true;
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
                                let tx = this.state.msg_tx.clone();
                                let is_register = this.state.auth_mode == AuthMode::Register;
                                let display_name = email.split('@').next().unwrap_or("User").to_string();

                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async {
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
                                });
                            }))
                    )
                    // Skip auth (for dev)
                    .child(
                        div().py(px(6.0)).flex().items_center().justify_center()
                            .text_xs().text_color(text_muted()).child("Skip (offline mode)")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.state.screen = Screen::Ide;
                            }))
                    )
            )
    }

    fn sync_block_content(&mut self, cx: &mut Context<Self>) {
        // Read content from Input widgets back into block state
        let mut changed = false;
        for (idx, block) in self.state.project.blocks.iter_mut().enumerate() {
            if let Some(Some(input)) = self.state.block_inputs.get(idx) {
                let new_content = input.read(cx).value().to_string();
                if new_content != block.content {
                    block.content = new_content;
                    changed = true;
                }
            }
        }
        // Auto-save to backend (debounced via save_timer)
        if changed {
            self.state.save_pending = true;
        }
        if self.state.save_pending && self.state.save_timer == 0 {
            self.state.save_timer = 30; // ~30 frames = ~0.5s at 60fps
        }
        if self.state.save_timer > 0 {
            self.state.save_timer -= 1;
            if self.state.save_timer == 0 && self.state.save_pending {
                self.state.save_pending = false;
                self.save_to_backend();
            }
        }
    }

    fn save_to_backend(&self) {
        if self.state.session.is_none() { return; }
        let project_id = self.state.project.id.clone();
        let blocks: Vec<inkwell_core::types::PromptBlock> = self.state.project.blocks.iter().map(|b| {
            inkwell_core::types::PromptBlock {
                id: b.id.clone(), block_type: b.block_type,
                content: b.content.clone(), enabled: b.enabled,
            }
        }).collect();
        let name = self.state.project.name.clone();
        let framework = self.state.project.framework.clone();
        let server_url = self.state.server_url.clone();
        let token = self.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut client = inkwell_core::api_client::ApiClient::new(&server_url);
                client.set_token(token);
                let _ = client.update_project(&project_id, &serde_json::json!({
                    "name": name,
                    "blocks_json": serde_json::to_string(&blocks).unwrap_or_default(),
                    "framework": framework,
                })).await;
            });
        });
    }

    fn ensure_terminal_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.terminal_input_entity.is_none() {
            self.state.terminal_input_entity = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Enter command...")
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
    }

    fn render_ide(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.t();
        let mut main_row = div().flex_1().flex().overflow_hidden();
        if self.state.left_open { main_row = main_row.child(self.render_sidebar(cx)); }
        main_row = main_row.child(self.render_editor(cx));
        if self.state.right_open { main_row = main_row.child(self.render_right_panel(cx)); }

        div().size_full().bg(t.bg_primary).flex().flex_col()
            .on_action(cx.listener(|this, _: &NewProject, _, _| {
                this.state.project = Project::default_prompt();
                this.state.block_inputs.clear();
            }))
            .on_action(cx.listener(|this, _: &ToggleTerminal, _, _| {
                this.state.right_tab = RightTab::Terminal;
                this.state.right_open = true;
            }))
            .on_action(cx.listener(|this, _: &RunPrompt, _, _| {
                this.state.right_tab = RightTab::Playground;
                this.state.right_open = true;
            }))
            .on_action(cx.listener(|this, _: &ToggleSettings, _, _| {
                this.state.show_settings = !this.state.show_settings;
            }))
            .on_action(cx.listener(|this, _: &Undo, _, _| {
                if let Some(prev_blocks) = this.state.undo_stack.pop() {
                    this.state.project.blocks = prev_blocks;
                    this.state.block_inputs.clear();
                }
            }))
            .on_action(cx.listener(|this, _: &SaveNow, _, _| {
                this.state.save_pending = true;
                this.state.save_timer = 1; // Save next frame
            }))
            .child(self.render_header(cx))
            .child(main_row)
            .children(if self.state.show_settings { Some(self.render_settings(cx)) } else { None })
            .child(self.render_bottom_bar(cx))
    }

    fn render_header(&self, cx: &mut Context<Self>) -> Div {
        div().h(px(40.0)).px(px(12.0)).flex().items_center().gap(px(8.0))
            .border_b_1().border_color(border_c()).bg(bg_secondary())
            // Toggle left sidebar
            .child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(if self.state.left_open { text_secondary() } else { text_muted() })
                    .child(if self.state.left_open { "[<]" } else { "[>]" })
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.left_open = !this.state.left_open;
                    }))
            )
            .child(div().text_sm().text_color(accent()).child("Inkwell"))
            .child(div().w(px(1.0)).h(px(16.0)).bg(border_c()))
            .child(div().text_sm().text_color(text_primary()).child(self.state.project.name.clone()))
            .child(div().flex_1())
            // Framework badge
            .children(self.state.project.framework.as_ref().map(|f| {
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0))
                    .bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.1))
                    .text_xs().text_color(accent()).child(f.clone())
            }))
            // Session info
            .children(self.state.session.as_ref().map(|s| {
                div().text_xs().text_color(text_muted()).child(s.email.clone())
            }))
            // Lang toggle
            .child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(text_muted())
                    .child(self.state.lang.to_uppercase())
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.lang = if this.state.lang == "fr" { "en".into() } else { "fr".into() };
                    }))
            )
            // Settings
            .child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(if self.state.show_settings { accent() } else { text_muted() })
                    .child("Settings")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.show_settings = !this.state.show_settings;
                    }))
            )
            // Theme toggle
            .child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(text_muted())
                    .child(if self.state.dark_mode { "Dark" } else { "Light" })
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.dark_mode = !this.state.dark_mode;
                    }))
            )
            .child(div().text_xs().text_color(success()).child("GPUI"))
            // Toggle right panel
            .child(
                div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                    .text_color(if self.state.right_open { text_secondary() } else { text_muted() })
                    .child(if self.state.right_open { "[>]" } else { "[<]" })
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
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
                            .child("Library")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.left_tab = LeftTab::Library; }))
                    )
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                            .text_color(if !is_library { accent() } else { text_muted() })
                            .bg(if !is_library { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) } else { hsla(0.0, 0.0, 0.0, 0.0) })
                            .child("Frameworks")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.left_tab = LeftTab::Frameworks; }))
                    )
            )
            // Content
            .child(if is_library { self.render_library(cx) } else { self.render_frameworks(cx) })
    }

    fn render_library(&self, cx: &mut Context<Self>) -> Div {
        let lang = &self.state.lang;
        let mut content = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(4.0));

        // Workspaces
        if !self.state.workspaces.is_empty() {
            for ws in &self.state.workspaces {
                let color = hex_to_hsla(&ws.color);
                content = content.child(
                    div().px(px(8.0)).py(px(6.0)).rounded(px(4.0))
                        .flex().items_center().gap(px(6.0))
                        .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(color))
                        .child(div().text_xs().text_color(text_primary()).child(ws.name.clone()))
                );
            }
            content = content.child(div().h(px(1.0)).bg(border_c()));
        }

        // New project button
        content = content.child(
            div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                .bg(bg_tertiary()).text_xs().text_color(accent())
                .flex().items_center().justify_center().child("+ New prompt")
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                    let new_proj = Project::default_prompt();
                    let name = new_proj.name.clone();
                    let id = new_proj.id.clone();
                    this.state.project = new_proj;
                    this.state.block_inputs.clear();

                    // Create on backend
                    let server = this.state.server_url.clone();
                    let token = this.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                    let blocks: Vec<inkwell_core::types::PromptBlock> = this.state.project.blocks.iter().map(|b| {
                        inkwell_core::types::PromptBlock {
                            id: b.id.clone(), block_type: b.block_type,
                            content: b.content.clone(), enabled: b.enabled,
                        }
                    }).collect();
                    this.state.projects.push(ProjectSummary { id: id.clone(), name: name.clone() });

                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            let mut client = inkwell_core::api_client::ApiClient::new(&server);
                            client.set_token(token);
                            let _ = client.create_project(&serde_json::json!({
                                "id": id, "name": name,
                                "blocks_json": serde_json::to_string(&blocks).unwrap_or_default(),
                            })).await;
                        });
                    });
                }))
        );

        // Project list
        for p in &self.state.projects {
            let id = p.id.clone();
            let is_active = self.state.project.id == p.id;
            content = content.child(
                div().px(px(10.0)).py(px(6.0)).rounded(px(4.0))
                    .text_xs().text_color(if is_active { text_primary() } else { text_secondary() })
                    .bg(if is_active { bg_tertiary() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                    .child(p.name.clone())
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                        // Switch to this project (data already loaded)
                        if let Some(p) = this.state.projects.iter().find(|p| p.id == id) {
                            this.state.project.id = p.id.clone();
                            this.state.project.name = p.name.clone();
                        }
                    }))
            );
        }

        if self.state.projects.is_empty() {
            content = content.child(div().text_xs().text_color(text_muted()).child("No projects yet"));
        }

        content
    }

    fn render_frameworks(&self, cx: &mut Context<Self>) -> Div {
        let frameworks = vec![
            ("CO-STAR", "co-star"), ("RISEN", "risen"), ("RACE", "race"),
            ("SDD (Spec-Driven)", "sdd"), ("STOKE", "stoke"),
        ];
        let mut content = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(4.0));

        // Save current as framework button
        content = content.child(
            div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(accent())
                .bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.1))
                .text_xs().text_color(accent())
                .flex().items_center().justify_center().child("Save current as framework")
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                    let name = format!("Custom {}", this.state.custom_frameworks.len() + 1);
                    let id = format!("custom-{}", uuid::Uuid::new_v4());
                    this.state.custom_frameworks.push((name, id));
                }))
        );
        content = content.child(div().h(px(1.0)).bg(border_c()));
        for (name, id) in frameworks {
            let id_str = id.to_string();
            let is_active = self.state.project.framework.as_deref() == Some(id);
            content = content.child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0))
                    .border_1().border_color(if is_active { accent() } else { border_c() })
                    .bg(if is_active { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) } else { bg_tertiary() })
                    .text_xs().text_color(if is_active { accent() } else { text_secondary() })
                    .child(name.to_string())
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                        this.state.project.framework = Some(id_str.clone());
                        this.apply_framework(&id_str.clone());
                    }))
            );
        }

        // Custom frameworks
        if !self.state.custom_frameworks.is_empty() {
            content = content.child(div().h(px(1.0)).bg(border_c()));
            content = content.child(div().text_xs().text_color(text_muted()).child("Custom"));
            for (name, _id) in &self.state.custom_frameworks {
                content = content.child(
                    div().px(px(10.0)).py(px(8.0)).rounded(px(6.0))
                        .border_1().border_color(border_c()).bg(bg_tertiary())
                        .text_xs().text_color(text_secondary())
                        .child(name.clone())
                );
            }
        }

        content
    }

    fn apply_framework(&mut self, id: &str) {
        // Save undo snapshot before replacing blocks
        self.state.undo_stack.push(self.state.project.blocks.clone());
        if self.state.undo_stack.len() > 50 { self.state.undo_stack.remove(0); }
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
                            .text_xs().text_color(hsla(0.0, 0.0, 1.0, 1.0))
                            .child(if self.state.sdd_running { "Running..." } else { "Generate all" })
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                if this.state.sdd_running { return; }
                                this.state.sdd_running = true;

                                let server = this.state.server_url.clone();
                                let tx = this.state.msg_tx.clone();
                                let blocks: Vec<(usize, BlockType)> = this.state.project.blocks.iter().enumerate()
                                    .filter(|(_, b)| b.block_type.is_sdd() && b.enabled)
                                    .map(|(i, b)| (i, b.block_type))
                                    .collect();

                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async {
                                        let client = reqwest::Client::new();
                                        let mut context = String::new();

                                        for (idx, bt) in &blocks {
                                            let prompt = if context.is_empty() {
                                                format!("Generate the {:?} for a new software project. Use Spec Kit SDD format.", bt)
                                            } else {
                                                format!("Based on:\n{}\n\nGenerate the {:?} phase.", context, bt)
                                            };

                                            if let Ok(resp) = client.post(format!("{server}/v1/chat/completions"))
                                                .json(&serde_json::json!({
                                                    "model": "qwen3.5:4b",
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
                                });
                            }))
                    )
                    .child(
                        div().px(px(8.0)).py(px(6.0)).text_xs().text_color(text_muted()).child("Validate")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                let blocks: Vec<inkwell_core::types::PromptBlock> = this.state.project.blocks.iter().map(|b| {
                                    inkwell_core::types::PromptBlock { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled }
                                }).collect();
                                let server = this.state.server_url.clone();
                                let tx = this.state.msg_tx.clone();
                                let mut all_content = String::new();
                                for b in &blocks { if b.enabled { all_content.push_str(&format!("\n### {:?}\n{}\n", b.block_type, b.content)); } }
                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async {
                                        let client = reqwest::Client::new();
                                        if let Ok(resp) = client.post(format!("{server}/v1/chat/completions"))
                                            .json(&serde_json::json!({"model":"qwen3.5:4b","messages":[
                                                {"role":"system","content":"Analyze cross-phase consistency. Check: coverage gaps, contradictions, underspecification, constitution alignment. Output: COHERENT/MISSING/CONTRADICTION/RECOMMENDATION items."},
                                                {"role":"user","content":format!("Validate these SDD phases:\n{all_content}")}
                                            ],"temperature":0.3,"max_tokens":4096,"stream":false})).send().await {
                                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                                let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                                let _ = tx.send(AsyncMsg::LlmResponse(format!("--- Validation ---\n{text}")));
                                            }
                                        }
                                    });
                                });
                                this.state.right_tab = RightTab::Playground;
                                this.state.right_open = true;
                            }))
                    )
                    // Checklist
                    .child(
                        div().px(px(8.0)).py(px(6.0)).text_xs().text_color(hsla(50.0 / 360.0, 0.8, 0.5, 1.0)).child("Checklist")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                let mut all_content = String::new();
                                for b in &this.state.project.blocks { if b.enabled { all_content.push_str(&format!("{}\n\n", b.content)); } }
                                let server = this.state.server_url.clone();
                                let tx = this.state.msg_tx.clone();
                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async {
                                        let client = reqwest::Client::new();
                                        if let Ok(resp) = client.post(format!("{server}/v1/chat/completions"))
                                            .json(&serde_json::json!({"model":"qwen3.5:4b","messages":[
                                                {"role":"system","content":"Generate a quality checklist (Unit Tests for English). Format: - [ ] CHK001 [Quality Dimension] Question. Validate requirements quality, not implementation."},
                                                {"role":"user","content":format!("Generate checklist for:\n{all_content}")}
                                            ],"temperature":0.3,"max_tokens":4096,"stream":false})).send().await {
                                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                                let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                                let _ = tx.send(AsyncMsg::LlmResponse(format!("--- Checklist ---\n{text}")));
                                            }
                                        }
                                    });
                                });
                                this.state.right_tab = RightTab::Playground;
                                this.state.right_open = true;
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
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
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
                let block_type_str = format!("{:?}", block.block_type);
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
                        .text_xs().text_color(accent()).child("Gen")
                        .on_mouse_down(MouseButton::Left, cx.listener(move |_this, _, _, _| {
                            let tx = tx1.clone();
                            let server = server1.clone();
                            let blocks = blocks1.clone();
                            let bt = bt1;
                            let idx = idx1;
                            std::thread::spawn(move || {
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                rt.block_on(async {
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
                                    if let Ok(resp) = client.post(format!("{server}/v1/chat/completions"))
                                        .json(&serde_json::json!({
                                            "model": "qwen3.5:4b",
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
                        .text_xs().text_color(hsla(280.0 / 360.0, 0.7, 0.6, 1.0)).child("AI")
                        .on_mouse_down(MouseButton::Left, cx.listener(move |_this, _, _, _| {
                            let tx = tx2.clone();
                            let server = server2.clone();
                            let content = current_content.clone();
                            let blocks = blocks2.clone();
                            let bt = bt2;
                            let idx = idx2;
                            std::thread::spawn(move || {
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                rt.block_on(async {
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
                                    if let Ok(resp) = client.post(format!("{server}/v1/chat/completions"))
                                        .json(&serde_json::json!({"model":"qwen3.5:4b","messages":[{"role":"system","content":"You improve SDD specifications. Keep the format strict."},{"role":"user","content":prompt}],"temperature":0.3,"max_tokens":4096,"stream":false}))
                                        .send().await {
                                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                                            let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                            let _ = tx.send(AsyncMsg::SddBlockResult { idx, content: text });
                                        }
                                    }
                                });
                            });
                        }))
                );

                // Clarify button
                let tx3 = self.state.msg_tx.clone();
                let server3 = self.state.server_url.clone();
                let content3 = block.content.clone();
                header = header.child(
                    div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(hsla(50.0 / 360.0, 0.8, 0.5, 1.0)).child("?")
                        .on_mouse_down(MouseButton::Left, cx.listener(move |_this, _, _, _| {
                            let tx = tx3.clone();
                            let server = server3.clone();
                            let content = content3.clone();
                            std::thread::spawn(move || {
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                rt.block_on(async {
                                    let prompt = format!("Analyze this specification and identify underspecified, ambiguous, or missing areas. Ask max 5 precise questions.\n\nContent:\n{content}");
                                    let client = reqwest::Client::new();
                                    if let Ok(resp) = client.post(format!("{server}/v1/chat/completions"))
                                        .json(&serde_json::json!({"model":"qwen3.5:4b","messages":[{"role":"system","content":"You are a technical reviewer. Identify underspecified areas."},{"role":"user","content":prompt}],"temperature":0.5,"max_tokens":2048,"stream":false}))
                                        .send().await {
                                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                                            let text = data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                                            let _ = tx.send(AsyncMsg::LlmResponse(format!("--- Clarify ---\n{text}")));
                                        }
                                    }
                                });
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
                        .child(if is_recording_this { "REC" } else { "Mic" })
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            if this.state.stt_recording {
                                // Stop recording
                                if let Some(stop_tx) = this.state.stt_stop_tx.take() {
                                    let _ = stop_tx.send(());
                                }
                                this.state.stt_recording = false;
                            } else {
                                // Start recording
                                this.state.stt_recording = true;
                                this.state.stt_target_block = Some(idx);
                                let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
                                this.state.stt_stop_tx = Some(stop_tx);
                                let tx = this.state.msg_tx.clone();
                                let server = this.state.server_url.clone();

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

                                            // Send to STT server
                                            let rt = tokio::runtime::Runtime::new().unwrap();
                                            rt.block_on(async {
                                                let client = reqwest::Client::new();
                                                if let Ok(resp) = client.post(format!("{server}/transcribe"))
                                                    .json(&serde_json::json!({"audio": b64, "language": "auto"}))
                                                    .send().await {
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
                        .child("^")
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
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
                        .child("v")
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
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
                        .child(if block.enabled { "on" } else { "off" })
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            if let Some(b) = this.state.project.blocks.get_mut(idx) { b.enabled = !b.enabled; }
                        }))
                )
                // Delete
                .child(
                    div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(danger()).child("x")
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            if idx < this.state.project.blocks.len() {
                                // Save undo snapshot
                                this.state.undo_stack.push(this.state.project.blocks.clone());
                                if this.state.undo_stack.len() > 50 { this.state.undo_stack.remove(0); }
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
            div().py(px(12.0)).flex().items_center().justify_center()
                .rounded(px(8.0)).border_1().border_color(border_c())
                .text_sm().text_color(text_muted()).child("+ Add block")
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
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            this.state.project.blocks.push(Block::new(bt));
                            this.state.show_add_menu = false;
                        }))
                );
            }
            block_list = block_list.child(menu);
        }

        // Variables panel
        let core_blocks: Vec<inkwell_core::types::PromptBlock> = self.state.project.blocks.iter().map(|b| {
            inkwell_core::types::PromptBlock { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled }
        }).collect();
        let vars = inkwell_core::prompt::extract_variables(&core_blocks);
        if !vars.is_empty() {
            let mut var_panel = div().p(px(12.0)).rounded(px(8.0)).bg(bg_secondary())
                .border_1().border_color(border_c()).flex().flex_col().gap(px(6.0))
                .child(div().text_xs().text_color(text_muted()).child("Variables"));
            for var in &vars {
                let val = self.state.project.variables.get(var).cloned().unwrap_or_default();
                let var_name = var.clone();
                var_panel = var_panel.child(
                    div().flex().items_center().gap(px(8.0))
                        .child(div().text_xs().text_color(accent()).child(format!("{{{{{var_name}}}}}")))
                        .child(
                            div().flex_1().h(px(28.0)).px(px(8.0)).bg(bg_tertiary())
                                .rounded(px(4.0)).border_1().border_color(border_c())
                                .flex().items_center().text_xs().text_color(text_secondary())
                                .child(if val.is_empty() { "click to set...".to_string() } else { val })
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                    // Simple prompt-based variable input (GPUI doesn't have modal text input easily)
                                    let new_val = format!("[{var_name}]");
                                    this.state.project.variables.insert(var_name.clone(), new_val);
                                }))
                        )
                );
            }
            block_list = block_list.child(var_panel);
        }

        div().flex_1().flex().flex_col().min_w_0().overflow_hidden()
            .child(div().flex_1().p(px(16.0)).flex().flex_col().gap(px(12.0)).child(block_list))
    }

    fn render_right_panel(&self, cx: &mut Context<Self>) -> Div {
        let tabs = vec![
            ("Preview", RightTab::Preview), ("Playground", RightTab::Playground),
            ("STT", RightTab::Stt), ("GPU", RightTab::Fleet), ("Terminal", RightTab::Terminal),
            ("Export", RightTab::Export), ("History", RightTab::History),
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
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                        this.state.right_tab = tab;
                    }))
            );
        }

        div().w(px(380.0)).flex_shrink_0().border_l_1().border_color(border_c()).bg(bg_secondary())
            .flex().flex_col()
            .child(tab_bar)
            .child(match self.state.right_tab {
                RightTab::Preview => self.render_preview(),
                RightTab::Playground => self.render_playground(cx),
                RightTab::Fleet => self.render_fleet(cx),
                RightTab::Export => self.render_export(cx),
                RightTab::History => self.render_history(cx),
                RightTab::Terminal => self.render_terminal(cx),
                RightTab::Stt => self.render_stt(),
                _ => div().flex_1().p(px(12.0)).child(div().text_xs().text_color(text_muted()).child("Coming soon...")),
            })
    }

    fn render_preview(&self) -> Div {
        let compiled = self.state.project.compiled_prompt();
        let lines = compiled.lines().count();
        let chars = compiled.len();
        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0))
            .child(
                div().flex().items_center().gap(px(8.0))
                    .child(div().text_xs().text_color(text_muted()).child("Compiled Prompt"))
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(text_muted()).child(format!("{lines} lines / {chars} chars")))
            )
            .child(
                div().flex_1().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary())
                    .border_1().border_color(border_c())
                    .text_xs().text_color(text_primary())
                    .child(if compiled.is_empty() { "No content yet...".into() } else { compiled })
            )
    }

    fn render_playground(&self, cx: &mut Context<Self>) -> Div {
        let models = vec![
            "gpt-4o-mini", "gpt-4o", "gpt-4.1", "claude-sonnet-4-6", "claude-opus-4-6",
            "gemini-2.5-pro", "gemini-2.5-flash",
        ];

        let mut model_list = div().flex().flex_col().gap(px(2.0));
        for model in &models {
            let model_str = model.to_string();
            let is_active = self.state.selected_model == *model;
            model_list = model_list.child(
                div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                    .text_xs().text_color(if is_active { accent() } else { text_secondary() })
                    .bg(if is_active { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) } else { hsla(0.0, 0.0, 0.0, 0.0) })
                    .child(model_str.clone())
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                        this.state.selected_model = model_str.clone();
                    }))
            );
        }

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().text_xs().text_color(text_muted()).child("Select Model"))
            .child(model_list)
            .child(div().h(px(1.0)).bg(border_c()))
            .child(
                div().py(px(10.0))
                    .bg(if self.state.playground_loading { text_muted() } else { accent() })
                    .rounded(px(8.0)).flex().items_center().justify_center()
                    .text_sm().text_color(hsla(0.0, 0.0, 1.0, 1.0))
                    .child(if self.state.playground_loading { "Running..." } else { "Run prompt" })
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        if this.state.playground_loading { return; }
                        this.state.playground_loading = true;
                        this.state.playground_response.clear();

                        let prompt = this.state.project.compiled_prompt();
                        let model = this.state.selected_model.clone();
                        let server_url = this.state.server_url.clone();
                        let tx = this.state.msg_tx.clone();

                        std::thread::spawn(move || {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(async {
                                let client = reqwest::Client::new();
                                let resp = client.post(format!("{server_url}/v1/chat/completions"))
                                    .json(&serde_json::json!({
                                        "model": model,
                                        "messages": [{"role": "user", "content": prompt}],
                                        "temperature": 0.7,
                                        "max_tokens": 2048,
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
                                                // Parse SSE lines
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
                                        if buffer.is_empty() {
                                            // Fallback: non-streaming response
                                            let _ = tx.send(AsyncMsg::LlmResponse("(empty response)".into()));
                                        }
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
    }

    fn render_fleet(&self, cx: &mut Context<Self>) -> Div {
        let mut content = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0));

        content = content.child(
            div().flex().items_center().gap(px(8.0))
                .child(div().text_xs().text_color(text_muted()).child("GPU Nodes"))
                .child(div().flex_1())
                .child(
                    div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                        .text_xs().text_color(text_muted()).child("Refresh")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            let server = this.state.server_url.clone();
                            let token = this.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                            let tx = this.state.msg_tx.clone();
                            std::thread::spawn(move || {
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                rt.block_on(async {
                                    let mut client = inkwell_core::api_client::ApiClient::new(&server);
                                    client.set_token(token);
                                    if let Ok(nodes) = client.list_nodes().await {
                                        let _ = tx.send(AsyncMsg::NodesLoaded(nodes));
                                    }
                                });
                            });
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
        let compiled = self.state.project.compiled_prompt();

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0))
            .child(div().text_xs().text_color(text_muted()).child("Export"))
            // Export Markdown
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child("Export Markdown (.md)")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
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
                    .child("Export JSON")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
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
                    .child("Export .specify/ (Spec Kit)")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
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
                        });
                    }))
            )
            // Copy to clipboard
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(accent())
                    .child("Copy to clipboard")
                    .on_mouse_down(MouseButton::Left, cx.listener(move |_this, _, _, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(compiled.clone()));
                    }))
            )
    }

    fn render_history(&self, cx: &mut Context<Self>) -> Div {
        let mut content = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0));

        content = content.child(
            div().flex().items_center().gap(px(8.0))
                .child(div().text_xs().text_color(text_muted()).child("Version History"))
                .child(div().flex_1())
                .child(
                    div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(accent())
                        .text_xs().text_color(hsla(0.0, 0.0, 1.0, 1.0)).child("Save version")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
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
                            let label = format!("v{}", chrono::Utc::now().format("%H:%M"));

                            std::thread::spawn(move || {
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                rt.block_on(async {
                                    let mut client = inkwell_core::api_client::ApiClient::new(&server);
                                    client.set_token(token.clone());
                                    let blocks_json = serde_json::to_string(&blocks).unwrap_or_default();
                                    let _ = client.create_version(&project_id, &blocks_json, "{}", &label).await;
                                    // Then reload versions
                                    if let Ok(versions) = client.list_versions(&project_id).await {
                                        let _ = tx.send(AsyncMsg::VersionsLoaded(versions));
                                    }
                                });
                            });
                        }))
                )
                .child(
                    div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                        .text_xs().text_color(text_muted()).child("Refresh")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                            let project_id = this.state.project.id.clone();
                            let server = this.state.server_url.clone();
                            let token = this.state.session.as_ref().map(|s| s.token.clone()).unwrap_or_default();
                            let tx = this.state.msg_tx.clone();
                            std::thread::spawn(move || {
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                rt.block_on(async {
                                    let mut client = inkwell_core::api_client::ApiClient::new(&server);
                                    client.set_token(token);
                                    if let Ok(versions) = client.list_versions(&project_id).await {
                                        let _ = tx.send(AsyncMsg::VersionsLoaded(versions));
                                    }
                                });
                            });
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
                        .child(
                            div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                                .text_xs().text_color(accent()).child("Restore")
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                                    // Parse blocks from version
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

        content
    }

    fn render_stt(&self) -> Div {
        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().text_xs().text_color(text_muted()).child("Speech-to-Text"))
            .child(
                div().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .flex().flex_col().gap(px(6.0))
                    .child(div().text_xs().text_color(text_primary()).child("How to use"))
                    .child(div().text_xs().text_color(text_secondary()).child(
                        "Click the 'Mic' button on any block header to start recording."
                    ))
                    .child(div().text_xs().text_color(text_secondary()).child(
                        "Click 'REC' again to stop. The transcription will be appended to the block."
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
                        format!("Server: {}", self.state.server_url)
                    ))
            )
            .child(
                div().text_xs().text_color(text_muted()).child(
                    "Requires a Whisper model loaded on a GPU server node."
                )
            )
    }

    fn render_settings(&self, cx: &mut Context<Self>) -> Div {
        let lang = self.state.lang.clone();
        div().h(px(200.0)).flex_shrink_0()
            .border_t_1().border_color(border_c()).bg(bg_secondary())
            .p(px(16.0)).flex().flex_col().gap(px(12.0))
            .child(
                div().flex().items_center().gap(px(8.0))
                    .child(div().text_sm().text_color(text_primary()).child("Settings"))
                    .child(div().flex_1())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                            .text_xs().text_color(text_muted()).child("Close")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
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
                                    .text_xs().text_color(if lang == "fr" { hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                                    .child("Francais")
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.lang = "fr".into(); }))
                            )
                            .child(
                                div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                                    .bg(if lang == "en" { accent() } else { bg_tertiary() })
                                    .text_xs().text_color(if lang == "en" { hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                                    .child("English")
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.lang = "en".into(); }))
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
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("API Keys"))
                        .child(div().text_xs().text_color(text_muted()).child(
                            format!("OpenAI: {} | Anthropic: {} | Google: {}",
                                if self.state.api_key_openai.is_empty() { "not set" } else { "set" },
                                if self.state.api_key_anthropic.is_empty() { "not set" } else { "set" },
                                if self.state.api_key_google.is_empty() { "not set" } else { "set" },
                            )
                        ))
                    )
            )
            .child(
                div().flex().gap(px(8.0))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+, to toggle settings"))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+N new project"))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+` terminal"))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+Enter run prompt"))
            )
    }

    fn render_terminal(&self, cx: &mut Context<Self>) -> Div {
        div().flex_1().flex().flex_col()
            .child(
                div().px(px(12.0)).py(px(6.0)).flex().items_center().gap(px(8.0))
                    .border_b_1().border_color(border_c())
                    .child(div().text_xs().text_color(text_muted()).child("Local Terminal"))
                    .child(div().flex_1())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(if self.state.terminal_running { danger() } else { success() })
                            .text_xs().text_color(hsla(0.0, 0.0, 1.0, 1.0))
                            .child(if self.state.terminal_running { "Stop" } else { "Start" })
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                if this.state.terminal_running {
                                    this.state.terminal_running = false;
                                } else {
                                    this.state.terminal_running = true;
                                    this.state.terminal_output = String::new();

                                    let tx = this.state.msg_tx.clone();
                                    let (input_tx, input_rx) = std::sync::mpsc::channel::<String>();
                                    this.state.terminal_input_tx = Some(input_tx);

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
                    .child(if self.state.terminal_output.is_empty() {
                        "Click Start to open a terminal session".to_string()
                    } else {
                        let lines: Vec<&str> = self.state.terminal_output.lines().collect();
                        let start = if lines.len() > 50 { lines.len() - 50 } else { 0 };
                        lines[start..].join("\n")
                    })
            )
            // Input area with real Input widget
            .child(
                div().h(px(36.0)).px(px(8.0)).bg(hsla(0.0, 0.0, 0.06, 1.0))
                    .border_t_1().border_color(border_c())
                    .flex().items_center().gap(px(6.0))
                    .child(div().text_xs().text_color(hsla(120.0 / 360.0, 0.8, 0.6, 1.0)).child("$"))
                    .child({
                        if let Some(ref entity) = self.state.terminal_input_entity {
                            div().flex_1().child(Input::new(entity))
                        } else {
                            div().flex_1().text_xs().text_color(text_muted()).child("...")
                        }
                    })
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(success())
                            .text_xs().text_color(hsla(0.0, 0.0, 0.0, 1.0)).child("Run")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                let cmd = if let Some(ref entity) = this.state.terminal_input_entity {
                                    let val = entity.read(cx).value().to_string();
                                    val
                                } else {
                                    this.state.terminal_input_buf.clone()
                                };
                                if !cmd.is_empty() {
                                    if let Some(ref tx) = this.state.terminal_input_tx {
                                        let _ = tx.send(format!("{cmd}\n"));
                                    }
                                    // Clear input
                                    this.state.terminal_input_entity = None; // Will be recreated next frame
                                }
                            }))
                    )
            )
    }

    fn render_bottom_bar(&self, cx: &mut Context<Self>) -> Div {
        let chars = self.state.project.char_count();
        let words = self.state.project.word_count();
        let tokens = self.state.project.token_count();
        let enabled = self.state.project.blocks.iter().filter(|b| b.enabled).count();
        let total = self.state.project.blocks.len();

        div().h(px(28.0)).px(px(12.0)).flex().items_center().gap(px(10.0))
            .border_t_1().border_color(border_c()).bg(bg_secondary())
            .child(div().text_xs().text_color(text_muted()).child(format!("{chars} chars")))
            .child(div().text_xs().text_color(text_muted()).child(format!("{words} words")))
            .child(div().text_xs().text_color(text_muted()).child(format!("~{tokens} tokens")))
            .child(div().w(px(1.0)).h(px(12.0)).bg(border_c()))
            .child(div().text_xs().text_color(text_muted()).child(format!("{enabled}/{total} blocks")))
            .child(div().flex_1())
            // Terminal button
            .child(
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0)).text_xs()
                    .text_color(text_muted()).child("Terminal")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                        this.state.right_tab = RightTab::Terminal;
                        this.state.right_open = true;
                    }))
            )
            .child(div().w(px(1.0)).h(px(12.0)).bg(border_c()))
            .child(div().text_xs().text_color(text_secondary()).child(self.state.selected_model.clone()))
    }
}

fn hex_to_hsla(hex: &str) -> Hsla {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128) as f32 / 255.0;
    let max = r.max(g).max(b); let min = r.min(g).min(b); let l = (max + min) / 2.0;
    if (max - min).abs() < 0.001 { return hsla(0.0, 0.0, l, 1.0); }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if (max - r).abs() < 0.001 { (g - b) / d + if g < b { 6.0 } else { 0.0 } }
        else if (max - g).abs() < 0.001 { (b - r) / d + 2.0 }
        else { (r - g) / d + 4.0 } / 6.0;
    hsla(h, s, l, 1.0)
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' { in_escape = true; continue; }
        if in_escape {
            if ch.is_ascii_alphabetic() { in_escape = false; }
            continue;
        }
        if ch == '\r' { continue; } // strip carriage returns
        result.push(ch);
    }
    result
}
