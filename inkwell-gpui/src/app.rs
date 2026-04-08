use gpui::*;
use crate::state::*;
use inkwell_core::types::BlockType;

fn bg_primary() -> Hsla { hsla(230.0 / 360.0, 0.15, 0.07, 1.0) }
fn bg_secondary() -> Hsla { hsla(230.0 / 360.0, 0.12, 0.10, 1.0) }
fn bg_tertiary() -> Hsla { hsla(230.0 / 360.0, 0.10, 0.14, 1.0) }
fn border_c() -> Hsla { hsla(230.0 / 360.0, 0.10, 0.20, 1.0) }
fn text_primary() -> Hsla { hsla(0.0, 0.0, 0.95, 1.0) }
fn text_secondary() -> Hsla { hsla(0.0, 0.0, 0.70, 1.0) }
fn text_muted() -> Hsla { hsla(0.0, 0.0, 0.50, 1.0) }
fn accent() -> Hsla { hsla(239.0 / 360.0, 0.84, 0.67, 1.0) }
fn danger() -> Hsla { hsla(0.0, 0.75, 0.55, 1.0) }
fn success() -> Hsla { hsla(150.0 / 360.0, 0.65, 0.45, 1.0) }

pub struct InkwellApp {
    pub state: AppState,
}

impl InkwellApp {
    pub fn new() -> Self {
        Self { state: AppState::new() }
    }
}

