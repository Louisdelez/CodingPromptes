use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;

pub struct LeftPanel {
    store: Entity<AppStore>,
    search_input: Option<Entity<InputState>>,
    workspace_name_input: Option<Entity<InputState>>,
    editing_workspace_id: Option<String>,
    is_library: bool,
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

        Self { store, search_input, workspace_name_input: None, editing_workspace_id: None, is_library: true }
    }
}

impl Render for LeftPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.store.read(cx);
        let projects: Vec<ProjectSummary> = s.projects.clone();
        let workspaces: Vec<inkwell_core::types::Workspace> = s.workspaces.clone();
        let current_id = s.project.id.clone();
        let selected_color = s.selected_workspace_color.clone();
        let custom_fw: Vec<CustomFramework> = s.custom_frameworks.clone();
        let confirm_delete = s.confirm_delete.clone();
        let lang = s.lang.clone();
        drop(s);

        let is_library = self.is_library;

        let mut content = div().w(px(250.0)).flex_shrink_0().border_r_1().border_color(border_c()).bg(bg_secondary())
            .flex().flex_col()
            // Tab bar
            .child(
                div().h(px(36.0)).px(px(8.0)).flex().items_center().gap(px(4.0)).border_b_1().border_color(border_c())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                            .text_color(if is_library { accent() } else { text_muted() })
                            .bg(if is_library { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) } else { hsla(0.0, 0.0, 0.0, 0.0) })
                            .child(Icon::new(IconName::FolderOpen)).child("Bibliotheque")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| { this.is_library = true; cx.notify(); }))
                    )
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                            .text_color(if !is_library { accent() } else { text_muted() })
                            .bg(if !is_library { hsla(239.0 / 360.0, 0.84, 0.67, 0.1) } else { hsla(0.0, 0.0, 0.0, 0.0) })
                            .child(Icon::new(IconName::Frame)).child("Frameworks")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| { this.is_library = false; cx.notify(); }))
                    )
            );

        if is_library {
            content = content.child(self.render_library(&projects, &workspaces, &current_id, &selected_color, &confirm_delete, cx));
        } else {
            content = content.child(self.render_frameworks(&custom_fw, &lang, cx));
        }

        content
    }
}

impl LeftPanel {
    fn render_library(&self, projects: &[ProjectSummary], workspaces: &[inkwell_core::types::Workspace],
                      current_id: &str, selected_color: &str, confirm_delete: &Option<String>,
                      cx: &mut Context<Self>) -> Div {
        let mut c = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(4.0));

        // Search
        if let Some(ref entity) = self.search_input {
            c = c.child(div().child(Input::new(entity)));
        }

