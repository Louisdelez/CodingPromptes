use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;

pub struct RightPanel {
    store: Entity<AppStore>,
    active_tab: RightTab,
    show_dropdown: bool,
    // Input entities for tabs
    chat_input: Option<Entity<InputState>>,
    terminal_input: Option<Entity<InputState>>,
    copy_feedback: u32,
}

impl RightPanel {
    pub fn new(store: Entity<AppStore>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let chat_input = Some(cx.new(|cx| InputState::new(window, cx).placeholder("Type a message...")));
        let terminal_input = Some(cx.new(|cx| InputState::new(window, cx).placeholder("Enter command...")));

        cx.subscribe(&store, |this, _, event: &StoreEvent, cx| {
            match event {
                StoreEvent::PlaygroundUpdated | StoreEvent::PromptCacheUpdated |
                StoreEvent::ChatMessageReceived | StoreEvent::TerminalOutput |
                StoreEvent::ProjectChanged => cx.notify(),
                StoreEvent::SwitchRightTab(tab) => {
                    this.active_tab = *tab;
                    cx.notify();
                }
                _ => {}
            }
        }).detach();

        Self {
            store, active_tab: RightTab::Preview,
            show_dropdown: false,
            chat_input, terminal_input, copy_feedback: 0,
        }
    }
}

impl Render for RightPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.copy_feedback > 0 { self.copy_feedback -= 1; }

        const TABS: &[(&str, RightTab, IconName)] = &[
            ("Preview", RightTab::Preview, IconName::File),
            ("Playground", RightTab::Playground, IconName::Play),
            ("Chat", RightTab::Chat, IconName::Bot),
            ("STT", RightTab::Stt, IconName::Mic),
            ("Optimize", RightTab::Optimize, IconName::Sparkles),
            ("Lint", RightTab::Lint, IconName::TriangleAlert),
            ("GPU", RightTab::Fleet, IconName::Settings),
            ("Terminal", RightTab::Terminal, IconName::SquareTerminal),
            ("Export", RightTab::Export, IconName::Download),
            ("Historique", RightTab::History, IconName::Redo),
            ("Stats", RightTab::Analytics, IconName::ChartPie),
            ("Chain", RightTab::Chain, IconName::Network),
            ("Collab", RightTab::Collab, IconName::User),
        ];

        // Find current tab label and icon
        let active = self.active_tab;
        let (tab_label, tab_icon) = TABS.iter()
            .find(|(_, t, _)| *t == active)
            .map(|(l, _, i)| (*l, i.clone()))
            .unwrap_or(("Preview", IconName::File));

        let show_dropdown = self.show_dropdown;

        // ── Dropdown header (like Tauri) ──
        let header = div().h(px(44.0)).px(px(16.0)).flex().items_center().gap(px(8.0))
            .border_b_1().border_color(border_c())
            .child(Icon::new(tab_icon).text_color(accent()))
            .child(div().flex_1().text_sm().font_weight(FontWeight::MEDIUM).text_color(text_primary()).child(tab_label))
            .child(
                div().text_color(text_muted())
                    .child(Icon::new(IconName::ChevronDown))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.show_dropdown = !this.show_dropdown; cx.notify();
                    }))
            );

        // ── Dropdown menu ──
        let dropdown = if show_dropdown {
            let mut menu = div().mx(px(8.0)).mt(px(4.0)).rounded(px(8.0))
                .bg(bg_tertiary()).border_1().border_color(border_c()).p(px(4.0))
                .flex().flex_col();
            for (label, tab, icon) in TABS {
                let tab = *tab;
                let icon = icon.clone();
                let is_active = self.active_tab == tab;
                menu = menu.child(
                    div().px(px(10.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                        .text_xs().text_color(if is_active { accent() } else { text_secondary() })
                        .bg(if is_active { accent_bg() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                        .hover(|s| s.bg(bg_secondary()))
                        .child(Icon::new(icon)).child(label.to_string())
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            this.active_tab = tab;
                            this.show_dropdown = false;
                            this.store.update(cx, |s, _| { s.right_tab = tab; });
                            cx.notify();
                        }))
                );
            }
            Some(menu)
        } else { None };

        let content = match self.active_tab {
            RightTab::Preview => self.render_preview(cx),
            RightTab::Playground => self.render_playground(cx),
            RightTab::Chat => self.render_chat(cx),
            RightTab::Lint => self.render_lint(cx),
            RightTab::Stt => self.render_stt(cx),
            RightTab::Analytics => self.render_analytics(cx),
            _ => self.render_placeholder(),
        };

        div().w(px(380.0)).flex_shrink_0().border_l_1().border_color(border_c()).bg(bg_secondary())
            .flex().flex_col()
            .child(header)
            .children(dropdown)
            .child(content)
    }
}

