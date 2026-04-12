use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::store::{AppStore, StoreEvent};
use crate::ui::colors::*;

pub struct HeaderBar {
    store: Entity<AppStore>,
    editing_name: bool,
    name_input: Option<Entity<InputState>>,
    show_user_menu: bool,
    show_theme_menu: bool,
    show_lang_menu: bool,
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
        Self { store, editing_name: false, name_input: None, show_user_menu: false, show_theme_menu: false, show_lang_menu: false }
    }
}

impl Render for HeaderBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Read all data from store upfront
        let s = self.store.read(cx);
        let save_status = s.save_status;
        let project_name = s.project.name.clone();
        let _framework = s.project.framework.clone();
        let session_email = s.session.as_ref().map(|s| s.email.clone());
        let dark_mode = s.dark_mode;
        let is_fr = s.lang == "fr";
        let lang = s.lang.to_uppercase();
        let _show_settings = s.show_settings;

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
                .child({
                    let logo_path = std::env::current_exe().ok()
                        .and_then(|p| p.parent().map(|d| d.join("../assets/logo-64.png")))
                        .unwrap_or_else(|| std::path::PathBuf::from("assets/logo-64.png"));
                    // Fallback to absolute if relative doesn't exist
                    let logo_path = if logo_path.exists() { logo_path }
                        else { std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo-64.png")) };
                    img(logo_path).w(px(28.0)).h(px(28.0)).rounded(px(8.0))
                        .object_fit(gpui::ObjectFit::Contain)
                })
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
            .child({
                let save_label = match save_status {
                    "saving" => div().text_xs().text_color(warning()).flex().items_center().gap(px(4.0))
                        .child(gpui_component::spinner::Spinner::new())
                        .child(if is_fr { "Sauvegarde..." } else { "Saving..." }),
                    "saved" => div().flex().items_center().gap(px(4.0)).text_xs().text_color(success())
                        .child(Icon::new(IconName::Check))
                        .child(if is_fr { "Sauvegarde" } else { "Saved" }),
                    _ => div(),
                };
                if save_status == "saved" {
                    div().child(save_label).with_animation(
                        "save-ok",
                        Animation::new(std::time::Duration::from_millis(400))
                            .with_easing(gpui_component::animation::cubic_bezier(0.25, 0.1, 0.25, 1.0)),
                        |this, delta| this.opacity(0.4 + delta * 0.6),
                    )
                } else {
                    div().child(save_label).with_animation(
                        "save-idle",
                        Animation::new(std::time::Duration::from_millis(1)),
                        |this, _| this,
                    )
                }
            });

        // Right section: theme dropdown, lang dropdown, user menu (no panel toggles)
        let show_theme = self.show_theme_menu;
        let show_lang = self.show_lang_menu;

        // Theme dropdown
        let mut theme_dd = div()
            .child(div().px(px(8.0)).py(px(4.0)).rounded(px(6.0)).flex().items_center().gap(px(4.0))
                .bg(bg_tertiary()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                .child(Icon::new(if dark_mode { IconName::Moon } else { IconName::Sun }).text_color(text_muted()))
                .child(Icon::new(IconName::ChevronDown).text_color(text_muted()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.show_theme_menu = !this.show_theme_menu;
                    this.show_lang_menu = false; this.show_user_menu = false; cx.notify();
                })));
        if show_theme {
            theme_dd = theme_dd.child(deferred(anchored().snap_to_window_with_margin(px(8.0)).child(
                div().mt(px(4.0)).w(px(140.0)).rounded(px(8.0)).bg(bg_secondary())
                    .border_1().border_color(border_c()).p(px(4.0)).flex().flex_col().gap(px(2.0))
                    .child(div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                        .text_xs().cursor_pointer().hover(|s| s.bg(bg_hover()))
                        .text_color(if !dark_mode { accent() } else { text_primary() })
                        .bg(if !dark_mode { accent_bg() } else { transparent() })
                        .child(Icon::new(IconName::Sun)).child("Light")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.show_theme_menu = false;
                            this.store.update(cx, |s, cx| { s.dark_mode = false; cx.emit(StoreEvent::SettingsChanged); });
                        })))
                    .child(div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                        .text_xs().cursor_pointer().hover(|s| s.bg(bg_hover()))
                        .text_color(if dark_mode { accent() } else { text_primary() })
                        .bg(if dark_mode { accent_bg() } else { transparent() })
                        .child(Icon::new(IconName::Moon)).child("Dark")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.show_theme_menu = false;
                            this.store.update(cx, |s, cx| { s.dark_mode = true; cx.emit(StoreEvent::SettingsChanged); });
                        })))
            )).with_priority(1));
        }

        // Language dropdown
        let mut lang_dd = div()
            .child(div().px(px(8.0)).py(px(4.0)).rounded(px(6.0)).flex().items_center().gap(px(4.0))
                .bg(bg_tertiary()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                .child(Icon::new(IconName::Globe).text_color(text_muted()))
                .child(div().text_xs().text_color(text_secondary()).child(lang))
                .child(Icon::new(IconName::ChevronDown).text_color(text_muted()))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.show_lang_menu = !this.show_lang_menu;
                    this.show_theme_menu = false; this.show_user_menu = false; cx.notify();
                })));
        if show_lang {
            lang_dd = lang_dd.child(deferred(anchored().snap_to_window_with_margin(px(8.0)).child(
                div().mt(px(4.0)).w(px(140.0)).rounded(px(8.0)).bg(bg_secondary())
                    .border_1().border_color(border_c()).p(px(4.0)).flex().flex_col().gap(px(2.0))
                    .child(div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                        .text_xs().cursor_pointer().hover(|s| s.bg(bg_hover()))
                        .text_color(if is_fr { accent() } else { text_primary() })
                        .bg(if is_fr { accent_bg() } else { transparent() })
                        .child("Francais")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.show_lang_menu = false;
                            this.store.update(cx, |s, cx| { s.lang = "fr".into(); cx.emit(StoreEvent::SettingsChanged); });
                        })))
                    .child(div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                        .text_xs().cursor_pointer().hover(|s| s.bg(bg_hover()))
                        .text_color(if !is_fr { accent() } else { text_primary() })
                        .bg(if !is_fr { accent_bg() } else { transparent() })
                        .child("English")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.show_lang_menu = false;
                            this.store.update(cx, |s, cx| { s.lang = "en".into(); cx.emit(StoreEvent::SettingsChanged); });
                        })))
            )).with_priority(1));
        }

        // User menu
        let show_menu = self.show_user_menu;
        let mut user_dd = div();
        if let Some(email) = session_email {
            let s = self.store.read(cx);
            let display_name = s.session.as_ref().map(|s| s.display_name.clone()).unwrap_or(email.clone());
            let initial = display_name.chars().next().unwrap_or('U').to_uppercase().to_string();
            let dn = display_name.clone(); let em = email.clone(); let ini = initial.clone();
            user_dd = user_dd
                .child(div().px(px(6.0)).py(px(4.0)).rounded(px(6.0)).flex().items_center().gap(px(6.0))
                    .cursor_pointer().hover(|s| s.bg(bg_hover()))
                    .child(div().w(px(24.0)).h(px(24.0)).rounded(px(12.0)).bg(accent())
                        .flex().items_center().justify_center().text_xs().text_color(ink_white()).child(initial))
                    .child(div().text_xs().text_color(text_secondary()).child(display_name))
                    .child(Icon::new(IconName::ChevronDown).text_color(text_muted()))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.show_user_menu = !this.show_user_menu;
                        this.show_theme_menu = false; this.show_lang_menu = false; cx.notify();
                    })));
            if show_menu {
                user_dd = user_dd.child(deferred(anchored().snap_to_window_with_margin(px(8.0)).child(
                    div().mt(px(4.0)).w(px(200.0)).rounded(px(8.0)).bg(bg_secondary())
                        .border_1().border_color(border_c()).p(px(8.0)).flex().flex_col().gap(px(4.0))
                        .child(div().p(px(8.0)).flex().items_center().gap(px(8.0))
                            .child(div().w(px(32.0)).h(px(32.0)).rounded(px(16.0)).bg(accent())
                                .flex().items_center().justify_center().text_xs().text_color(ink_white()).child(ini))
                            .child(div().flex().flex_col()
                                .child(div().text_xs().font_weight(FontWeight::MEDIUM).text_color(text_primary()).child(dn))
                                .child(div().text_xs().text_color(text_muted()).child(em))))
                        .child(div().h(px(1.0)).bg(border_c()))
                        .child(div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                            .text_xs().text_color(text_secondary()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                            .child(Icon::new(IconName::User)).child("Profil")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.show_user_menu = false;
                                this.store.update(cx, |s, cx| { s.show_profile = !s.show_profile; cx.emit(StoreEvent::SettingsChanged); });
                            })))
                        .child(div().px(px(8.0)).py(px(6.0)).rounded(px(4.0)).flex().items_center().gap(px(6.0))
                            .text_xs().text_color(danger()).cursor_pointer().hover(|s| s.bg(bg_hover()))
                            .child(Icon::new(IconName::LogOut)).child("Deconnexion")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.show_user_menu = false;
                                this.store.update(cx, |s, cx| { s.session = None; s.screen = crate::state::Screen::Auth; cx.emit(StoreEvent::SessionChanged); });
                            })))
                )).with_priority(1));
            }
        }

        let right = div().flex().items_center().gap(px(4.0))
            .child(theme_dd)
            .child(lang_dd)
            .child(div().w(px(1.0)).h(px(16.0)).bg(border_c()))
            .child(user_dd);

        div().h(px(44.0)).px(px(12.0)).flex().items_center()
            .border_b_1().border_color(border_c()).bg(bg_secondary())
            .child(left).child(div().flex_1()).child(right)
    }
}
