mod views;
use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use gpui_component::menu::ContextMenuExt;
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum SidebarView { Library, Frameworks, Versions }

/// Context menu action for right-click
#[derive(Clone)]
pub(crate) enum ContextTarget {
    File(String),
    Folder(String),
}

/// Drag payload for file → folder DnD
#[derive(Clone)]
pub struct DragFile {
    pub project_id: String,
    pub name: String,
}

impl Render for DragFile {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(160.0)).h(px(28.0)).rounded(px(6.0))
            .bg(bg_tertiary()).opacity(0.85).px(px(8.0))
            .flex().items_center().gap(px(6.0))
            .child(Icon::new(IconName::File).text_color(accent()))
            .child(div().text_xs().text_color(text_primary()).child(self.name.clone()))
    }
}

pub struct LeftPanel {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) store: Entity<AppStore>,
    pub(crate) search_input: Option<Entity<InputState>>,
    pub(crate) view: SidebarView,
    pub(crate) show_dropdown: bool,
    // Workspace creation
    pub(crate) show_new_workspace: bool,
    pub(crate) new_ws_input: Option<Entity<InputState>>,
    pub(crate) new_ws_color: String,
    // Expanded workspaces
    pub(crate) expanded_workspaces: Vec<String>,
    // Frameworks
    pub(crate) show_custom_frameworks: bool,
    pub(crate) show_builtin_frameworks: bool,
    // Versions
    pub(crate) version_label_input: Option<Entity<InputState>>,
    pub(crate) expanded_versions: Vec<String>,
    // Rename
    pub(crate) renaming_id: Option<String>,
    pub(crate) rename_input: Option<Entity<InputState>>,
    // Context menu (right-click)
    pub(crate) context_menu: Option<ContextTarget>,
    // Delete confirmation modal
    pub(crate) confirm_delete_target: Option<ContextTarget>,
}

impl LeftPanel {
    pub fn new(store: Entity<AppStore>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let search_input = Some(cx.new(|cx| InputState::new(window, cx).placeholder("Rechercher...")));
        cx.subscribe(&store, |this, _, event: &StoreEvent, cx| {
            match event {
                StoreEvent::ProjectChanged | StoreEvent::SessionChanged => cx.notify(),
                StoreEvent::CloseAllMenus => {
                    this.show_dropdown = false;
                    this.context_menu = None;
                    if this.renaming_id.is_some() { Self::confirm_rename(this, cx); }
                    cx.notify();
                }
                _ => {}
            }
        }).detach();
        Self {
            focus_handle, store, search_input, view: SidebarView::Library,
            show_dropdown: false, show_new_workspace: false,
            new_ws_input: None, new_ws_color: "#6366f1".into(),
            expanded_workspaces: vec![],
            show_custom_frameworks: true, show_builtin_frameworks: true,
            version_label_input: None, expanded_versions: vec![],
            renaming_id: None, rename_input: None,
            context_menu: None, confirm_delete_target: None,
        }
    }
}

impl Focusable for LeftPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle { self.focus_handle.clone() }
}