impl RightPanel {
    fn render_preview(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let compiled = s.cached_prompt.clone();
        let lines = s.cached_lines;
        let chars = s.cached_chars;
        let is_copied = self.copy_feedback > 0;
        drop(s);

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(8.0))
            .child(div().flex().items_center().gap(px(8.0))
                .child(Icon::new(IconName::Eye))
                .child(div().text_xs().text_color(text_muted()).child("Prompt compile"))
                .child(div().flex_1())
                .child({
                    let cc = compiled.clone();
                    div().px(px(6.0)).py(px(2.0)).rounded(px(4.0))
                        .text_xs().text_color(if is_copied { success() } else { accent() })
                        .flex().items_center().gap(px(4.0))
                        .child(Icon::new(if is_copied { IconName::Check } else { IconName::Copy }))
                        .child(if is_copied { "Copie !" } else { "Copier" })
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(cc.clone()));
                            this.copy_feedback = 120;
                        }))
                })
                .child(div().text_xs().text_color(text_muted()).child(format!("{lines} lines / {chars} chars")))
            )
            .child(div().flex_1().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                .text_xs().text_color(text_primary())
                .child(if compiled.is_empty() { "Commencez a ecrire dans les blocs...".to_string() } else { compiled }))
    }

    fn render_playground(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let response = s.playground_response.clone();
        let loading = s.playground_loading;
        let model = s.selected_model.clone();
        let tokens = s.cached_tokens;
        let last_exec = s.executions.last().cloned();
        drop(s);

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().text_xs().text_color(text_muted()).child(Icon::new(IconName::Bot)).child("Playground"))
            .child(div().text_xs().text_color(text_secondary()).child(format!("Model: {model}")))
            .child(
                div().py(px(10.0)).bg(if loading { text_muted() } else { accent() })
                    .rounded(px(8.0)).flex().items_center().justify_center()
                    .text_sm().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                    .child(if loading { "Running..." } else { "Run prompt" })
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.store.update(cx, |s, _| {
                            if s.playground_loading { return; }
                            s.playground_loading = true;
                            s.playground_response.clear();
                        });
                        let s = this.store.read(cx);
                        let prompt = s.cached_prompt.clone();
                        let model = s.selected_model.clone();
                        let server = s.server_url.clone();
                        let tx = s.msg_tx.clone();
                        let temp = s.playground_temperature;
                        let max_tok = s.playground_max_tokens;
                        drop(s);
                        std::thread::spawn(move || {
                            crate::app::rt().block_on(async {
                                let start = std::time::Instant::now();
                                let client = reqwest::Client::new();
                                let body = serde_json::json!({"model":model,"messages":[{"role":"user","content":prompt}],"temperature":temp,"max_tokens":max_tok,"stream":false});
                                if let Ok(resp) = crate::app::llm_post(&client, &model, &server, body).send().await {
                                    let latency = start.elapsed().as_millis() as u64;
                                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                                        let text = crate::llm::parse_llm_response(&model, &data).unwrap_or_default();
                                        let tokens_out = (text.len() as f64 / 4.0).ceil() as u64;
                                        let _ = tx.send(AsyncMsg::LlmResponse(text.clone()));
                                        let _ = tx.send(AsyncMsg::ExecutionRecorded(Execution {
                                            model: model.clone(), tokens_in: (prompt.len() as f64 / 4.0).ceil() as u64,
                                            tokens_out, latency_ms: latency, cost: 0.0,
                                            timestamp: chrono::Utc::now().timestamp_millis(),
                                            prompt_preview: prompt.chars().take(80).collect(),
                                            response_preview: text.chars().take(100).collect(),
                                        }));
                                    }
                                }
                                let _ = tx.send(AsyncMsg::LlmDone);
                            });
                        });
                    }))
            )
            .child(div().flex_1().p(px(12.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                .text_xs().text_color(if response.is_empty() { text_muted() } else { text_primary() })
                .child(if response.is_empty() { "Response will appear here...".to_string() } else { response }))
            .child(div().flex().items_center().gap(px(8.0))
                .child(div().text_xs().text_color(text_muted()).child(format!("~{tokens} tokens in")))
                .children(last_exec.map(|e| div().flex().items_center().gap(px(6.0))
                    .child(div().text_xs().text_color(accent()).child(format!("{}ms", e.latency_ms)))
                    .child(div().text_xs().text_color(success()).child(format!("{}/{} tok", e.tokens_in, e.tokens_out))))))
    }

    fn render_chat(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let messages: Vec<(String, String)> = s.chat_messages.clone();
        let server = s.server_url.clone();
        let tx = s.msg_tx.clone();
        drop(s);

        let mut msg_view = div().flex().flex_col().gap(px(6.0));
        for (role, content) in &messages {
            let is_user = role == "user";
            msg_view = msg_view.child(div().px(px(10.0)).py(px(6.0)).rounded(px(8.0))
                .bg(if is_user { bg_tertiary() } else { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) })
                .flex().flex_col().gap(px(2.0))
                .child(div().text_xs().text_color(if is_user { text_muted() } else { accent() }).child(if is_user { "You" } else { "Assistant" }))
                .child(div().text_xs().text_color(text_primary()).child(content.clone())));
        }

        div().flex_1().flex().flex_col()
            .child(div().px(px(12.0)).py(px(6.0)).flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::Bot)).child(div().text_xs().text_color(text_muted()).child("Conversation")))
            .child(div().flex_1().p(px(8.0)).child(msg_view)
                .child(if messages.is_empty() { div().text_xs().text_color(text_muted()).child("Start a conversation...") } else { div() }))
            .child(div().h(px(36.0)).px(px(8.0)).border_t_1().border_color(border_c()).flex().items_center().gap(px(6.0))
                .child(if let Some(ref entity) = self.chat_input { div().flex_1().child(Input::new(entity)) }
                    else { div().flex_1().text_xs().text_color(text_muted()).child("...") })
                .child(div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(accent()).child(Icon::new(IconName::Play))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        let msg = this.chat_input.as_ref().map(|e| e.read(cx).value().to_string()).unwrap_or_default();
                        if msg.is_empty() { return; }
                        this.store.update(cx, |s, _| { s.chat_messages.push(("user".into(), msg.clone())); });
                        this.chat_input = None;
                        cx.notify();
                        // LLM call
                        let messages: Vec<serde_json::Value> = this.store.read(cx).chat_messages.iter()
                            .map(|(r, c)| serde_json::json!({"role": r, "content": c})).collect();
                        let server = this.store.read(cx).server_url.clone();
                        let tx = this.store.read(cx).msg_tx.clone();
                        std::thread::spawn(move || {
                            crate::app::rt().block_on(async {
                                let client = reqwest::Client::new();
                                let body = serde_json::json!({"model":"gpt-4o-mini","messages":messages,"temperature":0.7,"max_tokens":2048,"stream":false});
                                if let Ok(resp) = crate::app::llm_post(&client, "gpt-4o-mini", &server, body).send().await {
                                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                                        let text = crate::llm::parse_llm_response("gpt-4o-mini", &data).unwrap_or_default();
                                        let _ = tx.send(AsyncMsg::LlmResponse(format!("__CHAT__{text}")));
                                    }
                                }
                            });
                        });
                    }))))
    }

    fn render_lint(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let blocks = &s.project.blocks;
        let enabled = blocks.iter().filter(|b| b.enabled).count();
        let empty = blocks.iter().filter(|b| b.enabled && b.content.trim().is_empty()).count();
        let has_task = blocks.iter().any(|b| b.enabled && b.block_type == inkwell_core::types::BlockType::Task);
        let compiled = &s.cached_prompt;
        let unresolved = compiled.matches("{{").count();
        let too_short = s.cached_chars < 50 && enabled > 0;
        let too_long = s.cached_chars > 10000;
        let has_negative = compiled.contains("don't") || compiled.contains("never") || compiled.contains("avoid");
        let has_examples = blocks.iter().any(|b| b.block_type == inkwell_core::types::BlockType::Examples && b.enabled);
        drop(s);

        let mut checks = div().flex().flex_col().gap(px(6.0));
        if enabled == 0 { checks = checks.child(lint_item("error", "No blocks enabled")); }
        if empty > 0 { checks = checks.child(lint_item("warning", &format!("{empty} empty block(s)"))); }
        if !has_task && enabled > 0 { checks = checks.child(lint_item("warning", "No task block")); }
        if unresolved > 0 { checks = checks.child(lint_item("warning", &format!("{unresolved} unresolved var(s)"))); }
        if too_short { checks = checks.child(lint_item("info", "Prompt very short")); }
        if too_long { checks = checks.child(lint_item("warning", "Prompt very long (>10K)")); }
        if has_negative { checks = checks.child(lint_item("info", "Negative instructions — consider positive framing")); }
        if !has_examples && s.cached_chars > 800 { checks = checks.child(lint_item("info", "No examples for complex prompt")); }
        if enabled > 0 && empty == 0 && has_task && unresolved == 0 && !too_short && !too_long { checks = checks.child(lint_item("success", "All checks passed!")); }

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::TriangleAlert)).child(div().text_xs().text_color(text_muted()).child("Linting")))
            .child(checks)
    }

    fn render_stt(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let recording = s.stt_recording;
        let server = s.server_url.clone();
        let provider = s.stt_provider;
        drop(s);

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(10.0))
            .child(div().text_xs().text_color(text_muted()).child(Icon::new(IconName::Mic)).child("Speech-to-Text"))
            .child(div().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                .flex().flex_col().gap(px(4.0))
                .child(div().flex().items_center().gap(px(6.0))
                    .child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0)).bg(if recording { danger() } else { success() }))
                    .child(div().text_xs().text_color(text_primary()).child(if recording { "Recording..." } else { "Ready" })))
                .child(div().text_xs().text_color(text_muted()).child(format!("Server: {server}"))))
    }

    fn render_analytics(&self, cx: &mut Context<Self>) -> Div {
        let s = self.store.read(cx);
        let exec_count = s.executions.len();
        let tokens = s.cached_tokens;
        let blocks = s.project.blocks.len();
        drop(s);

        div().flex_1().p(px(12.0)).flex().flex_col().gap(px(12.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(Icon::new(IconName::ChartPie)).child(div().text_xs().text_color(text_muted()).child("Analytics")))
            .child(div().flex().gap(px(8.0))
                .child(kpi_card("Executions", &exec_count.to_string(), accent()))
                .child(kpi_card("Tokens", &format!("~{tokens}"), success()))
                .child(kpi_card("Blocks", &blocks.to_string(), text_secondary())))
    }

    fn render_placeholder(&self) -> Div {
        div().flex_1().p(px(12.0)).flex().items_center().justify_center()
            .child(div().text_xs().text_color(text_muted()).child("This tab is available in the main app view."))
    }
}

fn lint_item(severity: &str, message: &str) -> Div {
    let color = match severity { "error" => danger(), "warning" => hsla(50.0/360.0, 0.8, 0.5, 1.0), "success" => success(), _ => text_muted() };
    div().px(px(10.0)).py(px(6.0)).rounded(px(6.0))
        .bg(hsla(color.h, color.s, color.l, 0.1)).border_1().border_color(hsla(color.h, color.s, color.l, 0.2))
        .flex().items_center().gap(px(8.0))
        .child(div().text_xs().text_color(color).child(message.to_string()))
}

fn kpi_card(label: &str, value: &str, color: Hsla) -> Div {
    div().flex_1().p(px(10.0)).rounded(px(8.0)).bg(bg_tertiary()).border_1().border_color(border_c())
        .flex().flex_col().items_center().gap(px(4.0))
        .child(div().text_xl().text_color(color).child(value.to_string()))
        .child(div().text_xs().text_color(text_muted()).child(label.to_string()))
}
