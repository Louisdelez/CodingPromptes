use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::state::*;
use crate::ui::colors::*;

pub struct HeaderBar {
    store: Entity<AppStore>,
    editing_name: bool,
    name_input: Option<Entity<InputState>>,
}

impl HeaderBar {
    pub fn new(store: Entity<AppStore>, cx: &mut Context<Self>) -> Self {
        cx.subscribe(&store, |_this, _, event: &StoreEvent, cx| {
            match event {
                StoreEvent::SaveStatusChanged | StoreEvent::ProjectChanged |
                StoreEvent::SessionChanged | StoreEvent::SettingsChanged => cx.notify(),
                _ => {}
            }
        }).detach();
        Self { store, editing_name: false, name_input: None }
    }
}

impl Render for HeaderBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Read all data from store upfront
        let s = self.store.read(cx);
        let save_status = s.save_status;
        let project_name = s.project.name.clone();
        let framework = s.project.framework.clone();
        let session_email = s.session.as_ref().map(|s| s.email.clone());
        let dark_mode = s.dark_mode;
        let lang = s.lang.to_uppercase();
        let show_settings = s.show_settings;
        let left_open = s.left_open;
        let right_open = s.right_open;
        drop(s);

        // Init name input
        if self.editing_name && self.name_input.is_none() {
            let name = project_name.clone();
            self.name_input = Some(cx.new(|cx| InputState::new(window, cx).default_value(name)));
        }

        let name_input_clone = self.name_input.clone();
        let editing = self.editing_name;

        div().h(px(40.0)).px(px(12.0)).flex().items_center().gap(px(8.0))
            .border_b_1().border_color(border_c()).bg(bg_secondary())
            .child(div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                .text_color(if left_open { text_secondary() } else { text_muted() })
                .child(if left_open { "[<]" } else { "[>]" })
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| { s.left_open = !s.left_open; cx.emit(StoreEvent::SettingsChanged); });
                })))
            .child(div().text_sm().text_color(accent()).child("Inkwell"))
            .child(div().w(px(1.0)).h(px(16.0)).bg(border_c()))
            // Project name
            .child(if editing {
                match name_input_clone {
                    Some(ref entity) => div().flex().items_center().gap(px(4.0))
                        .child(div().w(px(180.0)).child(Input::new(entity)))
                        .child(div().px(px(4.0)).py(px(2.0)).rounded(px(3.0))
                            .text_xs().text_color(success()).child(Icon::new(IconName::Check))
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                if let Some(ref entity) = this.name_input {
                                    let new_name = entity.read(cx).value().to_string();
                                    if !new_name.trim().is_empty() {
                                        this.store.update(cx, |s, cx| {
                                            s.project.name = new_name.trim().to_string();
                                            if let Some(p) = s.projects.iter_mut().find(|p| p.id == s.project.id) { p.name = s.project.name.clone(); }
                                            s.save_pending = true;
                                            cx.emit(StoreEvent::ProjectChanged);
                                        });
                                    }
                                }
                                this.editing_name = false;
                                this.name_input = None;
                                cx.notify();
                            }))),
                    None => div(),
                }
            } else {
                div().text_sm().text_color(text_primary()).child(project_name)
                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.editing_name = true; this.name_input = None; cx.notify();
                    }))
            })
            .child(match save_status {
                "saving" => div().text_xs().text_color(hsla(50.0 / 360.0, 0.8, 0.5, 1.0)).child("Saving..."),
                "saved" => div().text_xs().text_color(success()).child("Saved"),
                _ => div(),
            })
            .child(div().flex_1())
            .children(framework.map(|f| div().px(px(6.0)).py(px(2.0)).rounded(px(4.0))
                .bg(hsla(239.0 / 360.0, 0.84, 0.67, 0.1)).text_xs().text_color(accent()).child(f)))
            .children(session_email.map(|email| {
                let initial = email.chars().next().unwrap_or('U').to_uppercase().to_string();
                div().px(px(6.0)).py(px(2.0)).rounded(px(4.0)).flex().items_center().gap(px(4.0))
                    .child(div().w(px(18.0)).h(px(18.0)).rounded(px(9.0)).bg(accent())
                        .flex().items_center().justify_center().text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0)).child(initial))
                    .child(div().text_xs().text_color(text_muted()).child(email))
                    .cursor_pointer().hover(|s| s.bg(bg_tertiary()))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.store.update(cx, |s, cx| { s.show_profile = !s.show_profile; cx.emit(StoreEvent::SettingsChanged); });
                    }))
            }))
            .child(div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs().text_color(text_muted()).child(lang)
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| { s.lang = if s.lang == "fr" { "en".into() } else { "fr".into() }; cx.emit(StoreEvent::SettingsChanged); });
                })))
            .child(div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                .text_color(if show_settings { accent() } else { text_muted() }).child(Icon::new(IconName::Settings))
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| { s.show_settings = !s.show_settings; cx.emit(StoreEvent::SettingsChanged); });
                })))
            .child(div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs().text_color(text_muted())
                .child(Icon::new(if dark_mode { IconName::Moon } else { IconName::Sun }))
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| { s.dark_mode = !s.dark_mode; cx.emit(StoreEvent::SettingsChanged); });
                })))
            .child(div().text_xs().text_color(success()).child("GPUI"))
            .child(div().px(px(6.0)).py(px(4.0)).rounded(px(4.0)).text_xs()
                .text_color(if right_open { text_secondary() } else { text_muted() })
                .child(if right_open { "[>]" } else { "[<]" })
                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| { s.right_open = !s.right_open; cx.emit(StoreEvent::SettingsChanged); });
                })))
    }
}
