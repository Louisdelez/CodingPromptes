//! Left panel view implementations — extracted from left_panel.rs

use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use gpui_component::menu::{ContextMenuExt, PopupMenuItem};
use crate::store::StoreEvent;
use crate::state::*;
use crate::ui::colors::*;

use super::{LeftPanel, ContextTarget, DragFile};

impl LeftPanel {
    pub(crate) fn confirm_rename(this: &mut Self, cx: &mut Context<Self>) {
        if let (Some(ref id), Some(ref input)) = (&this.renaming_id, &this.rename_input) {
            let new_name = input.read(cx).value().to_string();
            if !new_name.trim().is_empty() {
                let id = id.clone();
                let name = new_name.trim().to_string();
                this.store.update(cx, |s, cx| {
                    // Try renaming workspace
                    if let Some(ws) = s.workspaces.iter_mut().find(|w| w.id == id) {
                        ws.name = name.clone();
                    }
                    // Try renaming project
                    if let Some(p) = s.projects.iter_mut().find(|p| p.id == id) {
                        p.name = name.clone();
                    }
                    if s.project.id == id {
                        s.project.name = name;
                    }
                    s.save_pending = true;
                    cx.emit(StoreEvent::ProjectChanged);
                });
            }
        }
        this.renaming_id = None;
        this.rename_input = None;
        cx.notify();
    }

    pub(crate) fn create_workspace(this: &mut Self, cx: &mut Context<Self>) {
        let name = this.new_ws_input.as_ref()
            .map(|i| i.read(cx).value().to_string()).unwrap_or_default();
        let name = if name.trim().is_empty() { "Nouveau Dossier".to_string() } else { name.trim().to_string() };
        let color = this.new_ws_color.clone();
        this.store.update(cx, |s, cx| {
            s.workspaces.push(inkwell_core::types::Workspace {
                id: uuid::Uuid::new_v4().to_string(), name,
                description: String::new(), color, constitution: None,
                created_at: chrono::Utc::now().timestamp_millis(),
                updated_at: chrono::Utc::now().timestamp_millis(),
            });
            cx.emit(StoreEvent::ProjectChanged);
        });
        this.show_new_workspace = false; this.new_ws_input = None; cx.notify();
    }

    pub(crate) fn render_library(&self, projects: &[ProjectSummary], workspaces: &[inkwell_core::types::Workspace],
                      current_id: &str, _confirm_delete: &Option<String>, cx: &mut Context<Self>) -> Div {
        let mut c = div().flex_1().px(px(12.0)).py(px(6.0)).flex().flex_col().gap(px(1.0));
        let search = self.search_input.as_ref().map(|i| i.read(cx).value().to_string().to_lowercase()).unwrap_or_default();

        let renaming = self.renaming_id.clone();
        let _ctx_menu = self.context_menu.clone();
        let weak_view = cx.entity().downgrade();

        // ── Workspaces (Dossiers) ──
        for ws in workspaces {
            let color = hex_to_hsla(&ws.color);
            let ws_id = ws.id.clone();
            let ws_id2 = ws.id.clone();
            let ws_id3 = ws.id.clone();
            let ws_id4 = ws.id.clone();
            let is_expanded = self.expanded_workspaces.contains(&ws.id);
            let project_count = projects.len();
            let is_renaming = renaming.as_deref() == Some(&ws.id);

            let ws_drop_id = ws.id.clone();
            let mut ws_row = div().id(SharedString::from(format!("ws-{}", ws.id)))
                .px(px(6.0)).py(px(5.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                .hover(|s| s.bg(bg_tertiary()))
                // Drop target: accept file DnD
                .drag_over::<DragFile>(|this, _, _, _| {
                    this.border_2().border_color(accent()).bg(accent_bg())
                })
                .on_drop(cx.listener(move |this, drag: &DragFile, _, cx| {
                    let pid = drag.project_id.clone();
                    let wid = ws_drop_id.clone();
                    this.store.update(cx, |s, cx| {
                        if let Some(p) = s.projects.iter_mut().find(|p| p.id == pid) {
                            p.workspace_id = Some(wid);
                        }
                        cx.emit(StoreEvent::ProjectChanged);
                    });
                }))
                // Expand chevron
                .child(div().w(px(14.0)).text_color(text_muted())
                    .child(Icon::new(if is_expanded { IconName::ChevronDown } else { IconName::ChevronRight }))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        if this.expanded_workspaces.contains(&ws_id) {
                            this.expanded_workspaces.retain(|id| id != &ws_id);
                        } else { this.expanded_workspaces.push(ws_id.clone()); }
                        cx.notify();
                    })))
                // Color dot
                .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(color));

