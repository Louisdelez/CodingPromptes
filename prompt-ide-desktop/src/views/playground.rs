use iced::widget::{button, column, container, pick_list, row, scrollable, text, slider, Space};
use iced::{Element, Length};

use crate::models::config::{available_models, ModelDef};
use crate::services::i18n::I18n;

#[derive(Debug, Clone)]
pub enum PlaygroundMessage {
    SelectModel(String),
    TemperatureChanged(f32),
    MaxTokensChanged(f32),
    Execute,
}

pub struct PlaygroundResult {
    pub model: String,
    pub response: String,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub latency_ms: i64,
    pub error: Option<String>,
}

pub fn view_playground<'a>(
    selected_model: &'a str,
    temperature: f32,
    max_tokens: f32,
    results: &'a [PlaygroundResult],
    executing: bool,
    i18n: &'a I18n,
) -> Element<'a, PlaygroundMessage> {
    let models = available_models();
    let model_names: Vec<String> = models.iter().map(|m| m.id.to_string()).collect();

    let model_picker = pick_list(
        model_names,
        Some(selected_model.to_string()),
        PlaygroundMessage::SelectModel,
    )
    .placeholder(i18n.t("playground.select_model"));

    let temp_slider = column![
        text(format!("{}: {:.1}", i18n.t("playground.temperature"), temperature)).size(11),
        slider(0.0..=2.0, temperature, PlaygroundMessage::TemperatureChanged).step(0.1),
    ].spacing(2);

    let tokens_slider = column![
        text(format!("{}: {}", i18n.t("playground.max_tokens"), max_tokens as u32)).size(11),
        slider(256.0..=8192.0, max_tokens, PlaygroundMessage::MaxTokensChanged).step(256.0),
    ].spacing(2);

    let exec_btn = if executing {
        button(text(i18n.t("playground.executing")).size(13))
    } else {
        button(text(i18n.t("playground.execute")).size(13))
            .on_press(PlaygroundMessage::Execute)
            .style(button::primary)
    };

    let mut results_col = column![].spacing(8);
    for r in results {
        let header = row![
            text(&r.model).size(12).color(iced::Color::from_rgb(0.39, 0.4, 0.95)),
            Space::with_width(Length::Fill),
            text(format!("{}ms", r.latency_ms)).size(10).color(iced::Color::from_rgb(0.5, 0.5, 0.55)),
            text(format!("{} tok", r.tokens_in + r.tokens_out)).size(10).color(iced::Color::from_rgb(0.5, 0.5, 0.55)),
        ].spacing(8).align_y(iced::Alignment::Center);

        let body = if let Some(ref err) = r.error {
            text(err.as_str()).size(12).color(iced::Color::from_rgb(0.97, 0.26, 0.26))
        } else {
            text(&r.response).size(12)
        };

        results_col = results_col.push(
            container(column![header, body].spacing(4))
                .padding(8)
                .width(Length::Fill)
                .style(|_theme: &iced::Theme| container::Style {
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.2, 0.2, 0.25),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }),
        );
    }

    let content = column![
        model_picker,
        temp_slider,
        tokens_slider,
        exec_btn,
        scrollable(results_col).height(Length::Fill),
    ]
    .spacing(10)
    .padding(12)
    .width(Length::Fill);

    container(content).width(Length::Fill).height(Length::Fill).into()
}