impl Render for InkwellApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_messages();

        match self.state.screen {
            Screen::Auth => self.render_auth(window, cx),
            Screen::Ide => self.render_ide(cx),
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
                AsyncMsg::LlmDone => {
                    self.state.playground_loading = false;
                }
                AsyncMsg::LlmError(e) => {
                    self.state.playground_loading = false;
                    self.state.playground_response = format!("Error: {e}");
                }
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
    fn render_auth(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> Div {
        div()
            .size_full().bg(bg_primary()).flex().items_center().justify_center()
            .child(
                div().w(px(400.0)).p(px(32.0)).bg(bg_secondary()).rounded(px(16.0))
                    .border_1().border_color(border_c()).flex().flex_col().gap(px(16.0))
                    .child(div().flex().flex_col().items_center().gap(px(8.0))
                        .child(div().text_xl().text_color(text_primary()).child("Inkwell"))
                        .child(div().text_sm().text_color(text_muted()).child("GPU-Accelerated Prompt IDE"))
                    )
                    // Server URL
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("Server"))
                        .child(div().h(px(32.0)).px(px(10.0)).bg(bg_tertiary()).rounded(px(6.0))
                            .border_1().border_color(border_c()).flex().items_center()
                            .text_sm().text_color(text_secondary()).child(self.state.server_url.clone()))
                    )
                    // Email
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("Email"))
                        .child(div().h(px(32.0)).px(px(10.0)).bg(bg_tertiary()).rounded(px(6.0))
                            .border_1().border_color(border_c()).flex().items_center()
                            .text_sm().text_color(text_muted()).child(
                                if self.state.email.is_empty() { "email@example.com".to_string() } else { self.state.email.clone() }
                            ))
                    )
                    // Password
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("Password"))
                        .child(div().h(px(32.0)).px(px(10.0)).bg(bg_tertiary()).rounded(px(6.0))
                            .border_1().border_color(border_c()).flex().items_center()
                            .text_sm().text_color(text_muted()).child("••••••••"))
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
                            .child(if self.state.auth_loading { "Connecting..." } else { "Sign in" })
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                if this.state.auth_loading { return; }
                                this.state.auth_loading = true;
                                this.state.auth_error = None;

                                let server_url = this.state.server_url.clone();
                                let email = this.state.email.clone();
                                let password = this.state.password.clone();
                                let tx = this.state.msg_tx.clone();

                                // Spawn auth in background thread with tokio
                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async {
                                        let mut client = inkwell_core::api_client::ApiClient::new(&server_url);
                                        match client.login(&email, &password).await {
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

    fn render_ide(&mut self, cx: &mut Context<Self>) -> Div {
        let mut main_row = div().flex_1().flex().overflow_hidden();
        if self.state.left_open { main_row = main_row.child(self.render_sidebar(cx)); }
        main_row = main_row.child(self.render_editor(cx));
        if self.state.right_open { main_row = main_row.child(self.render_right_panel(cx)); }

        div().size_full().bg(bg_primary()).flex().flex_col()
            .child(self.render_header(cx))
            .child(main_row)
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
        let mut content = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(4.0));

        // New project button
        content = content.child(
            div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                .bg(bg_tertiary()).text_xs().text_color(accent())
                .flex().items_center().justify_center().child("+ New prompt")
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                    this.state.project = Project::default_prompt();
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
        content
    }

    fn apply_framework(&mut self, id: &str) {
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
                    .child(div().px(px(12.0)).py(px(6.0)).rounded(px(4.0)).bg(accent())
                        .text_xs().text_color(hsla(0.0, 0.0, 1.0, 1.0)).child("Generate all"))
                    .child(div().px(px(8.0)).py(px(6.0)).text_xs().text_color(text_muted()).child("Validate"))
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
                header = header
                    .child(div().px(px(6.0)).py(px(2.0)).rounded(px(3.0)).text_xs().text_color(text_muted()).child("Gen"))
                    .child(div().px(px(6.0)).py(px(2.0)).rounded(px(3.0)).text_xs().text_color(text_muted()).child("AI"))
                    .child(div().px(px(6.0)).py(px(2.0)).rounded(px(3.0)).text_xs().text_color(text_muted()).child("?"));
            }

            header = header
                .child(
                    div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(if block.enabled { success() } else { text_muted() })
                        .child(if block.enabled { "on" } else { "off" })
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            if let Some(b) = this.state.project.blocks.get_mut(idx) { b.enabled = !b.enabled; }
                        }))
                )
                .child(
                    div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                        .text_xs().text_color(danger()).child("x")
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            if idx < this.state.project.blocks.len() { this.state.project.blocks.remove(idx); }
                        }))
                );

            let is_editing = self.state.editing_block_idx == Some(idx);
            let block_div = div().rounded(px(8.0))
                .border_1().border_color(if is_editing { accent() } else { border_c() })
                .bg(bg_secondary()).overflow_hidden()
                .child(header)
                .child(
                    div().p(px(12.0)).min_h(px(60.0)).text_sm()
                        .text_color(if is_editing { text_primary() } else { text_secondary() })
                        .bg(if is_editing { bg_tertiary() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                        .child(content.to_string())
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, _| {
                            this.state.editing_block_idx = Some(idx);
                        }))
                );

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
                RightTab::Fleet => self.render_fleet(),
                RightTab::Export => self.render_export(),
                RightTab::History => self.render_history(),
                RightTab::Terminal => self.render_terminal(cx),
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
                                // Call local Ollama via the server
                                let client = reqwest::Client::new();
                                let resp = client.post(format!("{server_url}/v1/chat/completions"))
                                    .json(&serde_json::json!({
                                        "model": model,
                                        "messages": [{"role": "user", "content": prompt}],
                                        "temperature": 0.7,
                                        "max_tokens": 2048,
                                        "stream": false,
                                    }))
                                    .send().await;

                                match resp {
                                    Ok(r) if r.status().is_success() => {
                                        if let Ok(data) = r.json::<serde_json::Value>().await {
                                            let text = data["choices"][0]["message"]["content"]
                                                .as_str().unwrap_or("No response").to_string();
                                            let _ = tx.send(AsyncMsg::LlmResponse(text));
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

    fn render_fleet(&self) -> Div {
        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0))
            .child(div().text_xs().text_color(text_muted()).child("GPU Nodes"))
            .child(
                div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                    .flex().flex_col().gap(px(4.0))
                    .child(div().flex().items_center().gap(px(6.0))
                        .child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0)).bg(success()))
                        .child(div().text_xs().text_color(text_primary()).child("Local server"))
                    )
                    .child(div().text_xs().text_color(text_muted()).child(self.state.server_url.clone()))
            )
            .child(
                div().text_xs().text_color(text_muted()).child("Connect more GPU servers in the GPU Server app")
            )
    }

    fn render_export(&self) -> Div {
        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0))
            .child(div().text_xs().text_color(text_muted()).child("Export"))
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child("Export .specify/ (Spec Kit compatible)")
            )
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child("Export JSON")
            )
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child("Export Markdown")
            )
            .child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
                    .bg(bg_tertiary()).text_xs().text_color(text_secondary())
                    .child("Copy to clipboard")
            )
    }

    fn render_history(&self) -> Div {
        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0))
            .child(div().text_xs().text_color(text_muted()).child("Execution History"))
            .child(div().text_xs().text_color(text_muted()).child("No executions yet. Run a prompt in the Playground."))
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
                                    this.state.terminal_output = "$ ".to_string();

                                    let tx = this.state.msg_tx.clone();
                                    std::thread::spawn(move || {
                                        use std::io::Read;
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
                                        let mut buf = [0u8; 4096];
                                        loop {
                                            match reader.read(&mut buf) {
                                                Ok(0) => break,
                                                Ok(n) => {
                                                    let text = String::from_utf8_lossy(&buf[..n]).to_string();
                                                    // Strip ANSI escape codes for simple display
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
            .child(
                div().flex_1().p(px(8.0)).bg(hsla(0.0, 0.0, 0.04, 1.0))
                    .text_xs().text_color(hsla(120.0 / 360.0, 0.8, 0.6, 1.0))
                    .child(if self.state.terminal_output.is_empty() {
                        "Click Start to open a terminal session".to_string()
                    } else {
                        // Show last 50 lines
                        let lines: Vec<&str> = self.state.terminal_output.lines().collect();
                        let start = if lines.len() > 50 { lines.len() - 50 } else { 0 };
                        lines[start..].join("\n")
                    })
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
