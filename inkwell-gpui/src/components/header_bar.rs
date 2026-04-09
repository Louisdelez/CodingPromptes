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
    show_user_menu: bool,
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
        Self { store, editing_name: false, name_input: None, show_user_menu: false }
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

        // Left section: logo + project name + save status
        let left = div().flex().items_center().gap(px(8.0))
            .child(div().flex().items_center().gap(px(6.0))
                .child(div().w(px(28.0)).h(px(28.0)).rounded(px(8.0)).bg(bg_tertiary())
                    .flex().items_center().justify_center()
                    .child(Icon::new(IconName::PenTool).text_color(accent())))
                .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).text_color(text_primary()).child("Inkwell")))
            .child(div().w(px(1.0)).h(px(16.0)).bg(border_c()))
            // Project name (editable)
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
                    .cursor_pointer().hover(|s| s.bg(bg_tertiary()).rounded(px(4.0)))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.editing_name = true; this.name_input = None; cx.notify();
                    }))
            })
            .child(match save_status {
                "saving" => div().text_xs().text_color(warning()).child("Saving..."),
                "saved" => div().flex().items_center().gap(px(2.0)).text_xs().text_color(success())
                    .child(Icon::new(IconName::Check)).child("Saved"),
                _ => div(),
            });

        // Right section: theme, lang, user, panel toggles
        let right = div().flex().items_center().gap(px(4.0))
            // Theme toggle (icon + dropdown style like web)
            .child(div().px(px(8.0)).py(px(4.0)).rounded(px(6.0)).flex().items_center().gap(px(4.0))
                .bg(bg_tertiary()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                .child(Icon::new(if dark_mode { IconName::Moon } else { IconName::Sun }).text_color(text_muted()))
                .child(Icon::new(IconName::ChevronDown).text_color(text_muted()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| { s.dark_mode = !s.dark_mode; cx.emit(StoreEvent::SettingsChanged); });
                })))
            // Language toggle (globe + FR/EN like web)
            .child(div().px(px(8.0)).py(px(4.0)).rounded(px(6.0)).flex().items_center().gap(px(4.0))
                .bg(bg_tertiary()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                .child(Icon::new(IconName::Globe).text_color(text_muted()))
                .child(div().text_xs().text_color(text_secondary()).child(lang))
                .child(Icon::new(IconName::ChevronDown).text_color(text_muted()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| { s.lang = if s.lang == "fr" { "en".into() } else { "fr".into() }; cx.emit(StoreEvent::SettingsChanged); });
                })))
            // Divider
            .child(div().w(px(1.0)).h(px(16.0)).bg(border_c()))
            // User avatar + name + floating dropdown (matching web)
            .children(session_email.map(|email| {
                let s = self.store.read(cx);
                let display_name = s.session.as_ref().map(|s| s.display_name.clone()).unwrap_or(email.clone());
                let initial = display_name.chars().next().unwrap_or('U').to_uppercase().to_string();
                drop(s);
                let show_menu = self.show_user_menu;
                let dn = display_name.clone(); let em = email.clone(); let ini = initial.clone();
                let mut user_section = div()
                    // Trigger button
                    .child(div().px(px(6.0)).py(px(4.0)).rounded(px(6.0)).flex().items_center().gap(px(6.0))
                        .cursor_pointer().hover(|s| s.bg(bg_hover()))
                        .child(div().w(px(24.0)).h(px(24.0)).rounded(px(12.0)).bg(accent())
                            .flex().items_center().justify_center().text_xs().text_color(ink_white()).child(initial))
                        .child(div().text_xs().text_color(text_secondary()).child(display_name))
                        .child(Icon::new(IconName::ChevronDown).text_color(text_muted()))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.show_user_menu = !this.show_user_menu; cx.notify();
                        })));
                // Floating dropdown menu
                if show_menu {
                    user_section = user_section.child(
                        div().mt(px(4.0))
                            .w(px(200.0)).rounded(px(8.0)).bg(bg_secondary())
                            .border_1().border_color(border_c())
                            .p(px(8.0)).flex().flex_col().gap(px(4.0))
                            // Avatar + name + email
                            .child(div().p(px(8.0)).flex().items_center().gap(px(8.0))
                                .child(div().w(px(32.0)).h(px(32.0)).rounded(px(16.0)).bg(accent())
                                    .flex().items_center().justify_center().text_xs().text_color(ink_white()).child(ini))
                                .child(div().flex().flex_col()
                                    .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_primary()).child(dn))
                                    .child(div().text_xs().text_color(text_muted()).child(em))))
                            // Separator
                            .child(div().h(px(1.0)).bg(border_c()))
                            // Profil
                            .child(div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                                .text_xs().text_color(text_secondary()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                                .child(Icon::new(IconName::User)).child("Profil")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                    this.show_user_menu = false;
                                    this.store.update(cx, |s, cx| { s.show_profile = !s.show_profile; cx.emit(StoreEvent::SettingsChanged); });
                                })))
                            // Deconnexion
                            .child(div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                                .text_xs().text_color(danger()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                                .child(Icon::new(IconName::LogOut)).child("Deconnexion")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                    this.show_user_menu = false;
                                    this.store.update(cx, |s, cx| { s.session = None; s.screen = crate::state::Screen::Auth; cx.emit(StoreEvent::SessionChanged); });
                                })))
                    );
                }
                user_section
            }))
            // Divider
            .child(div().w(px(1.0)).h(px(16.0)).bg(border_c()))
            // Left panel toggle (PanelLeftOpen/Close icons like web)
            .child(div().p(px(6.0)).rounded(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_hover()))
                .child(Icon::new(if left_open { IconName::PanelLeftClose } else { IconName::PanelLeftOpen }).text_color(text_muted()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| { s.left_open = !s.left_open; cx.emit(StoreEvent::SettingsChanged); });
                })))
            // Right panel toggle
            .child(div().p(px(6.0)).rounded(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_hover()))
                .child(Icon::new(if right_open { IconName::PanelRightClose } else { IconName::PanelRightOpen }).text_color(text_muted()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.store.update(cx, |s, cx| { s.right_open = !s.right_open; cx.emit(StoreEvent::SettingsChanged); });
                })));

        div().h(px(44.0)).px(px(12.0)).flex().items_center()
            .border_b_1().border_color(border_c()).bg(bg_secondary())
            .child(left).child(div().flex_1()).child(right)
    }
}
