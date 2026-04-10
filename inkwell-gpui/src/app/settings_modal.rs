use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName};
use crate::ui::colors::*;
use crate::state::*;

use super::InkwellApp;

/// Settings modal local state — owns API key input entities
pub(crate) struct SettingsInputs {
    pub openai: Option<Entity<InputState>>,
    pub anthropic: Option<Entity<InputState>>,
    pub google: Option<Entity<InputState>>,
    pub github_repo: Option<Entity<InputState>>,
    pub ssh_port: Option<Entity<InputState>>,
}

impl Default for SettingsInputs {
    fn default() -> Self {
        Self { openai: None, anthropic: None, google: None, github_repo: None, ssh_port: None }
    }
}

impl InkwellApp {
    pub(crate) fn render_settings(&self, cx: &mut Context<Self>) -> Div {
        let lang = self.store.read(cx).lang.clone();
        // Modal overlay (matching web: centered card over backdrop)
        div().size_full().absolute().top_0().left_0()
            .bg(hsla(0.0, 0.0, 0.0, 0.4))
            .flex().items_center().justify_center()
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.state.show_settings = false;
                this.store.update(cx, |s, cx| { s.show_settings = false; cx.emit(crate::store::StoreEvent::SettingsChanged); });
            }))
            .child(div().w(px(480.0)).max_h(px(600.0))
                .rounded(px(12.0)).bg(bg_secondary())
                .border_1().border_color(border_c())
                .p(px(24.0)).flex().flex_col().gap(px(16.0))
                .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, _| { /* stop propagation */ }))
            .child(
                div().flex().items_center().gap(px(8.0))
                    .child(div().text_sm().text_color(text_primary()).child(Icon::new(IconName::Settings)))
                    .child(div().flex_1())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                            .text_xs().text_color(text_muted()).child(Icon::new(IconName::Close)).child("Close")
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.state.show_settings = false;
                            }))
                    )
            )
            .child(
                div().flex().gap(px(16.0))
                    // Language
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("Language"))
                        .child(div().flex().gap(px(4.0))
                            .child(
                                div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                                    .bg(if lang == "fr" { accent() } else { bg_tertiary() })
                                    .text_xs().text_color(if lang == "fr" { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                                    .child("Francais")
                                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.lang = "fr".into(); }))
                            )
                            .child(
                                div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                                    .bg(if lang == "en" { accent() } else { bg_tertiary() })
                                    .text_xs().text_color(if lang == "en" { gpui::hsla(0.0, 0.0, 1.0, 1.0) } else { text_muted() })
                                    .child("English")
                                    .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.lang = "en".into(); }))
                            )
                        )
                    )
                    // Server URL
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("Server URL"))
                        .child(div().h(px(28.0)).px(px(8.0)).bg(bg_tertiary()).rounded(px(4.0))
                            .border_1().border_color(border_c())
                            .flex().items_center().text_xs().text_color(text_secondary())
                            .child(self.store.read(cx).server_url.clone()))
                    )
                    // API Keys
                    .child(div().flex().flex_col().gap(px(6.0))
                        .child(div().text_xs().text_color(text_muted()).child("API Keys"))
                        .child(div().flex().items_center().gap(px(6.0))
                            .child(div().w(px(60.0)).text_xs().text_color(text_muted()).child("OpenAI"))
                            .child({
                                if let Some(ref entity) = self.settings_inputs.openai {
                                    div().flex_1().child(Input::new(entity))
                                } else {
                                    div().flex_1().text_xs().text_color(text_muted()).child("not set")
                                }
                            })
                        )
                        .child(div().flex().items_center().gap(px(6.0))
                            .child(div().w(px(60.0)).text_xs().text_color(text_muted()).child("Anthropic"))
                            .child({
                                if let Some(ref entity) = self.settings_inputs.anthropic {
                                    div().flex_1().child(Input::new(entity))
                                } else {
                                    div().flex_1().text_xs().text_color(text_muted()).child("not set")
                                }
                            })
                        )
                        .child(div().flex().items_center().gap(px(6.0))
                            .child(div().w(px(60.0)).text_xs().text_color(text_muted()).child("Google"))
                            .child({
                                if let Some(ref entity) = self.settings_inputs.google {
                                    div().flex_1().child(Input::new(entity))
                                } else {
                                    div().flex_1().text_xs().text_color(text_muted()).child("not set")
                                }
                            })
                        )
                        .child(
                            div().px(px(8.0)).py(px(4.0)).rounded(px(4.0)).bg(accent())
                                .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                                .flex().items_center().justify_center().child("Save keys")
                                .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                    if let Some(ref e) = this.settings_inputs.openai {
                                        let v = e.read(cx).value().to_string();
                                        if !v.is_empty() { this.state.api_key_openai = v; }
                                    }
                                    if let Some(ref e) = this.settings_inputs.anthropic {
                                        let v = e.read(cx).value().to_string();
                                        if !v.is_empty() { this.state.api_key_anthropic = v; }
                                    }
                                    if let Some(ref e) = this.settings_inputs.google {
                                        let v = e.read(cx).value().to_string();
                                        if !v.is_empty() { this.state.api_key_google = v; }
                                    }
                                }))
                        )
                    )
                    // GitHub repo
                    .child(div().flex().flex_col().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted()).child("GitHub Repo"))
                        .child({
                            if let Some(ref entity) = self.settings_inputs.github_repo {
                                div().child(Input::new(entity))
                            } else {
                                div().text_xs().text_color(text_muted()).child("owner/repo")
                            }
                        })
                    )
            )
            .child(
                div().flex().gap(px(8.0))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+, settings"))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+N new"))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+` terminal"))
                    .child(div().text_xs().text_color(text_muted()).child("Ctrl+Enter run"))
                    .child(div().flex_1())
                    .child(
                        div().px(px(8.0)).py(px(4.0)).rounded(px(4.0))
                            .bg(if self.store.read(cx).session.is_some() { danger() } else { accent() })
                            .text_xs().text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
                            .child(if self.store.read(cx).session.is_some() { "Deconnecter sync" } else { "Connecter sync" })
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                if this.state.session.is_some() {
                                    // Disconnect sync (keep local data)
                                    this.state.session = None;
                                } else {
                                    // Go to auth to connect
                                    this.state.screen = Screen::Auth;
                                }
                                crate::persistence::save_session(&crate::persistence::SavedSession {
                                    server_url: this.state.server_url.clone(),
                                    token: String::new(),
                                    email: String::new(),
                                    dark_mode: this.state.dark_mode,
                                    lang: this.state.lang.clone(),
                                    last_project_id: None,
                                    left_open: this.state.left_open,
                                    right_open: this.state.right_open,
                                });
                            }))
                    )
            ))
    }

}
