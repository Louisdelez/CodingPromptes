use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;

#[derive(Clone, Copy, PartialEq)]
enum SidebarView { Library, Frameworks, Versions }

pub struct LeftPanel {
    store: Entity<AppStore>,
    search_input: Option<Entity<InputState>>,
    view: SidebarView,
    show_dropdown: bool,
    // Workspace creation
    show_new_workspace: bool,
    new_ws_input: Option<Entity<InputState>>,
    new_ws_color: String,
    // Expanded workspaces
    expanded_workspaces: Vec<String>,
    // Frameworks
    show_custom_frameworks: bool,
    show_builtin_frameworks: bool,
    // Versions
    version_label_input: Option<Entity<InputState>>,
    expanded_versions: Vec<String>,
}

impl LeftPanel {
    pub fn new(store: Entity<AppStore>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = Some(cx.new(|cx| InputState::new(window, cx).placeholder("Rechercher...")));
        cx.subscribe(&store, |_this, _, event: &StoreEvent, cx| {
            match event {
                StoreEvent::ProjectChanged | StoreEvent::SessionChanged => cx.notify(),
                _ => {}
            }
        }).detach();
        Self {
            store, search_input, view: SidebarView::Library,
            show_dropdown: false, show_new_workspace: false,
            new_ws_input: None, new_ws_color: "#6366f1".into(),
            expanded_workspaces: vec![],
            show_custom_frameworks: true, show_builtin_frameworks: true,
            version_label_input: None, expanded_versions: vec![],
        }
    }
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
        drop(s);

        let view_label = match self.view {
            SidebarView::Library => "Bibliotheque",
            SidebarView::Frameworks => "Frameworks",
            SidebarView::Versions => "Versions",
        };

        let mut panel = div().w(px(288.0)).flex_shrink_0().border_r_1().border_color(border_c()).bg(bg_secondary())
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

        // ── Dropdown menu (in flow, with hover animations) ──
        if show_dd {
            let items = [
                ("Bibliotheque", SidebarView::Library, IconName::FolderOpen),
                ("Frameworks", SidebarView::Frameworks, IconName::Layers),
                ("Versions", SidebarView::Versions, IconName::History),
            ];
            let mut menu = div().mx(px(8.0)).my(px(4.0)).rounded(px(8.0))
                .bg(bg_tertiary()).border_1().border_color(border_c())
                .p(px(4.0)).flex().flex_col().gap(px(2.0));
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
            panel = panel.child(menu);
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
                                this.new_ws_input = Some(cx.new(|cx| InputState::new(window, cx).placeholder("Nom du projet...")));
                                cx.notify();
                            }))
                    )
                    .child(
                        div().px(px(4.0)).py(px(2.0)).rounded(px(4.0))
                            .child(Icon::new(IconName::Plus).text_color(text_muted()))
                            .cursor_pointer().hover(|s| s.bg(accent_bg()))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.store.update(cx, |s, cx| {
                                    let p = Project::default_prompt();
                                    s.projects.push(ProjectSummary { id: p.id.clone(), name: p.name.clone() });
                                    s.project = p; s.prompt_dirty = true; s.save_pending = true;
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
                                    let name = this.new_ws_input.as_ref().map(|i| i.read(cx).value().to_string()).unwrap_or_default();
                                    if name.trim().is_empty() { return; }
                                    let color = this.new_ws_color.clone();
                                    this.store.update(cx, |s, cx| {
                                        s.workspaces.push(inkwell_core::types::Workspace {
                                            id: uuid::Uuid::new_v4().to_string(), name: name.trim().to_string(),
                                            description: String::new(), color, constitution: None,
                                            created_at: chrono::Utc::now().timestamp_millis(),
                                            updated_at: chrono::Utc::now().timestamp_millis(),
                                        });
                                        cx.emit(StoreEvent::ProjectChanged);
                                    });
                                    this.show_new_workspace = false; this.new_ws_input = None; cx.notify();
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
        panel = panel.child(div().id("left-content").flex_1().overflow_y_scroll()
            .child(content)
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                if this.show_dropdown { this.show_dropdown = false; cx.notify(); }
            })));

        panel
    }
}

