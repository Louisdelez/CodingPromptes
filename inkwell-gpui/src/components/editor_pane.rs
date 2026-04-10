use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;
use inkwell_core::types::BlockType;
use super::block_editor::BlockEditor;

/// Drag payload for block reordering
#[derive(Clone)]
pub struct DragBlock {
    pub block_index: usize,
    pub label: String,
    pub color: Hsla,
}

impl Render for DragBlock {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(200.0)).h(px(36.0)).rounded(px(8.0))
            .bg(bg_tertiary()).border_l_3().border_color(self.color).opacity(0.85)
            .px(px(12.0)).flex().items_center().gap(px(6.0))
            .child(Icon::new(IconName::GripVertical).text_color(text_muted()))
            .child(div().text_xs().text_color(text_primary()).child(self.label.clone()))
    }
}

/// The main editor pane — owns a Vec<Entity<BlockEditor>> for independent block rendering.
pub struct EditorPane {
    store: Entity<AppStore>,
    block_editors: Vec<Entity<BlockEditor>>,
    tag_input: Option<Entity<InputState>>,
    variable_inputs: std::collections::HashMap<String, Entity<InputState>>,
    show_add_menu: bool,
    last_block_count: usize,
    confirm_delete_block: Option<usize>,
}

impl EditorPane {
    pub fn new(store: Entity<AppStore>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let block_count = store.read(cx).project.blocks.len();
        let mut block_editors = Vec::new();
        for i in 0..block_count {
            let s = store.clone();
            let editor = cx.new(|cx| BlockEditor::new(s, i, window, cx));
            block_editors.push(editor);
        }

        let tag_input = Some(cx.new(|cx| InputState::new(window, cx).placeholder("Ajouter un tag...")));

        cx.subscribe(&store, |this: &mut Self, _, event: &StoreEvent, cx| {
            match event {
                StoreEvent::ProjectChanged => {
                    // Block count may have changed — need to rebuild editors
                    cx.notify();
                }
                StoreEvent::PromptCacheUpdated => {
                    // Variables may have changed
                    cx.notify();
                }
                _ => {}
            }
        }).detach();

        Self {
            store, block_editors, tag_input,
            variable_inputs: std::collections::HashMap::new(),
            show_add_menu: false, confirm_delete_block: None,
            last_block_count: block_count,
        }
    }

    /// Sync block editors with current block count/order
    fn sync_editors(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let store = self.store.read(cx);
        let count = store.project.blocks.len();
        // Check if block IDs match current editors (detects reordering)
        let ids_match = count == self.block_editors.len() && self.block_editors.iter().enumerate().all(|(i, e)| {
            e.read(cx).block_index == i
        });
        drop(store);
        if count == self.last_block_count && ids_match { return; }

        // Rebuild all editors (blocks were added/removed/reordered)
        self.block_editors.clear();
        for i in 0..count {
            let s = self.store.clone();
            let editor = cx.new(|cx| BlockEditor::new(s, i, window, cx));
            self.block_editors.push(editor);
        }
        self.last_block_count = count;
    }

    /// Sync all block editor inputs to store
    pub fn sync_content(&self, cx: &mut Context<Self>) -> bool {
        let mut changed = false;
        for editor in &self.block_editors {
            if editor.update(cx, |e, cx| e.sync_content(cx)) {
                changed = true;
            }
        }
        // Sync variable inputs
        let var_keys: Vec<String> = self.variable_inputs.keys().cloned().collect();
        for var_name in var_keys {
            if let Some(entity) = self.variable_inputs.get(&var_name) {
                let val = entity.read(cx).value();
                let store = self.store.read(cx);
                let old = store.project.variables.get(&var_name).map(|s| s.as_str()).unwrap_or("");
                if val != old && !val.is_empty() {
                    let new_val = val.to_string();
                    drop(store);
                    self.store.update(cx, |s, _| {
                        s.project.variables.insert(var_name.clone(), new_val);
                        s.prompt_dirty = true;
                    });
                    changed = true;
                }
            }
        }
        changed
    }
}

