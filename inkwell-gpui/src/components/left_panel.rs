use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;

pub struct LeftPanel {
    store: Entity<AppStore>,
    search_input: Option<Entity<InputState>>,
    show_frameworks: bool,
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
        Self { store, search_input, show_frameworks: false }
    }
}

impl Render for LeftPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.store.read(cx);
        let projects: Vec<ProjectSummary> = s.projects.clone();
        let workspaces: Vec<inkwell_core::types::Workspace> = s.workspaces.clone();
        let current_id = s.project.id.clone();
        let confirm_delete = s.confirm_delete.clone();
        let custom_fw: Vec<CustomFramework> = s.custom_frameworks.clone();
        drop(s);

        let show_fw = self.show_frameworks;

        div().w(px(250.0)).flex_shrink_0().border_r_1().border_color(border_c()).bg(bg_secondary())
            .flex().flex_col()
            // Header: "Bibliotheque" with chevron (like Tauri)
            .child(
                div().h(px(44.0)).px(px(16.0)).flex().items_center().gap(px(8.0))
                    .border_b_1().border_color(border_c())
                    .child(Icon::new(IconName::FolderOpen).text_color(text_muted()))
                    .child(div().flex_1().text_sm().text_color(text_primary())
                        .child(if show_fw { "Frameworks" } else { "Bibliotheque" }))
                    .child(
                        div().text_xs().text_color(text_muted())
                            .child(Icon::new(IconName::ChevronDown))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.show_frameworks = !this.show_frameworks;
                                cx.notify();
                            }))
                    )
            )
            // Search bar with icons (like Tauri: loupe + folder + plus)
            .child(
                div().px(px(12.0)).py(px(8.0)).flex().items_center().gap(px(6.0))
                    .child(Icon::new(IconName::Search).text_color(text_muted()))
                    .child({
                        if let Some(ref entity) = self.search_input {
                            div().flex_1().child(Input::new(entity))
                        } else {
                            div().flex_1()
                        }
                    })
                    // New folder button
                    .child(
                        div().px(px(4.0)).py(px(2.0)).rounded(px(4.0))
                            .child(Icon::new(IconName::FolderOpen).text_color(text_muted()))
                            .cursor_pointer().hover(|s| s.bg(accent_bg()))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.store.update(cx, |s, cx| {
                                    let name = format!("Workspace {}", s.workspaces.len() + 1);
                                    s.workspaces.push(inkwell_core::types::Workspace {
                                        id: uuid::Uuid::new_v4().to_string(), name, description: String::new(),
                                        color: "#6366f1".into(), constitution: None,
                                        created_at: chrono::Utc::now().timestamp_millis(),
                                        updated_at: chrono::Utc::now().timestamp_millis(),
                                    });
                                    cx.emit(StoreEvent::ProjectChanged);
                                });
                            }))
                    )
                    // New prompt button
                    .child(
                        div().px(px(4.0)).py(px(2.0)).rounded(px(4.0))
                            .child(Icon::new(IconName::Plus).text_color(text_muted()))
                            .cursor_pointer().hover(|s| s.bg(accent_bg()))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.store.update(cx, |s, cx| {
                                    let new_proj = Project::default_prompt();
                                    s.projects.push(ProjectSummary { id: new_proj.id.clone(), name: new_proj.name.clone() });
                                    s.project = new_proj;
                                    s.prompt_dirty = true;
                                    s.save_pending = true;
                                    cx.emit(StoreEvent::ProjectChanged);
                                });
                            }))
                    )
            )
            // Content
            .child(if show_fw {
                self.render_frameworks(&custom_fw, cx)
            } else {
                self.render_library(&projects, &workspaces, &current_id, &confirm_delete, cx)
            })
    }
}

