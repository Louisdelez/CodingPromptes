use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName, Theme, ThemeMode};
use crate::ui::colors::*;
use crate::state::*;

use super::{InkwellApp, rt};

/// Auth screen local state — owns the input entities instead of AppState
pub(crate) struct AuthScreenInputs {
    pub server_url: Option<Entity<InputState>>,
    pub email: Option<Entity<InputState>>,
    pub password: Option<Entity<InputState>>,
}

impl Default for AuthScreenInputs {
    fn default() -> Self {
        Self { server_url: None, email: None, password: None }
    }
}

impl InkwellApp {
    pub(crate) fn render_auth(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        // Initialize input entities (owned by auth_inputs, not AppState)
        if self.auth_inputs.server_url.is_none() {
            self.auth_inputs.server_url = Some(cx.new(|cx| {
                InputState::new(window, cx).default_value("http://localhost:8910")
            }));
        }
        if self.auth_inputs.email.is_none() {
            self.auth_inputs.email = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("email@example.com")
            }));
        }
        if self.auth_inputs.password.is_none() {
            self.auth_inputs.password = Some(cx.new(|cx| {
                InputState::new(window, cx).placeholder("Password").masked(true)
            }));
        }

        let (Some(server_input), Some(email_input), Some(password_input)) = (
            self.auth_inputs.server_url.clone(),
            self.auth_inputs.email.clone(),
            self.auth_inputs.password.clone(),
        ) else {
            return div().size_full().bg(bg_primary());
        };

        let is_login = self.state.auth_mode == AuthMode::Login;
        let lang = self.state.lang.clone();
        let is_fr = lang == "fr";

        div()
            .size_full().bg(bg_primary()).flex().flex_col()
            // Top bar: theme + lang toggles (matching web)
            .child(div().flex().justify_end().px(px(16.0)).py(px(8.0)).gap(px(8.0))
                .child(div().px(px(8.0)).py(px(4.0)).rounded(px(6.0)).bg(bg_tertiary())
                    .flex().items_center().gap(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(if self.state.dark_mode { IconName::Moon } else { IconName::Sun }).text_color(text_muted()))
                    .child(Icon::new(IconName::ChevronDown).text_color(text_muted()))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                        this.state.dark_mode = !this.state.dark_mode;
                        set_dark_mode(this.state.dark_mode);
                        Theme::change(if this.state.dark_mode { ThemeMode::Dark } else { ThemeMode::Light }, Some(window), cx);
                    })))
                .child(div().px(px(8.0)).py(px(4.0)).rounded(px(6.0)).bg(bg_tertiary())
                    .flex().items_center().gap(px(4.0)).cursor_pointer().hover(|s| s.bg(bg_hover()))
                    .child(Icon::new(IconName::Globe).text_color(text_muted()))
                    .child(div().text_xs().text_color(text_secondary()).child(if is_fr { "FR" } else { "EN" }))
                    .child(Icon::new(IconName::ChevronDown).text_color(text_muted()))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _cx| {
                        this.state.lang = if this.state.lang == "fr" { "en".into() } else { "fr".into() };
                    }))))
            // Centered auth form
            .child(div().flex_1().flex().items_center().justify_center()
                .child(div().w(px(420.0)).flex().flex_col().gap(px(20.0))
                    // Logo + title (matching web)
                    .child(div().flex().flex_col().items_center().gap(px(8.0))
                        .child(div().w(px(56.0)).h(px(56.0)).rounded(px(16.0)).bg(bg_tertiary())
                            .flex().items_center().justify_center()
                            .child(div().text_xl().text_color(accent()).child("I")))
                        .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(text_primary())
                            .child(if is_fr { "Bienvenue sur Inkwell" } else { "Welcome to Inkwell" }))
                        .child(div().text_sm().text_color(text_muted())
                            .child(if is_fr { "Votre atelier de creation de prompts IA" } else { "Your AI prompt creation workshop" })))
                    // Connexion / Inscription tabs (matching web exactly)
                    .child(div().flex().rounded(px(8.0)).bg(bg_tertiary()).p(px(2.0))
                        .child(div().flex_1().py(px(8.0)).rounded(px(6.0))
                            .bg(if is_login { accent() } else { transparent() })
                            .text_sm().text_color(if is_login { ink_white() } else { text_secondary() })
                            .flex().items_center().justify_center()
                            .child(if is_fr { "Connexion" } else { "Sign in" })
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.auth_mode = AuthMode::Login; })))
                        .child(div().flex_1().py(px(8.0)).rounded(px(6.0))
                            .bg(if !is_login { accent() } else { transparent() })
                            .text_sm().text_color(if !is_login { ink_white() } else { text_secondary() })
                            .flex().items_center().justify_center()
                            .child(if is_fr { "Inscription" } else { "Sign up" })
                            .cursor_pointer().on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| { this.state.auth_mode = AuthMode::Register; }))))
                    // Form fields (matching web: label + icon-prefixed input)
                    .child(div().flex().flex_col().gap(px(12.0))
                        // Server URL
                        .child(div().flex().flex_col().gap(px(4.0))
                            .child(div().text_xs().text_color(text_muted()).child(if is_fr { "Serveur" } else { "Server" }))
                            .child(Input::new(&server_input)))
                        // Email
                        .child(div().flex().flex_col().gap(px(4.0))
                            .child(div().text_xs().text_color(text_muted()).child("Email"))
                            .child(Input::new(&email_input)))
                        // Password
                        .child(div().flex().flex_col().gap(px(4.0))
                            .child(div().text_xs().text_color(text_muted()).child(if is_fr { "Mot de passe" } else { "Password" }))
                            .child(Input::new(&password_input))))
                    // Error message
                    .children(self.state.auth_error.clone().map(|e| {
                        div().px(px(12.0)).py(px(8.0)).rounded(px(8.0))
                            .bg(hsla(0.0, 0.75, 0.5, 0.1))
                            .flex().items_center().gap(px(6.0))
                            .child(Icon::new(IconName::TriangleAlert).text_color(danger()))
                            .child(div().text_xs().text_color(danger()).child(e))
                    }))
                    // Submit button (matching web)
                    .child(div().py(px(10.0)).bg(if self.state.auth_loading { text_muted() } else { accent() }).rounded(px(8.0))
                        .flex().items_center().justify_center()
                        .text_sm().text_color(ink_white())
                        .child(if self.state.auth_loading { if is_fr { "Connexion..." } else { "Connecting..." } }
                            else if is_login { if is_fr { "Se connecter" } else { "Sign in" } }
                            else { if is_fr { "S'inscrire" } else { "Sign up" } })
                        .cursor_pointer().hover(|s| s.bg(accent_hover()))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                if this.state.auth_loading { return; }
                                this.state.auth_error = None;

                                let server_url = this.auth_inputs.server_url.as_ref()
                                    .map(|i| i.read(cx).value().to_string())
                                    .unwrap_or_else(|| this.state.server_url.clone());
                                let email = this.auth_inputs.email.as_ref()
                                    .map(|i| i.read(cx).value().to_string())
                                    .unwrap_or_default();
                                let password = this.auth_inputs.password.as_ref()
                                    .map(|i| i.read(cx).value().to_string())
                                    .unwrap_or_default();
                                // Validate inputs
                                if email.trim().is_empty() || password.trim().is_empty() {
                                    this.state.auth_error = Some("Email et mot de passe requis".into());
                                    return;
                                }
                                this.state.auth_loading = true;
                                let tx = this.state.msg_tx.clone();
                                let is_register = this.state.auth_mode == AuthMode::Register;
                                let display_name = email.split('@').next().unwrap_or("User").to_string();

                                rt().spawn(async move {
                                        let mut client = inkwell_core::api_client::ApiClient::new(&server_url);
                                        let result = if is_register {
                                            client.register(&email, &password, &display_name).await
                                        } else {
                                            client.login(&email, &password).await
                                        };
                                        match result {
                                            Ok(session) => {
                                                client.set_token(session.token.clone());
                                                let projects = client.list_projects().await.unwrap_or_default();
                                                let workspaces = client.list_workspaces().await.unwrap_or_default();
                                                let _ = tx.send(AsyncMsg::AuthSuccess { session, projects, workspaces });
                                            }
                                            Err(e) => { let _ = tx.send(AsyncMsg::AuthError(e)); }
                                        }
                                    });
                            }))
                    )
                    // "ou" divider (matching web)
                    .child(div().flex().items_center().gap(px(12.0))
                        .child(div().flex_1().h(px(1.0)).bg(border_c()))
                        .child(div().text_xs().text_color(text_muted()).child("ou"))
                        .child(div().flex_1().h(px(1.0)).bg(border_c())))
                    // OAuth buttons (matching web: Google + GitHub)
                    .child(div().flex().flex_col().gap(px(8.0))
                        .child(div().py(px(10.0)).rounded(px(8.0)).border_1().border_color(border_c()).bg(bg_secondary())
                            .flex().items_center().justify_center().gap(px(8.0))
                            .text_sm().text_color(text_primary())
                            .child(div().text_sm().text_color(text_secondary()).child("G"))
                            .child(if is_fr { "Continuer avec Google" } else { "Continue with Google" })
                            .cursor_pointer().hover(|s| s.bg(bg_hover())))
                        .child(div().py(px(10.0)).rounded(px(8.0)).border_1().border_color(border_c()).bg(bg_secondary())
                            .flex().items_center().justify_center().gap(px(8.0))
                            .text_sm().text_color(text_primary())
                            .child(Icon::new(IconName::Github).text_color(text_secondary()))
                            .child(if is_fr { "Continuer avec GitHub" } else { "Continue with GitHub" })
                            .cursor_pointer().hover(|s| s.bg(bg_hover()))))
                    // Bottom link (matching web: "Pas encore de compte? Inscription")
                    .child(div().flex().items_center().justify_center().gap(px(4.0))
                        .child(div().text_xs().text_color(text_muted())
                            .child(if is_fr {
                                if is_login { "Pas encore de compte ?" } else { "Deja un compte ?" }
                            } else {
                                if is_login { "Don't have an account?" } else { "Already have an account?" }
                            }))
                        .child(div().text_xs().text_color(accent()).cursor_pointer()
                            .child(if is_fr {
                                if is_login { "Inscription" } else { "Connexion" }
                            } else {
                                if is_login { "Sign up" } else { "Sign in" }
                            })
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, _| {
                                this.state.auth_mode = if this.state.auth_mode == AuthMode::Login { AuthMode::Register } else { AuthMode::Login };
                            }))))
                ))
    }
}