impl Render for LeftPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.store.read(cx);
        let projects: Vec<ProjectSummary> = s.projects.clone();
        let workspaces: Vec<inkwell_core::types::Workspace> = s.workspaces.clone();
        let current_id = s.project.id.clone();
        let confirm_delete = s.confirm_delete.clone();
        let custom_fw: Vec<CustomFramework> = s.custom_frameworks.clone();
        let versions: Vec<inkwell_core::types::Version> = s.versions.clone();

        let view_label = match self.view {
            SidebarView::Library => "Bibliotheque",
            SidebarView::Frameworks => "Frameworks",
            SidebarView::Versions => "Versions",
        };

        let panel_width = self.store.read(cx).left_width;
        let mut panel = div().track_focus(&self.focus_handle).w(px(panel_width)).flex_shrink_0().border_r_1().border_color(border_c()).bg(bg_secondary())
            .flex().flex_col();

        let view_icon = match self.view {
            SidebarView::Library => IconName::FolderOpen,
            SidebarView::Frameworks => IconName::Layers,
            SidebarView::Versions => IconName::History,
        };

        let show_dd = self.show_dropdown;

        // ── Header: click anywhere on header toggles dropdown ──
        panel = panel.child(
            div().id("left-panel-header").h(px(44.0)).px(px(16.0)).flex().items_center().gap(px(8.0))
                .border_b_1().border_color(border_c())
                .cursor_pointer().hover(|s| s.bg(bg_hover()))
                .child(Icon::new(view_icon).text_color(text_muted()))
                .child(div().flex_1().text_sm().font_weight(FontWeight::SEMIBOLD).text_color(text_primary()).child(view_label))
                .child(Icon::new(if show_dd { IconName::ChevronUp } else { IconName::ChevronDown }).text_color(text_muted()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.show_dropdown = !this.show_dropdown; cx.notify();
                }))
        );

        // ── Dropdown menu (floating overlay via deferred+anchored) ──
        if show_dd {
            let items = [
                ("Bibliotheque", SidebarView::Library, IconName::FolderOpen),
                ("Frameworks", SidebarView::Frameworks, IconName::Layers),
                ("Versions", SidebarView::Versions, IconName::History),
            ];
            let mut menu = div().mx(px(8.0)).mt(px(4.0)).rounded(px(8.0))
                .bg(bg_secondary()).border_1().border_color(border_c())
                .p(px(4.0)).flex().flex_col().gap(px(2.0))
                .w(px(panel_width - 16.0));
            for (label, view, icon) in items {
                let is_active = self.view == view;
                menu = menu.child(
                    div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).flex().items_center().gap(px(8.0))
                        .text_sm().cursor_pointer()
                        .text_color(if is_active { accent() } else { text_primary() })
                        .bg(if is_active { accent_bg() } else { transparent() })
                        .hover(|s| s.bg(bg_hover()))
                        .child(Icon::new(icon))
                        .child(label.to_string())
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            this.view = view; this.show_dropdown = false; cx.notify();
                        }))
                );
            }
            // Floating overlay — does NOT push content down
            panel = panel.child(
                deferred(
                    anchored().snap_to_window_with_margin(px(8.0)).child(menu)
                ).with_priority(1)
            );
        }

        // ── Search bar + action buttons (like web: loupe + input + FolderPlus + Plus) ──
        if self.view == SidebarView::Library {
            panel = panel.child(
                div().px(px(14.0)).py(px(8.0)).flex().items_center().gap(px(8.0))
                    .child(Icon::new(IconName::Search).text_color(text_muted()))
                    .child(if let Some(ref entity) = self.search_input {
                        div().flex_1().child(Input::new(entity))
                    } else { div().flex_1() })
                    .child(
                        div().px(px(4.0)).py(px(2.0)).rounded(px(4.0))
                            .child(Icon::new(IconName::FolderPlus).text_color(text_muted()))
                            .cursor_pointer().hover(|s| s.bg(accent_bg()))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                this.show_new_workspace = true;
                                let input = cx.new(|cx| InputState::new(window, cx).placeholder("Nom du dossier..."));
                                // Subscribe to Enter key on the input
                                cx.subscribe(&input, |this, _, event: &gpui_component::input::InputEvent, cx| {
                                    if matches!(event, gpui_component::input::InputEvent::PressEnter { .. }) {
                                        Self::create_workspace(this, cx);
                                    }
                                }).detach();
                                this.new_ws_input = Some(input);
                                cx.notify();
                            }))
                    )
                    .child(
                        div().px(px(4.0)).py(px(2.0)).rounded(px(4.0))
                            .child(Icon::new(IconName::Plus).text_color(text_muted()))
                            .cursor_pointer().hover(|s| s.bg(accent_bg()))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.store.update(cx, |s, cx| {
                                    let num = s.feature_counter;
                                    let mut p = Project::default_prompt();
                                    p.name = format!("{:03}-nouveau-prompte", num);
                                    let now = chrono::Local::now();
                                    p.tags.push(now.format("%Y-%m-%d %H:%M").to_string());
                                    s.projects.push(ProjectSummary { id: p.id.clone(), name: p.name.clone(), workspace_id: None });
                                    s.project = p; s.prompt_dirty = true; s.save_pending = true;
                                    s.feature_counter += 1;
                                    cx.emit(StoreEvent::ProjectChanged);
                                });
                            }))
                    )
            );

            // ── New workspace inline form ──
            if self.show_new_workspace {
                const COLORS: &[&str] = &["#6366f1","#8b5cf6","#ec4899","#22c55e","#06b6d4","#f97316","#ef4444","#eab308"];
                let sel_color = self.new_ws_color.clone();
                panel = panel.child(
                    div().mx(px(10.0)).mb(px(6.0)).p(px(8.0)).rounded(px(6.0))
                        .bg(bg_tertiary()).border_1().border_color(border_c())
                        .flex().flex_col().gap(px(6.0))
                        // Color picker
                        .child({
                            let mut row = div().flex().gap(px(4.0));
                            for hex in COLORS {
                                let h = hex.to_string();
                                let is_sel = sel_color == *hex;
                                row = row.child(
                                    div().w(px(16.0)).h(px(16.0)).rounded(px(8.0)).bg(hex_to_hsla(hex))
                                        .border_2().border_color(if is_sel { text_primary() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                            this.new_ws_color = h.clone(); cx.notify();
                                        }))
                                );
                            }
                            row
                        })
                        // Name input + create + cancel
                        .child(div().flex().items_center().gap(px(4.0))
                            .child(if let Some(ref entity) = self.new_ws_input {
                                div().flex_1().child(Input::new(entity))
                            } else { div().flex_1() })
                            .child(div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(accent())
                                .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)).child("Creer")
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                    Self::create_workspace(this, cx);
                                })))
                            .child(div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                                .child(Icon::new(IconName::Close).text_color(text_muted()))
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                    this.show_new_workspace = false; this.new_ws_input = None; cx.notify();
                                })))
                        )
                );
            }
        }

        // ── Content (scrollable) ──
        let content = match self.view {
            SidebarView::Library => self.render_library(&projects, &workspaces, &current_id, &confirm_delete, cx),
            SidebarView::Frameworks => self.render_frameworks(&custom_fw, cx),
            SidebarView::Versions => self.render_versions(&versions, window, cx),
        };
        // Create rename input if needed and not yet created
        if self.renaming_id.is_some() && self.rename_input.is_none() {
            let current_name = if let Some(ref rid) = self.renaming_id {
                // Try workspace name first, then project name
                workspaces.iter().find(|w| w.id == *rid).map(|w| w.name.clone())
                    .or_else(|| projects.iter().find(|p| p.id == *rid).map(|p| p.name.clone()))
                    .unwrap_or_default()
            } else { String::new() };
            let input = cx.new(|cx| InputState::new(window, cx).default_value(current_name));
            // Enter to confirm rename
            cx.subscribe(&input, |this, _, event: &gpui_component::input::InputEvent, cx| {
                if matches!(event, gpui_component::input::InputEvent::PressEnter { .. }) {
                    Self::confirm_rename(this, cx);
                }
            }).detach();
            self.rename_input = Some(input);
        }

        panel = panel.child(div().id("left-content").flex_1().overflow_y_scroll()
            .child(content)
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                if this.show_dropdown { this.show_dropdown = false; }
                if this.context_menu.is_some() { this.context_menu = None; }
                if this.renaming_id.is_some() { Self::confirm_rename(this, cx); }
                window.blur(); // Remove focus from search input
                cx.notify();
            })));

        // Delete confirmation modal (centered overlay)
        if let Some(ref target) = self.confirm_delete_target.clone() {
            let label = match target {
                ContextTarget::File(id) => format!("le fichier \"{}\"", projects.iter().find(|p| p.id == *id).map(|p| p.name.as_str()).unwrap_or("?")),
                ContextTarget::Folder(id) => format!("le dossier \"{}\" et tout son contenu", workspaces.iter().find(|w| w.id == *id).map(|w| w.name.as_str()).unwrap_or("?")),
            };
            let target_clone = target.clone();
            panel = panel.child(
                div().id("delete-modal").size_full().absolute().top_0().left_0()
                    .bg(hsla(0.0, 0.0, 0.0, 0.4))
                    .flex().items_center().justify_center()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.confirm_delete_target = None; cx.notify();
                    }))
                    .child(div().w(px(340.0)).rounded(px(12.0)).bg(bg_secondary())
                        .border_1().border_color(border_c()).p(px(24.0))
                        .flex().flex_col().gap(px(16.0)).items_center()
                        .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, _| {}))
                        .child(Icon::new(IconName::TriangleAlert).text_color(danger()))
                        .child(div().text_sm().text_color(text_primary()).child(format!("Supprimer {label} ?")))
                        .child(div().text_xs().text_color(text_muted()).child("Cette action est irreversible."))
                        .child(div().flex().gap(px(8.0))
                            .child(div().px(px(16.0)).py(px(6.0)).rounded(px(6.0)).bg(bg_tertiary())
                                .text_xs().text_color(text_secondary()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                                .child("Annuler")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                    this.confirm_delete_target = None; cx.notify();
                                })))
                            .child(div().px(px(16.0)).py(px(6.0)).rounded(px(6.0)).bg(danger())
                                .text_xs().text_color(ink_white()).cursor_pointer()
                                .child("Supprimer")
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                    match &target_clone {
                                        ContextTarget::File(id) => {
                                            let id = id.clone();
                                            crate::persistence::delete_project(&id);
                                            this.store.update(cx, |s, cx| {
                                                s.projects.retain(|p| p.id != id);
                                                s.confirm_delete = None;
                                                cx.emit(StoreEvent::ProjectChanged);
                                            });
                                        }
                                        ContextTarget::Folder(id) => {
                                            let id = id.clone();
                                            // Cascade: delete all projects in this workspace
                                            let id2 = id.clone();
                                            this.store.update(cx, |s, cx| {
                                                // Delete projects belonging to this workspace
                                                let to_delete: Vec<String> = s.projects.iter()
                                                    .filter(|p| p.workspace_id.as_deref() == Some(&id2))
                                                    .map(|p| p.id.clone()).collect();
                                                for pid in &to_delete {
                                                    crate::persistence::delete_project(pid);
                                                }
                                                s.projects.retain(|p| p.workspace_id.as_deref() != Some(&id2));
                                                s.workspaces.retain(|w| w.id != id2);
                                                cx.emit(StoreEvent::ProjectChanged);
                                            });
                                        }
                                    }
                                    this.confirm_delete_target = None; cx.notify();
                                })))))
            );
        }

        panel
    }
}
