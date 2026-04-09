use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;
use inkwell_core::types::BlockType;

/// A single block editor — owns its InputState, only re-renders when THIS block changes.
pub struct BlockEditor {
    store: Entity<AppStore>,
    pub block_index: usize,
    input: Option<Entity<InputState>>,
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
            InputState::new(window, cx).default_value(content).placeholder(placeholder).multi_line(true).auto_grow(3, 20)
        }));

        // Subscribe to store — only re-render when OUR block changes
        cx.subscribe(&store, move |_this, _, event: &StoreEvent, cx| {
            match event {
                StoreEvent::BlockContentChanged(idx) if *idx == block_index => cx.notify(),
                StoreEvent::ProjectChanged => cx.notify(),
                _ => {}
            }
        }).detach();

        Self { store, block_index, input }
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
                    drop(store);
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
            InputState::new(window, cx).default_value(content).multi_line(true).auto_grow(3, 20)
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
        drop(store);

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

        // Block header (matches web: grip handle, color dot + type icon + label, spacer, actions)
        let mut header = div().px(px(8.0)).py(px(8.0)).flex().items_center().gap(px(6.0))
            .border_b_1().border_color(border_c())
            // Drag handle (6-dot grip)
            .child(div().text_color(text_muted()).cursor_pointer()
                .child(Icon::new(IconName::GripVertical)))
            // Color dot
            .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(color))
            // Type icon
            .child(Icon::new(type_icon).text_color(color))
            // Type label
            .child(div().text_sm().text_color(color).child(label))
            .child(div().flex_1());

        // SDD action buttons (generate/improve/clarify) — simplified in block, full version in EditorPane
        if is_sdd {
            header = header.child(
                div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                    .text_xs().text_color(accent()).child(Icon::new(IconName::Wand2))
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

        // Delete
        header = header.child(
            div().px(px(6.0)).py(px(2.0)).rounded(px(3.0))
                .text_xs().text_color(danger()).child(Icon::new(IconName::Trash2))
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                    this.store.update(cx, |s, cx| {
                        if idx < s.project.blocks.len() {
                            s.undo_stack.push_back(s.project.blocks.clone());
                            while s.undo_stack.len() > 50 { s.undo_stack.pop_front(); }
                            s.project.blocks.remove(idx);
                            s.prompt_dirty = true;
                            cx.emit(StoreEvent::ProjectChanged);
                        }
                    });
                }))
        );

        // Block content — the Input widget
        let block_content = if let Some(ref input_entity) = self.input {
            div().p(px(4.0)).min_h(px(60.0)).child(Input::new(input_entity))
        } else {
            div().p(px(4.0)).min_h(px(60.0)).text_sm().text_color(text_secondary()).child("Click to edit...")
        };

        div().rounded(px(8.0))
            .border_1().border_color(border_c())
            .border_l_3().border_color(color)
            .bg(bg_secondary()).overflow_hidden()
            .child(header)
            .child(block_content)
    }
}