        // Workspaces header + color picker
        c = c.child(div().flex().items_center().gap(px(4.0))
            .child(Icon::new(IconName::FolderOpen))
            .child(div().text_xs().text_color(text_muted()).child("Espaces de travail"))
            .child(div().flex_1())
            .child(div().px(px(4.0)).py(px(2.0)).rounded(px(3.0)).child(Icon::new(IconName::Plus)).text_color(text_muted())
                .cursor_pointer().hover(|s| s.bg(accent_bg()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    let color = this.store.read(cx).selected_workspace_color.clone();
                    this.store.update(cx, |s, cx| {
                        let name = format!("Workspace {}", s.workspaces.len() + 1);
                        s.workspaces.push(inkwell_core::types::Workspace {
                            id: uuid::Uuid::new_v4().to_string(), name, description: String::new(), color,
                            constitution: None,
                            created_at: chrono::Utc::now().timestamp_millis(), updated_at: chrono::Utc::now().timestamp_millis(),
                        });
                        cx.emit(StoreEvent::ProjectChanged);
                    });
                })))
        );

        // Color swatches
        {
            const PALETTE: &[&str] = &["#6366f1","#8b5cf6","#ec4899","#22c55e","#06b6d4","#f97316","#ef4444","#eab308"];
            let mut row = div().flex().gap(px(3.0)).px(px(4.0));
            for hex in PALETTE {
                let hex_str = hex.to_string();
                let is_sel = selected_color == *hex;
                row = row.child(div().w(px(14.0)).h(px(14.0)).rounded(px(7.0)).bg(hex_to_hsla(hex))
                    .border_1().border_color(if is_sel { text_primary() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.store.update(cx, |s, _| { s.selected_workspace_color = hex_str.clone(); });
                    })));
            }
            c = c.child(row);
        }

        // Workspaces list
        for ws in workspaces {
            let color = hex_to_hsla(&ws.color);
            let ws_id = ws.id.clone();
            c = c.child(div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                .hover(|s| s.bg(bg_tertiary()))
                .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(color))
                .child(div().flex_1().text_xs().text_color(text_primary()).child(ws.name.clone()))
                .child(div().text_xs().text_color(danger()).child(Icon::new(IconName::Close))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.store.update(cx, |s, cx| { s.workspaces.retain(|w| w.id != ws_id); cx.emit(StoreEvent::ProjectChanged); });
                    })))
            );
        }

        // New prompt
        c = c.child(div().px(px(10.0)).py(px(8.0)).rounded(px(6.0)).border_1().border_color(border_c())
            .bg(bg_tertiary()).text_xs().text_color(accent())
            .flex().items_center().justify_center().child(Icon::new(IconName::Plus)).child("Nouveau prompt")
            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.store.update(cx, |s, cx| {
                    let new_proj = Project::default_prompt();
                    s.projects.push(ProjectSummary { id: new_proj.id.clone(), name: new_proj.name.clone() });
                    s.project = new_proj;
                    s.prompt_dirty = true;
                    s.save_pending = true;
                    cx.emit(StoreEvent::ProjectChanged);
                });
            }))
        );

        // Project list
        let search = self.search_input.as_ref().map(|i| i.read(cx).value().to_string().to_lowercase()).unwrap_or_default();
        let filtered: Vec<&ProjectSummary> = projects.iter().filter(|p| search.is_empty() || p.name.to_lowercase().contains(&search)).collect();
        for p in &filtered {
            let id = p.id.clone();
            let del_id = p.id.clone();
            let is_active = current_id == p.id;
            c = c.child(div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(4.0))
                .bg(if is_active { bg_tertiary() } else { hsla(0.0, 0.0, 0.0, 0.0) }).hover(|s| s.bg(bg_tertiary()))
                .child(div().flex_1().text_xs().text_color(if is_active { text_primary() } else { text_secondary() }).child(p.name.clone())
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        let local_projects = crate::persistence::load_all_projects();
                        if let Some(lp) = local_projects.iter().find(|p| p.id == id) {
                            this.store.update(cx, |s, cx| {
                                s.project.id = lp.id.clone(); s.project.name = lp.name.clone();
                                s.project.framework = lp.framework.clone(); s.project.tags = lp.tags.clone();
                                s.project.variables = lp.variables.clone();
                                s.project.blocks = lp.blocks.iter().map(|b| Block {
                                    id: b.id.clone(), block_type: b.block_type, content: b.content.clone(), enabled: b.enabled, editing: false
                                }).collect();
                                s.prompt_dirty = true;
                                cx.emit(StoreEvent::ProjectChanged);
                            });
                        }
                    })))
                .child(div().px(px(4.0)).py(px(2.0)).rounded(px(3.0)).text_xs().text_color(danger()).child("x")
                    .hover(|s| s.bg(hsla(0.0, 0.75, 0.55, 0.15)))
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.store.update(cx, |s, _| { s.confirm_delete = Some(del_id.clone()); });
                        cx.notify();
                    })))
            );
        }

        // Delete confirm
        if let Some(ref del_id) = confirm_delete {
            let del = del_id.clone();
            c = c.child(div().p(px(10.0)).rounded(px(8.0)).bg(hsla(0.0, 0.75, 0.55, 0.1)).border_1().border_color(danger())
                .flex().flex_col().gap(px(6.0))
                .child(div().text_xs().text_color(danger()).child("Supprimer ce projet ?"))
                .child(div().flex().gap(px(6.0))
                    .child(div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(danger()).text_xs().text_color(white()).child("Supprimer")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            let id = del.clone();
                            crate::persistence::delete_project(&id);
                            this.store.update(cx, |s, cx| {
                                s.projects.retain(|p| p.id != id); s.confirm_delete = None;
                                cx.emit(StoreEvent::ProjectChanged);
                            });
                        })))
                    .child(div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(bg_tertiary()).text_xs().text_color(text_secondary()).child("Annuler")
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.store.update(cx, |s, _| { s.confirm_delete = None; });
                            cx.notify();
                        })))
                )
            );
        }

        if filtered.is_empty() && projects.is_empty() {
            c = c.child(div().text_xs().text_color(text_muted()).child("Rien ici encore"));
        }

        c
    }

    fn render_frameworks(&self, frameworks: &[CustomFramework], lang: &str, cx: &mut Context<Self>) -> Div {
        const BUILT_IN: &[(&str, &str)] = &[
            ("CO-STAR", "co-star"), ("RISEN", "risen"), ("RACE", "race"),
            ("SDD (Spec-Driven)", "sdd"), ("APE", "ape"), ("STOKE", "stoke"),
        ];
        let mut c = div().flex_1().p(px(12.0)).flex().flex_col().gap(px(4.0));

        for &(name, id) in BUILT_IN {
            let id_str = id.to_string();
            c = c.child(div().px(px(10.0)).py(px(8.0)).rounded(px(6.0))
                .border_1().border_color(border_c()).bg(bg_tertiary())
                .text_xs().text_color(text_secondary()).child(name.to_string())
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                    this.store.update(cx, |s, cx| {
                        s.project.framework = Some(id_str.clone());
                        s.prompt_dirty = true;
                        cx.emit(StoreEvent::ProjectChanged);
                    });
                }))
            );
        }

        if !frameworks.is_empty() {
            c = c.child(div().h(px(1.0)).bg(border_c()));
            c = c.child(div().text_xs().text_color(text_muted()).child("Custom"));
            for (i, fw) in frameworks.iter().enumerate() {
                c = c.child(div().px(px(10.0)).py(px(8.0)).rounded(px(6.0))
                    .border_1().border_color(border_c()).bg(bg_tertiary())
                    .flex().items_center().gap(px(6.0))
                    .child(div().flex_1().text_xs().text_color(text_secondary())
                        .child(format!("{} ({} blocks)", fw.name, fw.blocks.len())))
                    .child(div().px(px(4.0)).py(px(2.0)).rounded(px(3.0)).text_xs().text_color(danger()).child(Icon::new(IconName::Close))
                        .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            this.store.update(cx, |s, cx| {
                                if i < s.custom_frameworks.len() { s.custom_frameworks.remove(i); }
                                cx.emit(StoreEvent::ProjectChanged);
                            });
                        })))
                );
            }
        }

        c
    }
}
