use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;

/// A single block editor — owns its InputState, only re-renders when THIS block changes.
pub struct BlockEditor {
    store: Entity<AppStore>,
    pub block_index: usize,
    input: Option<Entity<InputState>>,
}

impl BlockEditor {
    pub fn new(store: Entity<AppStore>, block_index: usize, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Create input for this block
        let content = store.read(cx).project.blocks.get(block_index)
            .map(|b| b.content.clone()).unwrap_or_default();
        let input = Some(cx.new(|cx| {
            InputState::new(window, cx).default_value(content).multi_line(true).auto_grow(3, 20)
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
        let block_count = store.project.blocks.len();
        let is_sdd = block.block_type.is_sdd();
        let is_recording = store.stt_recording && store.stt_target_block == Some(idx);
        drop(store);

        // Block header
        let mut header = div().px(px(12.0)).py(px(8.0)).flex().items_center().gap(px(8.0))
            .border_b_1().border_color(border_c())
            .child(div().w(px(3.0)).h(px(14.0)).rounded(px(2.0)).bg(color))
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

        // Move up
        header = header.child(
            div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                .text_xs().text_color(if idx > 0 { text_secondary() } else { hsla(0.0, 0.0, 0.2, 1.0) })
                .child(Icon::new(IconName::ChevronUp))
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                    if idx > 0 {
                        this.store.update(cx, |s, cx| {
                            s.project.blocks.swap(idx, idx - 1);
                            s.prompt_dirty = true;
                            cx.emit(StoreEvent::ProjectChanged);
                        });
                    }
                }))
        );

        // Move down
        header = header.child(
            div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                .text_xs().text_color(if idx < block_count - 1 { text_secondary() } else { hsla(0.0, 0.0, 0.2, 1.0) })
                .child(Icon::new(IconName::ChevronDown))
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                    let len = this.store.read(cx).project.blocks.len();
                    if idx + 1 < len {
                        this.store.update(cx, |s, cx| {
                            s.project.blocks.swap(idx, idx + 1);
                            s.prompt_dirty = true;
                            cx.emit(StoreEvent::ProjectChanged);
                        });
                    }
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
                .text_xs().text_color(danger()).child("x")
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
            .bg(bg_secondary()).overflow_hidden()
            .child(header)
            .child(block_content)
    }
}