            // Name or rename input
            if is_renaming {
                ws_row = ws_row.child(if let Some(ref entity) = self.rename_input {
                    div().flex_1().child(Input::new(entity))
                } else { div().flex_1() });
            } else {
                ws_row = ws_row.child(div().flex_1().text_xs().text_color(text_primary()).child(ws.name.clone())
                    // Double-click to rename
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                        if ev.click_count == 2 {
                            this.renaming_id = Some(ws_id2.clone());
                            this.rename_input = None; // will be created below
                            cx.notify();
                        }
                    })));
            }

            ws_row = ws_row
                // Count
                .child(div().text_xs().text_color(text_muted()).child(format!("{project_count}")))
                // Add file in folder button
                .child(div().text_color(text_muted()).child(Icon::new(IconName::Plus))
                    .cursor_pointer().hover(|s| s.bg(accent_bg()))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.store.update(cx, |s, cx| {
                            let num = s.feature_counter;
                            let mut p = Project::default_prompt();
                            p.name = format!("{:03}-nouveau-prompte", num);
                            p.workspace_id = Some(ws_id3.clone());
                            let now = chrono::Local::now();
                            p.tags.push(now.format("%Y-%m-%d %H:%M").to_string());
                            let ws = Some(ws_id3.clone());
                            s.projects.push(ProjectSummary { id: p.id.clone(), name: p.name.clone(), workspace_id: ws });
                            s.project = p; s.prompt_dirty = true; s.save_pending = true;
                            s.feature_counter += 1;
                            cx.emit(StoreEvent::ProjectChanged);
                        });
                    })))
                // Delete folder (with confirmation)
                .child(div().text_color(text_muted()).child(Icon::new(IconName::Trash2))
                    .cursor_pointer().hover(|s| s.opacity(1.0))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.confirm_delete_target = Some(ContextTarget::Folder(ws_id4.clone()));
                        cx.notify();
                    })));

            // Native right-click context menu (floating)
            let ws_rename = ws.id.clone();
            let ws_delete = ws.id.clone();
            let view_ws = weak_view.clone();
            let view_ws2 = weak_view.clone();
            c = c.child(ws_row.context_menu(move |menu, _, _| {
                let rid = ws_rename.clone();
                let did = ws_delete.clone();
                let v1 = view_ws.clone();
                let v2 = view_ws2.clone();
                menu.item(PopupMenuItem::new("Renommer").on_click(move |_, _, cx| {
                    v1.update(cx, |this, cx| {
                        this.renaming_id = Some(rid.clone());
                        this.rename_input = None; cx.notify();
                    }).ok();
                }))
                .separator()
                .item(PopupMenuItem::new("Supprimer").on_click(move |_, _, cx| {
                    v2.update(cx, |this, cx| {
                        this.confirm_delete_target = Some(ContextTarget::Folder(did.clone()));
                        cx.notify();
                    }).ok();
                }))
            }));
        }

        if !workspaces.is_empty() {
            c = c.child(div().h(px(1.0)).bg(border_c()).my(px(4.0)));
        }

        // ── Projects (Fichiers) ──
        let filtered: Vec<&ProjectSummary> = projects.iter()
            .filter(|p| search.is_empty() || p.name.to_lowercase().contains(&search)).collect();

        for p in &filtered {
            let id = p.id.clone();
            let id2 = p.id.clone();
            let _id_ctx = p.id.clone();
            let is_active = current_id == p.id;
            let is_renaming = renaming.as_deref() == Some(&p.id);

            let drag_id = p.id.clone();
            let drag_name = p.name.clone();
            let mut row = div().id(SharedString::from(format!("file-{}", p.id)))
                .px(px(8.0)).py(px(6.0)).rounded(px(6.0)).flex().items_center().gap(px(8.0))
                .hover(|s| s.bg(bg_tertiary()))
                .cursor_pointer()
                // Make file draggable for DnD into folders
                .on_drag(DragFile { project_id: drag_id, name: drag_name }, |drag, _, _, cx| {
                    cx.new(|_| drag.clone())
                });
            if is_active {
                row = row.border_l_3().border_color(accent()).bg(accent_bg());
            }

            row = row
                .child(Icon::new(IconName::File).text_color(if is_active { accent() } else { text_muted() }))
                .child(Icon::new(IconName::Clock).text_color(text_muted()));

            // Name (or rename input)
            if is_renaming {
                row = row.child(if let Some(ref entity) = self.rename_input {
                    div().flex_1().child(Input::new(entity))
                } else { div().flex_1() });
            } else {
                row = row.child(div().flex_1().text_xs().overflow_hidden()
                    .text_color(if is_active { text_primary() } else { text_secondary() })
                    .child(p.name.clone())
                    // Single click to open, double-click to rename
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                        if ev.click_count == 2 {
                            this.renaming_id = Some(id2.clone());
                            this.rename_input = None;
                            cx.notify();
                        } else {
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
                        }
                    })));
            }

            // Native right-click context menu (floating)
            let file_rename = p.id.clone();
            let file_delete = p.id.clone();
            let view_f1 = weak_view.clone();
            let view_f2 = weak_view.clone();
            c = c.child(row.context_menu(move |menu, _, _| {
                let rid = file_rename.clone();
                let did = file_delete.clone();
                let v1 = view_f1.clone();
                let v2 = view_f2.clone();
                menu.item(PopupMenuItem::new("Renommer").on_click(move |_, _, cx| {
                    v1.update(cx, |this, cx| {
                        this.renaming_id = Some(rid.clone());
                        this.rename_input = None; cx.notify();
                    }).ok();
                }))
                .separator()
                .item(PopupMenuItem::new("Supprimer").on_click(move |_, _, cx| {
                    v2.update(cx, |this, cx| {
                        this.confirm_delete_target = Some(ContextTarget::File(did.clone()));
                        cx.notify();
                    }).ok();
                }))
            }));
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

    pub(crate) fn render_frameworks(&self, frameworks: &[CustomFramework], cx: &mut Context<Self>) -> Div {
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

        // ── Steering Rules Section (Kiro) — dynamic from store ──
        c = c.child(div().h(px(1.0)).bg(border_c()).my(px(4.0)));
        c = c.child(div().flex().items_center().gap(px(6.0))
            .child(Icon::new(IconName::Scroll).text_color(text_muted()))
            .child(div().flex_1().text_xs().text_color(text_muted()).child("Steering (Kiro)")));

        let steering = &self.store.read(cx).steering;
        for (i, rule) in steering.rules.iter().enumerate() {
            c = c.child(
                div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                    .hover(|s| s.bg(bg_tertiary()))
                    .cursor_pointer()
                    .child(div().w(px(6.0)).h(px(6.0)).rounded(px(3.0))
                        .bg(if rule.enabled { success() } else { text_muted() }))
                    .child(Icon::new(IconName::File).text_color(if rule.enabled { accent() } else { text_muted() }))
                    .child(div().flex_1().flex().flex_col()
                        .child(div().text_xs().text_color(text_primary()).child(rule.name.clone()))
                        .child(div().text_xs().text_color(text_muted()).child(rule.description.clone())))
                    .child(div().text_xs().text_color(text_muted()).child(format!("{:?}", rule.inclusion)))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.store.update(cx, |s, _| { s.steering.toggle(i); });
                        cx.notify();
                    }))
            );
        }

        c
    }

    pub(crate) fn render_versions(&mut self, versions: &[inkwell_core::types::Version], window: &mut Window, cx: &mut Context<Self>) -> Div {
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
