use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::ui::colors::*;
use inkwell_core::types::BlockType;

/// A single block editor — owns its InputState, only re-renders when THIS block changes.
pub struct BlockEditor {
    store: Entity<AppStore>,
    pub block_index: usize,
    input: Option<Entity<InputState>>,
    show_type_menu: bool,
}

impl BlockEditor {
    pub fn new(store: Entity<AppStore>, block_index: usize, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Create input for this block with FR placeholder
        let block_data = store.read(cx).project.blocks.get(block_index)
            .map(|b| (b.content.clone(), b.block_type));
        let (content, block_type) = block_data.unwrap_or((String::new(), inkwell_core::types::BlockType::Role));
        let placeholder = match block_type {
            BlockType::Role => "Tu es un expert en...",
            BlockType::Context => "Le contexte est...",
            BlockType::Task => "Ta tache est de...",
            BlockType::Examples => "Exemple:\nEntree: ...\nSortie: ...",
            BlockType::Constraints => "Ne fais pas..., Limite-toi a...",
            BlockType::Format => "Reponds en JSON / Markdown / ...",
            _ => "Ecris ici...",
        };
        let input = Some(cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(content)
                .placeholder(placeholder)
                .multi_line(true)
                .auto_grow(3, 30)
        }));

        // Subscribe to store — only re-render when OUR block changes
        cx.subscribe(&store, move |_this, _, event: &StoreEvent, cx| {
            match event {
                StoreEvent::BlockContentChanged(idx) if *idx == block_index => cx.notify(),
                StoreEvent::ProjectChanged => cx.notify(),
                _ => {}
            }
        }).detach();

        Self { store, block_index, input, show_type_menu: false }
    }

    /// Read current input value and sync to store if changed
    pub fn sync_content(&self, cx: &mut Context<Self>) -> bool {
        if let Some(ref input) = self.input {
            let val = input.read(cx).value();
            let store = self.store.read(cx);
            if let Some(block) = store.project.blocks.get(self.block_index) {
                if val != block.content.as_str() {
                    let new_content = val.to_string();
                    let idx = self.block_index;
                    self.store.update(cx, |s, _cx| {
                        if let Some(b) = s.project.blocks.get_mut(idx) {
                            b.content = new_content;
                        }
                        s.prompt_dirty = true;
                    });
                    return true;
                }
            }
        }
        false
    }

    /// Reset the input entity (e.g. after SDD generation fills content)
    pub fn reset_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.store.read(cx).project.blocks.get(self.block_index)
            .map(|b| b.content.clone()).unwrap_or_default();
        self.input = Some(cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(content)
                .multi_line(true)
                .auto_grow(3, 30)
        }));
    }
}