impl LeftPanel {
    fn render_library(&self, projects: &[ProjectSummary], workspaces: &[inkwell_core::types::Workspace],
                      current_id: &str, confirm_delete: &Option<String>, cx: &mut Context<Self>) -> Div {
        let mut c = div().flex_1().px(px(12.0)).py(px(6.0)).flex().flex_col().gap(px(1.0));
        let search = self.search_input.as_ref().map(|i| i.read(cx).value().to_string().to_lowercase()).unwrap_or_default();

        // ── Workspaces ──
        for ws in workspaces {
            let color = hex_to_hsla(&ws.color);
            let ws_id = ws.id.clone();
            let ws_del_id = ws.id.clone();
            let is_expanded = self.expanded_workspaces.contains(&ws.id);
            let project_count = projects.len(); // All projects shown under workspace for now

            c = c.child(
                div().px(px(6.0)).py(px(5.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                    .hover(|s| s.bg(bg_tertiary()))
                    // Expand chevron
                    .child(
                        div().w(px(14.0)).text_color(text_muted())
                            .child(Icon::new(if is_expanded { IconName::ChevronDown } else { IconName::ChevronRight }))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                if this.expanded_workspaces.contains(&ws_id) {
                                    this.expanded_workspaces.retain(|id| id != &ws_id);
                                } else {
                                    this.expanded_workspaces.push(ws_id.clone());
                                }
                                cx.notify();
                            }))
                    )
                    // Color dot
                    .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(color))
                    // Name
                    .child(div().flex_1().text_xs().text_color(text_primary()).child(ws.name.clone()))
                    // Count
                    .child(div().text_xs().text_color(text_muted()).child(format!("{project_count}")))
                    // Delete (hover only — we keep it always visible for now)
                    .child(div().text_color(text_muted()).child(Icon::new(IconName::Close))
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            this.store.update(cx, |s, cx| { s.workspaces.retain(|w| w.id != ws_del_id); cx.emit(StoreEvent::ProjectChanged); });
                        })))
            );
        }

        if !workspaces.is_empty() {
            c = c.child(div().h(px(1.0)).bg(border_c()).my(px(4.0)));
        }

        // ── Projects ──
        let filtered: Vec<&ProjectSummary> = projects.iter()
            .filter(|p| search.is_empty() || p.name.to_lowercase().contains(&search)).collect();

        for p in &filtered {
            let id = p.id.clone();
            let del_id = p.id.clone();
            let is_active = current_id == p.id;
            let mut row = div().px(px(8.0)).py(px(6.0)).rounded(px(6.0)).flex().items_center().gap(px(8.0))
                .hover(|s| s.bg(bg_tertiary()))
                .cursor_pointer();
            // Active state: accent left border + tinted background (matching web)
            if is_active {
                row = row.border_l_3().border_color(accent()).bg(accent_bg());
            }
            row = row
                // File icon
                .child(Icon::new(IconName::File).text_color(if is_active { accent() } else { text_muted() }))
                // Clock icon + relative time (matching web "il y a 43min")
                .child(Icon::new(IconName::Clock).text_color(text_muted()))
                // Name + time
                .child(div().flex_1().flex().flex_col().gap(px(1.0)).overflow_hidden()
                    .child(div().text_xs()
                        .text_color(if is_active { text_primary() } else { text_secondary() })
                        .child(p.name.clone()))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        let local = crate::persistence::load_all_projects();
                        if let Some(lp) = local.iter().find(|p| p.id == id) {
                            this.store.update(cx, |s, cx| {
                                s.project.id = lp.id.clone(); s.project.name = lp.name.clone();
                                s.project.framework = lp.framework.clone(); s.project.tags = lp.tags.clone();
                                s.project.variables = lp.variables.clone();
                                s.project.blocks = lp.blocks.iter().map(|b| Block {
                                    id: b.id.clone(), block_type: b.block_type, content: b.content.clone(),
                                    enabled: b.enabled, editing: false,
                                }).collect();
                                s.prompt_dirty = true; cx.emit(StoreEvent::ProjectChanged);
                            });
                        }
                    })))
                // Delete (Trash icon, visible on hover)
                .child(div().text_color(danger()).opacity(0.5)
                    .hover(|s| s.opacity(1.0))
                    .child(Icon::new(IconName::Trash2))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.store.update(cx, |s, _| { s.confirm_delete = Some(del_id.clone()); }); cx.notify();
                    })));
            c = c.child(row);
        }

        // ── Delete confirm ──
        if let Some(ref del_id) = confirm_delete {
            let del = del_id.clone();
            c = c.child(
                div().p(px(10.0)).mt(px(4.0)).rounded(px(8.0)).bg(hsla(0.0, 0.75, 0.55, 0.1)).border_1().border_color(danger())
                    .flex().flex_col().gap(px(6.0))
                    .child(div().text_xs().text_color(danger()).child("Supprimer ce prompt ?"))
                    .child(div().flex().gap(px(6.0))
                        .child(div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(danger()).text_xs().text_color(white()).child("Supprimer")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                let id = del.clone(); crate::persistence::delete_project(&id);
                                this.store.update(cx, |s, cx| { s.projects.retain(|p| p.id != id); s.confirm_delete = None; cx.emit(StoreEvent::ProjectChanged); });
                            })))
                        .child(div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(bg_tertiary()).text_xs().text_color(text_secondary()).child("Annuler")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.store.update(cx, |s, _| { s.confirm_delete = None; }); cx.notify();
                            }))))
            );
        }

        // ── Empty state ──
        if filtered.is_empty() && projects.is_empty() {
            c = c.child(
                div().py(px(24.0)).px(px(12.0)).flex().flex_col().items_center().gap(px(10.0))
                    .child(div().text_sm().text_color(text_muted()).child("Rien ici encore"))
                    .child(div().text_xs().text_color(text_muted()).text_center()
                        .child("Creez un projet (dossier) pour organiser vos prompts, ou un prompt libre."))
            );
        } else if filtered.is_empty() {
            c = c.child(div().text_xs().text_color(text_muted()).py(px(8.0)).child("Aucun resultat"));
        }

        c
    }

    fn render_frameworks(&self, frameworks: &[CustomFramework], cx: &mut Context<Self>) -> Div {
        const BUILT_IN: &[(&str, &str, &str)] = &[
            ("CO-STAR", "co-star", "Context, Objective, Style, Tone, Audience, Response"),
            ("RISEN", "risen", "Role, Instructions, Steps, End goal, Narrowing"),
            ("RACE", "race", "Role, Action, Context, Example"),
            ("SDD (Spec-Driven)", "sdd", "Constitution, Specification, Plan, Tasks, Implementation"),
            ("APE", "ape", "Action, Purpose, Expectation"),
            ("STOKE", "stoke", "Situation, Task, Objective, Knowledge, Example"),
        ];
        let current_fw = self.store.read(cx).project.framework.clone();

        let mut c = div().flex_1().px(px(12.0)).py(px(8.0)).flex().flex_col().gap(px(6.0));

        // ── Header: Frameworks title + buttons ──
        c = c.child(div().flex().items_center().gap(px(6.0))
            .child(Icon::new(IconName::Frame).text_color(text_muted()))
            .child(div().flex_1().text_xs().text_color(text_muted()).child("Frameworks")));

        // Save current as framework button
        c = c.child(
            div().w_full().py(px(6.0)).rounded(px(6.0)).bg(bg_tertiary()).border_1().border_color(border_c())
                .flex().items_center().justify_center().gap(px(6.0))
                .text_xs().text_color(text_secondary())
                .child(Icon::new(IconName::Save)).child("Sauvegarder comme framework")
                .cursor_pointer().hover(|s| s.bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.1)))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| {
                        let name = format!("Custom {}", s.custom_frameworks.len() + 1);
                        let blocks: Vec<(inkwell_core::types::BlockType, String)> = s.project.blocks.iter()
                            .map(|b| (b.block_type, b.content.clone())).collect();
                        s.custom_frameworks.push(CustomFramework { name, blocks });
                        cx.emit(StoreEvent::ProjectChanged);
                    });
                }))
        );

        // ── Section: Mes Frameworks (custom) ──
        if !frameworks.is_empty() {
            c = c.child(
                div().flex().items_center().gap(px(6.0)).py(px(4.0))
                    .child(Icon::new(if self.show_custom_frameworks { IconName::ChevronDown } else { IconName::ChevronRight }).text_color(text_muted()))
                    .child(Icon::new(IconName::User).text_color(text_muted()))
                    .child(div().flex_1().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_primary())
                        .child(format!("Mes Frameworks ({})", frameworks.len())))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.show_custom_frameworks = !this.show_custom_frameworks; cx.notify();
                    }))
            );
            if self.show_custom_frameworks {
                for (i, fw) in frameworks.iter().enumerate() {
                    let is_active = current_fw.as_deref() == Some(&format!("custom:{}", fw.name));
                    // Block type badges
                    let badges: Vec<(&str, Hsla)> = fw.blocks.iter().map(|&(bt, _)| {
                        let abbr = match bt { inkwell_core::types::BlockType::Role => "Ro", inkwell_core::types::BlockType::Context => "Cx",
                            inkwell_core::types::BlockType::Task => "Ta", inkwell_core::types::BlockType::Examples => "Ex",
                            inkwell_core::types::BlockType::Constraints => "Co", inkwell_core::types::BlockType::Format => "Fo",
                            _ => "Sd" };
                        (abbr, hex_to_hsla(bt.color()))
                    }).collect();

                    let mut card = div().p(px(8.0)).rounded(px(6.0))
                        .border_1().border_color(if is_active { accent() } else { border_c() })
                        .bg(bg_tertiary()).flex().flex_col().gap(px(4.0))
                        .hover(|s| s.bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.08)));

                    card = card.child(div().flex().items_center().gap(px(4.0))
                        .child(Icon::new(IconName::Star).text_color(accent()))
                        .child(div().flex_1().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(text_primary()).child(fw.name.clone()))
                        // Delete
                        .child(div().text_color(danger()).opacity(0.5).hover(|s| s.opacity(1.0))
                            .child(Icon::new(IconName::Trash2))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                this.store.update(cx, |s, cx| { if i < s.custom_frameworks.len() { s.custom_frameworks.remove(i); } cx.emit(StoreEvent::ProjectChanged); });
                            })))
                    );

                    // Block badges row
                    let mut badge_row = div().flex().flex_wrap().gap(px(2.0));
                    for (abbr, color) in &badges {
                        badge_row = badge_row.child(
                            div().px(px(4.0)).py(px(1.0)).rounded(px(3.0))
                                .bg(hsla(color.h, color.s, color.l, 0.15))
                                .text_xs().text_color(*color).child(abbr.to_string())
                        );
                    }
                    card = card.child(badge_row);

                    c = c.child(card.cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        if let Some(fw) = this.store.read(cx).custom_frameworks.get(i).cloned() {
                            this.store.update(cx, |s, cx| {
                                s.undo_stack.push_back(s.project.blocks.clone());
                                s.project.blocks = fw.blocks.iter().map(|(bt, content)| {
                                    let mut b = Block::new(*bt); b.content = content.clone(); b
                                }).collect();
                                s.prompt_dirty = true; cx.emit(StoreEvent::ProjectChanged);
                            });
                        }
                    })));
                }
            }
        }

        // ── Section: Built-in ──
        c = c.child(
            div().flex().items_center().gap(px(6.0)).py(px(4.0))
                .child(Icon::new(if self.show_builtin_frameworks { IconName::ChevronDown } else { IconName::ChevronRight }).text_color(text_muted()))
                .child(Icon::new(IconName::Frame).text_color(text_muted()))
                .child(div().flex_1().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_primary())
                    .child(format!("Built-in ({})", BUILT_IN.len())))
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.show_builtin_frameworks = !this.show_builtin_frameworks; cx.notify();
                }))
        );
        if self.show_builtin_frameworks {
            for &(name, id, desc) in BUILT_IN {
                let id_str = id.to_string();
                let is_active = current_fw.as_deref() == Some(id);
                c = c.child(
                    div().p(px(8.0)).rounded(px(6.0))
                        .border_1().border_color(if is_active { accent() } else { border_c() })
                        .bg(bg_tertiary()).flex().flex_col().gap(px(2.0))
                        .hover(|s| s.bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.08)))
                        .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(text_primary()).child(name.to_string()))
                        .child(div().text_xs().text_color(text_muted()).child(desc.to_string()))
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            this.store.update(cx, |s, cx| { s.project.framework = Some(id_str.clone()); s.prompt_dirty = true; cx.emit(StoreEvent::ProjectChanged); });
                        }))
                );
            }
        }

        c
    }

    fn render_versions(&mut self, versions: &[inkwell_core::types::Version], window: &mut Window, cx: &mut Context<Self>) -> Div {
        let mut c = div().flex_1().px(px(12.0)).py(px(8.0)).flex().flex_col().gap(px(6.0));

        // ── Header: Versions title ──
        c = c.child(div().flex().items_center().gap(px(6.0))
            .child(Icon::new(IconName::Redo).text_color(text_muted()))
            .child(div().flex_1().text_xs().text_color(text_muted()).child("Versions")));

        // ── Save new version: input + button ──
        if self.version_label_input.is_none() {
            self.version_label_input = Some(cx.new(|cx| InputState::new(window, cx).placeholder("Label de la version...")));
        }
        c = c.child(
            div().flex().items_center().gap(px(6.0))
                .child(if let Some(ref entity) = self.version_label_input {
                    div().flex_1().child(Input::new(entity))
                } else { div().flex_1() })
                .child(
                    div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(accent())
                        .flex().items_center().gap(px(4.0))
                        .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                        .child(Icon::new(IconName::Save)).child("Sauvegarder")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            let label = this.version_label_input.as_ref()
                                .map(|i| i.read(cx).value().to_string()).unwrap_or_default();
                            let label = if label.trim().is_empty() { format!("v{}", chrono::Utc::now().format("%H:%M")) }
                                else { label.trim().to_string() };
                            this.store.update(cx, |s, cx| {
                                let blocks_json = serde_json::to_string(&s.project.blocks.iter().map(|b| {
                                    inkwell_core::types::PromptBlock { id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled }
                                }).collect::<Vec<_>>()).unwrap_or_default();
                                s.versions.push(inkwell_core::types::Version {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    project_id: s.project.id.clone(),
                                    blocks_json, variables_json: "{}".into(),
                                    label, created_at: chrono::Utc::now().timestamp_millis(),
                                });
                                cx.emit(StoreEvent::ProjectChanged);
                            });
                            this.version_label_input = None; cx.notify();
                        }))
                )
        );

        c = c.child(div().h(px(1.0)).bg(border_c()));

        // ── Version list ──
        if versions.is_empty() {
            c = c.child(
                div().py(px(16.0)).flex().flex_col().items_center().gap(px(8.0))
                    .child(div().text_xs().text_color(text_muted()).child("Aucune version sauvegardee"))
            );
        } else {
            for v in versions.iter().rev() {
                let blocks_json = v.blocks_json.clone();
                let blocks_json2 = v.blocks_json.clone();
                let v_id = v.id.clone();
                let is_expanded = self.expanded_versions.contains(&v.id);

                let date_str = chrono::DateTime::from_timestamp_millis(v.created_at)
                    .map(|d| d.format("%d %b %H:%M").to_string()).unwrap_or_default();

                // Header row (expandable)
                let mut row = div().px(px(8.0)).py(px(6.0)).rounded(px(6.0))
                    .border_1().border_color(border_c()).bg(bg_tertiary())
                    .flex().flex_col().gap(px(4.0));

                row = row.child(
                    div().flex().items_center().gap(px(6.0))
                        // Chevron expand
                        .child(div().child(Icon::new(if is_expanded { IconName::ChevronDown } else { IconName::ChevronRight }).text_color(text_muted()))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                if this.expanded_versions.contains(&v_id) {
                                    this.expanded_versions.retain(|id| id != &v_id);
                                } else { this.expanded_versions.push(v_id.clone()); }
                                cx.notify();
                            })))
                        // Label (bold)
                        .child(div().flex_1().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(text_primary()).child(v.label.clone()))
                        // Date
                        .child(div().text_xs().text_color(text_muted()).child(date_str))
                        // Restore button
                        .child(div().text_color(accent())
                            .child(Icon::new(IconName::Undo))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                if let Ok(blocks) = serde_json::from_str::<Vec<inkwell_core::types::PromptBlock>>(&blocks_json) {
                                    this.store.update(cx, |s, cx| {
                                        s.undo_stack.push_back(s.project.blocks.clone());
                                        s.project.blocks = blocks.into_iter().map(|b| Block {
                                            id: b.id, block_type: b.block_type, content: b.content, enabled: b.enabled, editing: false
                                        }).collect();
                                        s.prompt_dirty = true; cx.emit(StoreEvent::ProjectChanged);
                                    });
                                }
                            })))
                );

                // Expanded content: preview of blocks
                if is_expanded {
                    if let Ok(blocks) = serde_json::from_str::<Vec<inkwell_core::types::PromptBlock>>(&blocks_json2) {
                        let preview: String = blocks.iter().filter(|b| b.enabled)
                            .map(|b| b.content.as_str()).collect::<Vec<_>>().join("\n\n");
                        row = row.child(
                            div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).bg(bg_secondary())
                                .max_h(px(120.0)).overflow_hidden()
                                .text_xs().text_color(text_secondary())
                                .child(if preview.is_empty() { "(vide)".to_string() } else { preview.chars().take(500).collect() })
                        );
                    }
                }

                c = c.child(row);
            }
        }

        c
    }
}