impl Render for EditorPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_editors(window, cx);

        let store = self.store.read(cx);
        let cached_vars = store.cached_vars.clone();
        let tags = store.project.tags.clone();
        let lang = store.lang.clone();
        drop(store);

        let mut block_list = div().flex().flex_col().gap(px(12.0));

        // Block editors — each is an independent Entity, wrapped in drag-drop targets
        for (i, editor) in self.block_editors.iter().enumerate() {
            // Read block info for drag preview
            let store = self.store.read(cx);
            let (label, color) = store.project.blocks.get(i)
                .map(|b| (b.block_type.label(&store.lang).to_string(), hex_to_hsla(b.block_type.color())))
                .unwrap_or(("Block".into(), text_muted()));
            drop(store);

            block_list = block_list.child(
                div().id(("block-drop", i))
                    .drag_over::<DragBlock>(|this, _, _, _cx| {
                        this.border_t_2().border_color(accent())
                    })
                    .on_drop(cx.listener(move |this, drag: &DragBlock, _window, cx| {
                        let from = drag.block_index;
                        let to = i;
                        if from != to {
                            this.store.update(cx, |s, cx| {
                                let block = s.project.blocks.remove(from);
                                let insert_at = if from < to { to.saturating_sub(1) } else { to };
                                s.project.blocks.insert(insert_at, block);
                                s.prompt_dirty = true;
                                cx.emit(StoreEvent::ProjectChanged);
                            });
                        }
                    }))
                    .child(editor.clone())
            );
        }

        // Add block button (dashed border style like web)
        block_list = block_list.child(
            div().py(px(12.0)).flex().items_center().justify_center().gap(px(6.0))
                .rounded(px(8.0)).border_2().border_color(border_c())
                .text_sm().text_color(text_muted())
                .child(Icon::new(IconName::Plus))
                .child("Ajouter un bloc")
                .child(Icon::new(IconName::ChevronDown))
                .cursor_pointer()
                .hover(|s| s.border_color(accent()).text_color(accent()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.show_add_menu = !this.show_add_menu;
                    cx.notify();
                }))
        );

        // Add block menu
        if self.show_add_menu {
            let all_types = vec![
                BlockType::Role, BlockType::Context, BlockType::Task,
                BlockType::Examples, BlockType::Constraints, BlockType::Format,
                BlockType::SddConstitution, BlockType::SddSpecification,
                BlockType::SddPlan, BlockType::SddTasks, BlockType::SddImplementation,
            ];
            let mut menu = div().p(px(12.0)).rounded(px(12.0)).bg(bg_secondary())
                .border_1().border_color(border_c()).flex().flex_wrap().gap(px(4.0));
            for bt in all_types {
                let label = bt.label(&lang).to_string();
                let color = hex_to_hsla(bt.color());
                menu = menu.child(
                    div().w(px(240.0)).px(px(12.0)).py(px(10.0)).rounded(px(6.0)).flex().items_center().gap(px(8.0))
                        .text_sm().text_color(text_primary())
                        .hover(|s| s.bg(bg_hover()))
                        .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(color))
                        .child(label)
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            this.store.update(cx, |s, cx| {
                                s.project.blocks.push(Block::new(bt));
                                s.prompt_dirty = true;
                                cx.emit(StoreEvent::ProjectChanged);
                            });
                            this.show_add_menu = false;
                            cx.notify();
                        }))
                );
            }
            block_list = block_list.child(menu);
        }

        // Variable hint
        block_list = block_list.child(
            div().px(px(12.0)).py(px(10.0)).rounded(px(8.0))
                .bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.08))
                .border_1().border_color(hsla(239.0 / 360.0, 0.84, 0.67, 0.15))
                .flex().items_center()
                .child(div().text_xs().text_color(text_muted()).child("Utilisez "))
                .child(div().px(px(4.0)).py(px(1.0)).rounded(px(3.0)).bg(accent_bg())
                    .text_xs().text_color(accent()).child("{{variable}}"))
                .child(div().text_xs().text_color(text_muted()).child(" dans vos blocs pour creer des variables."))
        );

        // Variables panel
        if !cached_vars.is_empty() {
            let mut var_panel = div().p(px(12.0)).rounded(px(8.0)).bg(bg_secondary())
                .border_1().border_color(border_c()).flex().flex_col().gap(px(6.0))
                .child(div().text_xs().text_color(text_muted()).child(Icon::new(IconName::Asterisk)).child("Variables"));
            for var in &cached_vars {
                let input_entity = self.variable_inputs.get(var).cloned();
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

            // Ensure variable inputs exist
            for var in &cached_vars {
                if !self.variable_inputs.contains_key(var) {
                    let val = self.store.read(cx).project.variables.get(var).cloned().unwrap_or_default();
                    let entity = cx.new(|cx| {
                        InputState::new(window, cx).placeholder(format!("value for {var}")).default_value(val)
                    });
                    self.variable_inputs.insert(var.clone(), entity);
                }
            }
            self.variable_inputs.retain(|k, _| cached_vars.contains(k));
        }

        // Tags
        let mut tags_row = div().flex().flex_wrap().gap(px(4.0));
        for tag in &tags {
            let tag_name = tag.clone();
            tags_row = tags_row.child(
                div().px(px(8.0)).py(px(3.0)).rounded(px(12.0))
                    .bg(accent_bg()).text_xs().text_color(accent())
                    .flex().items_center().gap(px(4.0))
                    .child(tag.clone())
                    .child(div().text_xs().text_color(text_muted()).child("x")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            this.store.update(cx, |s, cx| {
                                s.project.tags.retain(|t| t != &tag_name);
                                cx.emit(StoreEvent::ProjectChanged);
                            });
                        })))
            );
        }
        if let Some(ref entity) = self.tag_input {
            tags_row = tags_row.child(
                div().flex().items_center().gap(px(4.0))
                    .child(div().w(px(100.0)).child(Input::new(entity)))
                    .child(div().px(px(8.0)).py(px(3.0)).rounded(px(12.0))
                        .bg(accent()).text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)).child("+")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            let name = this.tag_input.as_ref()
                                .map(|i| i.read(cx).value().to_string()).unwrap_or_default();
                            let tag = if name.trim().is_empty() { format!("tag-{}", this.store.read(cx).project.tags.len() + 1) }
                                else { name.trim().to_string() };
                            this.store.update(cx, |s, cx| {
                                s.project.tags.push(tag);
                                cx.emit(StoreEvent::ProjectChanged);
                            });
                            this.tag_input = None;
                            cx.notify();
                        })))
            );
        }
        block_list = block_list.child(
            div().p(px(8.0)).rounded(px(8.0)).bg(bg_secondary()).border_1().border_color(border_c())
                .flex().flex_col().gap(px(4.0))
                .child(div().flex().items_center().gap(px(4.0))
                    .child(Icon::new(IconName::Tag).text_color(text_muted()))
                    .child(div().text_xs().text_color(text_muted()).child("Tags")))
                .child(tags_row)
        );

        // Terminal at bottom (when open)
        let terminal_open = self.store.read(cx).terminal_open;
        let terminal_output = if terminal_open {
            let s = self.store.read(cx);
            let output = s.terminal_sessions.get(s.active_terminal).map(|t| t.output.clone()).unwrap_or_default();
            drop(s);
            Some(div().h(px(200.0)).flex_shrink_0().border_t_1().border_color(border_c())
                .bg(hsla(0.0, 0.0, 0.04, 1.0))
                .p(px(8.0)).text_xs()
                .text_color(hsla(120.0 / 360.0, 0.8, 0.6, 1.0))
                .child(if output.is_empty() { "Terminal pret. Utilisez le serveur pour executer des commandes.".into() } else {
                    let lines: Vec<&str> = output.lines().collect();
                    let start = if lines.len() > 30 { lines.len() - 30 } else { 0 };
                    lines[start..].join("\n")
                }))
        } else { None };

        // Delete block confirmation modal
        let confirm_del = self.store.read(cx).confirm_delete_block;
        let delete_modal = confirm_del.map(|block_idx| {
            let block_label = self.store.read(cx).project.blocks.get(block_idx)
                .map(|b| b.block_type.label(&self.store.read(cx).lang).to_string())
                .unwrap_or("bloc".into());
            div().id("delete-modal-backdrop").size_full().absolute().top_0().left_0()
                .bg(hsla(0.0, 0.0, 0.0, 0.4))
                .flex().items_center().justify_center()
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| { s.confirm_delete_block = None; cx.emit(StoreEvent::ProjectChanged); });
                }))
                .child(div().w(px(340.0)).rounded(px(12.0)).bg(bg_secondary())
                    .border_1().border_color(border_c()).p(px(24.0))
                    .flex().flex_col().gap(px(16.0)).items_center()
                    .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, _| { /* stop propagation */ }))
                    .child(Icon::new(IconName::TriangleAlert).text_color(danger()))
                    .child(div().text_sm().text_color(text_primary()).child(format!("Supprimer le bloc \"{block_label}\" ?")))
                    .child(div().text_xs().text_color(text_muted()).child("Cette action est irreversible."))
                    .child(div().flex().gap(px(8.0))
                        .child(div().px(px(16.0)).py(px(6.0)).rounded(px(6.0)).bg(bg_tertiary())
                            .text_xs().text_color(text_secondary()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                            .child("Annuler")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.store.update(cx, |s, cx| { s.confirm_delete_block = None; cx.emit(StoreEvent::ProjectChanged); });
                            })))
                        .child(div().px(px(16.0)).py(px(6.0)).rounded(px(6.0)).bg(danger())
                            .text_xs().text_color(ink_white()).cursor_pointer().hover(|s| s.bg(hsla(0.0, 0.7, 0.4, 1.0)))
                            .child("Supprimer")
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                this.store.update(cx, |s, cx| {
                                    if let Some(idx) = s.confirm_delete_block {
                                        if idx < s.project.blocks.len() {
                                            s.undo_stack.push_back(s.project.blocks.clone());
                                            while s.undo_stack.len() > 50 { s.undo_stack.pop_front(); }
                                            s.project.blocks.remove(idx);
                                            s.prompt_dirty = true;
                                        }
                                    }
                                    s.confirm_delete_block = None;
                                    cx.emit(StoreEvent::ProjectChanged);
                                });
                            })))))
        });

        div().flex_1().flex().flex_col().min_w_0()
            // Scrollable editor area
            .child(div().id("editor-scroll").flex_1().overflow_y_scroll()
                .p(px(16.0)).flex().flex_col().gap(px(12.0)).child(block_list)
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    // Close all sidebar menus when clicking on editor
                    this.store.update(cx, |_s, cx| { cx.emit(StoreEvent::CloseAllMenus); });
                    // Close add menu
                    if this.show_add_menu { this.show_add_menu = false; cx.notify(); }
                })))
            // Terminal at bottom (like VS Code / Zed)
            .children(terminal_output)
            // Delete confirmation overlay
            .children(delete_modal)
    }
}