impl LeftPanel {
    fn render_library(&self, projects: &[ProjectSummary], workspaces: &[inkwell_core::types::Workspace],
                      current_id: &str, confirm_delete: &Option<String>, cx: &mut Context<Self>) -> Div {
        let mut c = div().flex_1().px(px(12.0)).py(px(4.0)).flex().flex_col().gap(px(2.0));

        // Workspaces (only show if there are any)
        if !workspaces.is_empty() {
            for ws in workspaces {
                let color = hex_to_hsla(&ws.color);
                let ws_id = ws.id.clone();
                c = c.child(
                    div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                        .hover(|s| s.bg(bg_tertiary()))
                        .child(div().w(px(8.0)).h(px(8.0)).rounded(px(4.0)).bg(color))
                        .child(div().flex_1().text_xs().text_color(text_primary()).child(ws.name.clone()))
                        .child(div().text_xs().text_color(danger()).child(Icon::new(IconName::Close))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                this.store.update(cx, |s, cx| { s.workspaces.retain(|w| w.id != ws_id); cx.emit(StoreEvent::ProjectChanged); });
                            })))
                );
            }
            c = c.child(div().h(px(1.0)).bg(border_c()).my(px(4.0)));
        }

        // Project list
        let search = self.search_input.as_ref().map(|i| i.read(cx).value().to_string().to_lowercase()).unwrap_or_default();
        let filtered: Vec<&ProjectSummary> = projects.iter().filter(|p| search.is_empty() || p.name.to_lowercase().contains(&search)).collect();

        for p in &filtered {
            let id = p.id.clone();
            let del_id = p.id.clone();
            let is_active = current_id == p.id;
            c = c.child(
                div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(4.0))
                    .bg(if is_active { bg_tertiary() } else { hsla(0.0, 0.0, 0.0, 0.0) })
                    .hover(|s| s.bg(bg_tertiary()))
                    .child(div().flex_1().text_xs()
                        .text_color(if is_active { text_primary() } else { text_secondary() })
                        .child(p.name.clone())
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

        // Delete confirm dialog
        if let Some(ref del_id) = confirm_delete {
            let del = del_id.clone();
            c = c.child(
                div().p(px(10.0)).rounded(px(8.0)).bg(hsla(0.0, 0.75, 0.55, 0.1)).border_1().border_color(danger())
                    .flex().flex_col().gap(px(6.0))
                    .child(div().text_xs().text_color(danger()).child("Supprimer ce projet ?"))
                    .child(div().flex().gap(px(6.0))
                        .child(div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(danger()).text_xs().text_color(white()).child("Supprimer")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                let id = del.clone();
                                crate::persistence::delete_project(&id);
                                this.store.update(cx, |s, cx| { s.projects.retain(|p| p.id != id); s.confirm_delete = None; cx.emit(StoreEvent::ProjectChanged); });
                            })))
                        .child(div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(bg_tertiary()).text_xs().text_color(text_secondary()).child("Annuler")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.store.update(cx, |s, _| { s.confirm_delete = None; }); cx.notify();
                            }))))
            );
        }

        // Empty state (like Tauri: message + explanation)
        if filtered.is_empty() && projects.is_empty() {
            c = c.child(
                div().py(px(16.0)).flex().flex_col().items_center().gap(px(8.0))
                    .child(div().text_xs().text_color(text_muted()).child("Rien ici encore"))
                    .child(div().text_xs().text_color(text_muted()).text_center()
                        .child("Creez un projet (dossier) pour organiser vos prompts, ou un prompt libre."))
            );
        } else if filtered.is_empty() {
            c = c.child(div().text_xs().text_color(text_muted()).child("Aucun projet correspondant"));
        }

        c
    }

    fn render_frameworks(&self, frameworks: &[CustomFramework], cx: &mut Context<Self>) -> Div {
        const BUILT_IN: &[(&str, &str)] = &[
            ("CO-STAR", "co-star"), ("RISEN", "risen"), ("RACE", "race"),
            ("SDD (Spec-Driven)", "sdd"), ("APE", "ape"), ("STOKE", "stoke"),
        ];
        let mut c = div().flex_1().px(px(12.0)).py(px(4.0)).flex().flex_col().gap(px(2.0));

        for &(name, id) in BUILT_IN {
            let id_str = id.to_string();
            c = c.child(
                div().px(px(10.0)).py(px(8.0)).rounded(px(6.0))
                    .border_1().border_color(border_c()).bg(bg_tertiary())
                    .text_xs().text_color(text_secondary()).child(name.to_string())
                    .cursor_pointer().hover(|s| s.bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.1)))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.store.update(cx, |s, cx| {
                            s.project.framework = Some(id_str.clone());
                            s.prompt_dirty = true;
                            cx.emit(StoreEvent::ProjectChanged);
                        });
                    }))
            );
        }

        if !frameworks.is_empty() {
            c = c.child(div().h(px(1.0)).bg(border_c()).my(px(4.0)));
            c = c.child(div().text_xs().text_color(text_muted()).child("Custom"));
            for (i, fw) in frameworks.iter().enumerate() {
                c = c.child(
                    div().px(px(10.0)).py(px(8.0)).rounded(px(6.0))
                        .border_1().border_color(border_c()).bg(bg_tertiary())
                        .flex().items_center().gap(px(6.0))
                        .child(div().flex_1().text_xs().text_color(text_secondary())
                            .child(format!("{} ({} blocks)", fw.name, fw.blocks.len())))
                        .child(div().text_xs().text_color(danger()).child(Icon::new(IconName::Close))
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
