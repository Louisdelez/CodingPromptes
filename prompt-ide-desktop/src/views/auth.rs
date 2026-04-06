use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

use crate::services::i18n::I18n;

#[derive(Debug, Clone)]
pub enum AuthMessage {
    EmailChanged(String),
    PasswordChanged(String),
    ConfirmPasswordChanged(String),
    DisplayNameChanged(String),
    SwitchMode,
    Submit,
}

pub struct AuthView {
    pub mode: AuthMode,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub display_name: String,
    pub error: Option<String>,
    pub loading: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthMode {
    Login,
    Register,
}

impl AuthView {
    pub fn new() -> Self {
        Self {
            mode: AuthMode::Login,
            email: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            display_name: String::new(),
            error: None,
            loading: false,
        }
    }

    pub fn view(&self, i18n: &I18n) -> Element<AuthMessage> {
        let title = text(i18n.t("auth.welcome")).size(24);
        let subtitle = text(i18n.t("auth.subtitle"))
            .size(13)
            .color(iced::Color::from_rgb(0.6, 0.6, 0.65));

        let tab_login = button(text(i18n.t("auth.login")).size(14))
            .on_press(if self.mode != AuthMode::Login { AuthMessage::SwitchMode } else { AuthMessage::Submit })
            .style(if self.mode == AuthMode::Login { button::primary } else { button::secondary });

        let tab_register = button(text(i18n.t("auth.register")).size(14))
            .on_press(if self.mode != AuthMode::Register { AuthMessage::SwitchMode } else { AuthMessage::Submit })
            .style(if self.mode == AuthMode::Register { button::primary } else { button::secondary });

        let tabs = row![tab_login, tab_register].spacing(4);

        let mut form = column![].spacing(12).width(Length::Fixed(320.0));

        if self.mode == AuthMode::Register {
            form = form.push(
                text_input(i18n.t("auth.display_name"), &self.display_name)
                    .on_input(AuthMessage::DisplayNameChanged)
                    .size(14),
            );
        }

        form = form.push(
            text_input(i18n.t("auth.email"), &self.email)
                .on_input(AuthMessage::EmailChanged)
                .size(14),
        );

        form = form.push(
            text_input(i18n.t("auth.password"), &self.password)
                .on_input(AuthMessage::PasswordChanged)
                .secure(true)
                .size(14),
        );

        if self.mode == AuthMode::Register {
            form = form.push(
                text_input(i18n.t("auth.confirm_password"), &self.confirm_password)
                    .on_input(AuthMessage::ConfirmPasswordChanged)
                    .secure(true)
                    .size(14),
            );
        }

        if let Some(ref err) = self.error {
            form = form.push(
                text(err.as_str())
                    .size(12)
                    .color(iced::Color::from_rgb(0.97, 0.26, 0.26)),
            );
        }

        let btn_label = if self.mode == AuthMode::Login {
            i18n.t("auth.login_button")
        } else {
            i18n.t("auth.register_button")
        };

        let submit = button(text(btn_label).size(14))
            .on_press(AuthMessage::Submit)
            .width(Length::Fill)
            .style(button::primary);

        form = form.push(submit);

        let content = column![
            Space::with_height(60),
            title,
            subtitle,
            Space::with_height(20),
            tabs,
            Space::with_height(10),
            form,
        ]
        .spacing(8)
        .align_x(Alignment::Center)
        .width(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