impl Render for BlockEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.read(cx);
        let idx = self.block_index;

        let Some(block) = store.project.blocks.get(idx) else {
            return div();
        };

        let color = hex_to_hsla(block.block_type.color());
        let label = block.block_type.label(&store.lang).to_string();
        let is_enabled = block.enabled;
        let is_sdd = block.block_type.is_sdd();
        let is_recording = store.stt_recording && store.stt_target_block == Some(idx);
        let lang = store.lang.clone();

        // Block type icon mapping (matching web)
        let type_icon = match block.block_type {
            BlockType::Role => IconName::CircleUser,
            BlockType::Context => IconName::BookOpen,
            BlockType::Task => IconName::Target,
            BlockType::Examples => IconName::ListChecks,
            BlockType::Constraints => IconName::Shield,
            BlockType::Format => IconName::LayoutDashboard,
            BlockType::SddConstitution => IconName::Scroll,
            BlockType::SddSpecification => IconName::FileCode,
            BlockType::SddPlan => IconName::Map,
            BlockType::SddTasks => IconName::ListChecks,
            BlockType::SddImplementation => IconName::PencilRuler,
        };

        let show_type = self.show_type_menu;

        // Block header (matches web: grip handle, color dot + type icon + label, spacer, actions)
        let mut header = div().px(px(8.0)).py(px(8.0)).flex().items_center().gap(px(6.0))
            .border_b_1().border_color(border_c())
            // Drag handle (6-dot grip) — starts drag for block reordering
            .child(div().id(("grip", idx)).text_color(text_muted()).cursor_pointer()
                .child(Icon::new(IconName::GripVertical))
                .on_drag(super::editor_pane::DragBlock {
                    block_index: idx,
                    label: label.clone(),
                    color,
                }, |drag, _, _, cx| cx.new(|_| drag.clone())))
            // Color dot + Type icon + label (clickable to change type)
            .child({
                let mut type_btn = div().flex().items_center().gap(px(6.0)).cursor_pointer()
                    .hover(|s| s.bg(bg_hover()).rounded(px(4.0)))
                    .px(px(4.0)).py(px(2.0)).rounded(px(4.0))
                    .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(color))
                    .child(Icon::new(type_icon).text_color(color))
                    .child(div().text_sm().text_color(color).child(label))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.show_type_menu = !this.show_type_menu; cx.notify();
                    }));
                // Type selector dropdown
                if show_type {
                    let all_types = [
                        BlockType::Role, BlockType::Context, BlockType::Task,
                        BlockType::Examples, BlockType::Constraints, BlockType::Format,
                        BlockType::SddConstitution, BlockType::SddSpecification,
                        BlockType::SddPlan, BlockType::SddTasks, BlockType::SddImplementation,
                    ];
                    let mut menu = div().mt(px(4.0)).w(px(220.0)).rounded(px(8.0))
                        .bg(bg_secondary()).border_1().border_color(border_c()).p(px(4.0))
                        .flex().flex_col().gap(px(2.0));
                    for bt in all_types {
                        let bt_label = bt.label(&lang).to_string();
                        let bt_color = hex_to_hsla(bt.color());
                        menu = menu.child(
                            div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                                .text_xs().text_color(text_primary()).hover(|s| s.bg(bg_hover()))
                                .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(bt_color))
                                .child(bt_label)
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                    this.store.update(cx, |s, cx| {
                                        if let Some(b) = s.project.blocks.get_mut(this.block_index) { b.block_type = bt; }
                                        s.prompt_dirty = true;
                                        cx.emit(StoreEvent::ProjectChanged);
                                    });
                                    this.show_type_menu = false; cx.notify();
                                }))
                        );
                    }
                    // Floating overlay
                    type_btn = type_btn.child(
                        deferred(anchored().snap_to_window_with_margin(px(8.0)).child(menu)).with_priority(1)
                    );
                }
                type_btn
            })
            .child(div().flex_1());

        // SDD action buttons — Generate, Validate, Improve
        if is_sdd {
            // Generate button (fills block from previous SDD blocks context)
            header = header.child(
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0)).flex().items_center().gap(px(2.0))
                    .text_xs().text_color(accent()).cursor_pointer()
                    .hover(|s| s.bg(accent_bg()))
                    .child(Icon::new(IconName::Sparkles))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        // Collect SDD context from all blocks and generate this one
                        let store = this.store.read(cx);
                        let blocks: Vec<(inkwell_core::types::BlockType, String)> = store.project.blocks.iter()
                            .filter(|b| b.enabled && b.block_type.is_sdd())
                            .map(|b| (b.block_type, b.content.clone()))
                            .collect();
                        let project_name = store.project.name.clone();
                        let model = store.selected_model.clone();
                        let server = store.server_url.clone();
                        let block_type = store.project.blocks.get(idx).map(|b| b.block_type);
                        let tx = store.msg_tx.clone();

                        if let Some(bt) = block_type {
                            if let Some(phase) = crate::spec::generator::block_type_to_phase(bt) {
                                let ctx = crate::spec::generator::SpecContext::from_blocks(&project_name, &blocks);
                                let (system, user) = crate::spec::workflow::build_llm_messages(
                                    phase, crate::spec::generator::SpecAction::Generate, &ctx
                                );
                                this.store.update(cx, |s, _| { s.sdd_running = true; });

                                std::thread::spawn(move || {
                                    crate::app::rt().block_on(async {
                                        let client = reqwest::Client::new();
                                        let body = serde_json::json!({
                                            "model": model,
                                            "messages": [
                                                {"role": "system", "content": system},
                                                {"role": "user", "content": user}
                                            ],
                                            "temperature": 0.3,
                                            "max_tokens": 4096,
                                            "stream": false
                                        });
                                        if let Ok(resp) = crate::app::llm_post(&client, &model, &server, body).send().await {
                                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                                let text = crate::llm::parse_llm_response(&model, &data).unwrap_or_default();
                                                let _ = tx.send(crate::types::AsyncMsg::SddBlockResult { idx, content: text });
                                            }
                                        }
                                        let _ = tx.send(crate::types::AsyncMsg::LlmDone);
                                    });
                                });
                            }
                        }
                    }))
            );
            // Validate button
            header = header.child(
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0))
                    .text_xs().text_color(success()).cursor_pointer()
                    .hover(|s| s.bg(accent_bg()))
                    .child(Icon::new(IconName::Check))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        let store = this.store.read(cx);
                        if let Some(block) = store.project.blocks.get(idx) {
                            let issues = match block.block_type {
                                BlockType::SddConstitution => crate::spec::validator::validate_constitution(&block.content),
                                BlockType::SddSpecification => crate::spec::validator::validate_specification(&block.content),
                                BlockType::SddPlan => crate::spec::validator::validate_plan(&block.content),
                                BlockType::SddTasks => crate::spec::validator::validate_tasks(&block.content),
                                _ => vec![],
                            };
                            let msg = if issues.is_empty() {
                                "Validation OK".to_string()
                            } else {
                                issues.iter().map(|i| format!("{:?}: {}", i.severity, i.message)).collect::<Vec<_>>().join("\n")
                            };
                            let _ = store.msg_tx.send(crate::types::AsyncMsg::LlmResponse(msg));
                        }
                    }))
            );
            // Improve button
            header = header.child(
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0))
                    .text_xs().text_color(text_muted()).cursor_pointer()
                    .hover(|s| s.bg(accent_bg()))
                    .child(Icon::new(IconName::Wand2))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        let store = this.store.read(cx);
                        let blocks: Vec<(inkwell_core::types::BlockType, String)> = store.project.blocks.iter()
                            .filter(|b| b.enabled && b.block_type.is_sdd())
                            .map(|b| (b.block_type, b.content.clone()))
                            .collect();
                        let project_name = store.project.name.clone();
                        let model = store.selected_model.clone();
                        let server = store.server_url.clone();
                        let block_type = store.project.blocks.get(idx).map(|b| b.block_type);
                        let tx = store.msg_tx.clone();

                        if let Some(bt) = block_type {
                            if let Some(phase) = crate::spec::generator::block_type_to_phase(bt) {
                                let ctx = crate::spec::generator::SpecContext::from_blocks(&project_name, &blocks);
                                let (system, user) = crate::spec::workflow::build_llm_messages(
                                    phase, crate::spec::generator::SpecAction::Improve, &ctx
                                );
                                this.store.update(cx, |s, _| { s.sdd_running = true; });

                                std::thread::spawn(move || {
                                    crate::app::rt().block_on(async {
                                        let client = reqwest::Client::new();
                                        let body = serde_json::json!({
                                            "model": model,
                                            "messages": [
                                                {"role": "system", "content": system},
                                                {"role": "user", "content": user}
                                            ],
                                            "temperature": 0.3,
                                            "max_tokens": 4096,
                                            "stream": false
                                        });
                                        if let Ok(resp) = crate::app::llm_post(&client, &model, &server, body).send().await {
                                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                                let text = crate::llm::parse_llm_response(&model, &data).unwrap_or_default();
                                                let _ = tx.send(crate::types::AsyncMsg::SddBlockResult { idx, content: text });
                                            }
                                        }
                                        let _ = tx.send(crate::types::AsyncMsg::LlmDone);
                                    });
                                });
                            }
                        }
                    }))
            );
            // Clarify button — generates clarifying questions
            header = header.child(
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0))
                    .text_xs().text_color(text_secondary()).cursor_pointer()
                    .hover(|s| s.bg(accent_bg()))
                    .child(Icon::new(IconName::CircleHelp))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        let store = this.store.read(cx);
                        let blocks: Vec<(inkwell_core::types::BlockType, String)> = store.project.blocks.iter()
                            .filter(|b| b.enabled && b.block_type.is_sdd())
                            .map(|b| (b.block_type, b.content.clone())).collect();
                        let project_name = store.project.name.clone();
                        let model = store.selected_model.clone();
                        let server = store.server_url.clone();
                        let block_type = store.project.blocks.get(idx).map(|b| b.block_type);
                        let tx = store.msg_tx.clone();
                        if let Some(bt) = block_type {
                            if let Some(phase) = crate::spec::generator::block_type_to_phase(bt) {
                                let ctx = crate::spec::generator::SpecContext::from_blocks(&project_name, &blocks);
                                let (system, user) = crate::spec::workflow::build_llm_messages(
                                    phase, crate::spec::generator::SpecAction::Clarify, &ctx);
                                std::thread::spawn(move || { crate::app::rt().block_on(async {
                                    let client = reqwest::Client::new();
                                    let body = serde_json::json!({"model":model,"messages":[
                                        {"role":"system","content":system},{"role":"user","content":user}
                                    ],"temperature":0.5,"max_tokens":2048,"stream":false});
                                    if let Ok(resp) = crate::app::llm_post(&client, &model, &server, body).send().await {
                                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                                            let text = crate::llm::parse_llm_response(&model, &data).unwrap_or_default();
                                            let _ = tx.send(crate::types::AsyncMsg::LlmResponse(format!("--- Questions de clarification ---\n{text}")));
                                        }
                                    }
                                }); });
                            }
                        }
                    }))
            );
        }

        // Mic (STT)
        header = header.child(
            div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                .text_xs().text_color(if is_recording { danger() } else { text_muted() })
                .child(Icon::new(if is_recording { IconName::Circle } else { IconName::Mic }))
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                    this.store.update(cx, |s, _cx| {
                        if s.stt_recording {
                            if let Some(stop_tx) = s.stt_stop_tx.take() { let _ = stop_tx.send(()); }
                            s.stt_recording = false;
                        } else {
                            s.stt_recording = true;
                            s.stt_target_block = Some(idx);
                            // Note: actual recording logic stays in app.rs for now (needs cpal thread)
                        }
                    });
                }))
        );

        // Toggle enabled
        header = header.child(
            div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                .text_xs().text_color(if is_enabled { success() } else { text_muted() })
                .child(Icon::new(if is_enabled { IconName::Eye } else { IconName::EyeOff }))
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                    this.store.update(cx, |s, cx| {
                        if let Some(b) = s.project.blocks.get_mut(idx) { b.enabled = !b.enabled; }
                        s.prompt_dirty = true;
                        cx.emit(StoreEvent::ProjectChanged);
                    });
                }))
        );

        // Delete — triggers confirmation modal in EditorPane
        header = header.child(
            div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                .text_xs().text_color(danger()).child(Icon::new(IconName::Trash2))
                .cursor_pointer().hover(|s| s.bg(bg_hover()))
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                    this.store.update(cx, |s, cx| {
                        s.confirm_delete_block = Some(idx);
                        cx.emit(StoreEvent::ProjectChanged);
                    });
                }))
        );

        // Block content — line number gutter + Input widget (matching web CodeMirror)
        let line_count = if let Some(ref input_entity) = self.input {
            let val = input_entity.read(cx).value();
            if val.is_empty() { 1 } else { val.matches('\n').count() + 1 }
        } else { 1 };

        let mut gutter = div().w(px(32.0)).flex_shrink_0().pt(px(6.0))
            .border_r_1().border_color(border_c());
        for n in 1..=line_count {
            // 20px per line matches the default line-height of text_sm
            gutter = gutter.child(
                div().h(px(20.0)).pr(px(8.0))
                    .flex().items_center().justify_end()
                    .text_xs().text_color(text_muted())
                    .child(format!("{n}"))
            );
        }

        let block_content = div().flex().min_h(px(60.0))
            .child(gutter)
            .child(if let Some(ref input_entity) = self.input {
                div().flex_1().child(Input::new(input_entity))
            } else {
                div().flex_1().p(px(8.0)).text_sm().text_color(text_secondary()).child("Click to edit...")
            });

        div().rounded(px(8.0))
            .border_1().border_color(border_c())
            .border_l_3().border_color(color)
            .bg(bg_secondary())
            .child(header)
            .child(block_content)
    }
}
