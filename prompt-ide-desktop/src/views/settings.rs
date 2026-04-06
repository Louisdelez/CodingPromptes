use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler, Space};
use iced::{Element, Length};

use crate::models::config::{AppConfig, ThemeMode};
use crate::services::i18n::I18n;

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    OpenAiKeyChanged(String),
    AnthropicKeyChanged(String),
    GoogleKeyChanged(String),
    GroqKeyChanged(String),
    LocalServerUrlChanged(String),
    ToggleTheme(bool),
    ToggleLang(bool),
    Logout,
}

pub fn view_settings<'a>(config: &'a AppConfig, user_name: &'a str, i18n: &'a I18n) -> Element<'a, SettingsMessage> {
    let mut col = column![].spacing(14).padding(12).width(Length::Fill);

    col = col.push(text(i18n.t("settings.title")).size(16));

    // User
    col = col.push(
        row![
            text(format!("👤 {user_name}")).size(13),
            Space::with_width(Length::Fill),
            button(text(i18n.t("auth.logout")).size(11))
                .on_press(SettingsMessage::Logout)
                .style(button::danger),
        ]
        .align_y(iced::Alignment::Center),
    );

    // Theme
    let is_dark = config.theme == ThemeMode::Dark;
    col = col.push(
        row![
            text(i18n.t("settings.theme")).size(13),
            Space::with_width(Length::Fill),
            toggler(is_dark).label("Dark").on_toggle(SettingsMessage::ToggleTheme),
        ]
        .align_y(iced::Alignment::Center),
    );

    // Language
    let is_en = config.lang == "en";
    col = col.push(
        row![
            text(i18n.t("settings.language")).size(13),
            Space::with_width(Length::Fill),
            toggler(is_en).label("EN").on_toggle(SettingsMessage::ToggleLang),
        ]
        .align_y(iced::Alignment::Center),
    );

    // API Keys
    col = col.push(text(i18n.t("settings.api_keys")).size(14));

    col = col.push(column![
        text("OpenAI").size(11),
        text_input("sk-...", &config.openai_key)
            .on_input(SettingsMessage::OpenAiKeyChanged)
            .size(12)
            .secure(true),
    ].spacing(2));

    col = col.push(column![
        text("Anthropic").size(11),
        text_input("sk-ant-...", &config.anthropic_key)
            .on_input(SettingsMessage::AnthropicKeyChanged)
            .size(12)
            .secure(true),
    ].spacing(2));

    col = col.push(column![
        text("Google").size(11),
        text_input("AI...", &config.google_key)
            .on_input(SettingsMessage::GoogleKeyChanged)
            .size(12)
            .secure(true),
    ].spacing(2));

    col = col.push(column![
        text("Groq").size(11),
        text_input("gsk_...", &config.groq_key)
            .on_input(SettingsMessage::GroqKeyChanged)
            .size(12)
            .secure(true),
    ].spacing(2));

    // Local server
    col = col.push(text(i18n.t("settings.local_server")).size(14));
    col = col.push(
        text_input("http://localhost:8910", &config.local_server_url)
            .on_input(SettingsMessage::LocalServerUrlChanged)
            .size(12),
    );

    container(scrollable(col).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
