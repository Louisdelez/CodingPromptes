use iced::widget::{button, column, container, row, scrollable, text, text_editor, Space};
use iced::{Element, Length};

use crate::models::block::{BlockType, PromptBlock};
use crate::services::i18n::I18n;

#[derive(Debug, Clone)]
pub enum EditorMessage {
    BlockContentChanged(usize, text_editor::Action),
    ToggleBlock(usize),
    RemoveBlock(usize),
    AddBlock(BlockType),
    MoveBlockUp(usize),
    MoveBlockDown(usize),
}

pub fn view_blocks<'a>(
    blocks: &'a [PromptBlock],
    editor_contents: &'a [text_editor::Content],
    i18n: &'a I18n,
) -> Element<'a, EditorMessage> {
    let lang = i18n.lang();
    let mut col = column![].spacing(8).width(Length::Fill);

    for (i, block) in blocks.iter().enumerate() {
        let bt = block.block_type;
        let color = bt.color();
        let label = bt.label(lang);

        let header = row![
            text("≡").size(14).color(iced::Color::from_rgb(0.5, 0.5, 0.55)),
            text("●").size(10).color(color),
            text(label).size(13).color(color),
            Space::with_width(Length::Fill),
            button(text(if block.enabled { "◉" } else { "○" }).size(12))
                .on_press(EditorMessage::ToggleBlock(i))
                .style(button::text),
            button(text("▲").size(10))
                .on_press(EditorMessage::MoveBlockUp(i))
                .style(button::text),
            button(text("▼").size(10))
                .on_press(EditorMessage::MoveBlockDown(i))
                .style(button::text),
            button(text("✕").size(12))
                .on_press(EditorMessage::RemoveBlock(i))
                .style(button::text),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        let Some(content) = editor_contents.get(i) else { continue };
        let editor = text_editor(content)
            .on_action(move |action| EditorMessage::BlockContentChanged(i, action))
            .height(Length::Shrink)
            .size(13);

        let opacity = if block.enabled { 1.0 } else { 0.4 };

        let block_widget = container(
            column![header, editor].spacing(4),
        )
        .padding(8)
        .width(Length::Fill)
        .style(move |_theme: &iced::Theme| container::Style {
            border: iced::Border {
                color: color.scale_alpha(0.3),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        });

        col = col.push(block_widget);
    }

    // Add block buttons
    let mut add_row = row![].spacing(4);
    for bt in BlockType::all() {
        let label = bt.label(lang);
        let short = label.split('/').next().unwrap_or(label).trim();
        add_row = add_row.push(
            button(text(format!("+ {short}")).size(11))
                .on_press(EditorMessage::AddBlock(*bt))
                .style(button::secondary),
        );
    }
    col = col.push(add_row);

    scrollable(col.padding(12)).height(Length::Fill).into()
}
