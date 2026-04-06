use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length};

use crate::services::i18n::I18n;

#[derive(Debug, Clone)]
pub enum PreviewMessage {
    Copy,
}

pub fn view_preview<'a>(compiled: &'a str, i18n: &'a I18n) -> Element<'a, PreviewMessage> {
    let header = row![
        text(i18n.t("preview.title")).size(14),
        Space::with_width(Length::Fill),
        button(text(i18n.t("preview.copy")).size(11))
            .on_press(PreviewMessage::Copy)
            .style(button::secondary),
    ]
    .align_y(iced::Alignment::Center);

    let body = if compiled.is_empty() {
        text(i18n.t("preview.empty"))
            .size(12)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.55))
    } else {
        text(compiled).size(13)
    };

    let content = column![header, scrollable(body).height(Length::Fill)]
        .spacing(8)
        .padding(12)
        .width(Length::Fill);

    container(content).width(Length::Fill).height(Length::Fill).into()
}
