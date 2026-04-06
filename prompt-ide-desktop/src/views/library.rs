use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length};

use crate::models::project::{Workspace, PromptProject};
use crate::services::i18n::I18n;

#[derive(Debug, Clone)]
pub enum LibraryMessage {
    SearchChanged(String),
    SelectProject(String),
    NewProject(Option<String>),
    DeleteProject(String),
    DuplicateProject(String),
    NewWorkspace,
    WorkspaceNameChanged(String),
    CreateWorkspace,
    DeleteWorkspace(String),
}

pub fn view_library<'a>(
    workspaces: &'a [Workspace],
    projects: &'a [PromptProject],
    current_id: &'a str,
    search: &'a str,
    new_ws_name: &'a str,
    i18n: &'a I18n,
) -> Element<'a, LibraryMessage> {
    let search_lower = search.to_lowercase();
    let mut col = column![].spacing(4).width(Length::Fill);

    // Search
    col = col.push(
        text_input(i18n.t("library.search"), search)
            .on_input(LibraryMessage::SearchChanged)
            .size(12),
    );

    // New workspace
    col = col.push(
        row![
            button(text(format!("+ {}", i18n.t("library.new_workspace"))).size(11))
                .on_press(LibraryMessage::NewWorkspace)
                .style(button::secondary),
            button(text(format!("+ {}", i18n.t("library.new_prompt"))).size(11))
                .on_press(LibraryMessage::NewProject(None))
                .style(button::secondary),
        ]
        .spacing(4),
    );

    // New workspace input
    if !new_ws_name.is_empty() || true {
        // Shown when creating
    }

    // Workspaces
    for ws in workspaces {
        if !search.is_empty() && !ws.name.to_lowercase().contains(&search_lower) {
            continue;
        }

        col = col.push(
            row![
                text(format!("📂 {}", ws.name)).size(13),
                Space::with_width(Length::Fill),
                button(text("+").size(11))
                    .on_press(LibraryMessage::NewProject(Some(ws.id.clone())))
                    .style(button::text),
                button(text("✕").size(11))
                    .on_press(LibraryMessage::DeleteWorkspace(ws.id.clone()))
                    .style(button::text),
            ]
            .align_y(iced::Alignment::Center)
            .spacing(4),
        );

        // Projects in workspace
        for p in projects.iter().filter(|p| p.workspace_id.as_deref() == Some(&ws.id)) {
            if !search.is_empty() && !p.name.to_lowercase().contains(&search_lower)
                && !p.blocks.iter().any(|b| b.content.to_lowercase().contains(&search_lower))
            {
                continue;
            }

            let is_active = p.id == current_id;
            col = col.push(
                button(
                    row![
                        text("  📄").size(12),
                        text(&p.name).size(12).color(if is_active {
                            iced::Color::from_rgb(0.39, 0.4, 0.95)
                        } else {
                            iced::Color::from_rgb(0.8, 0.8, 0.82)
                        }),
                    ]
                    .spacing(4),
                )
                .on_press(LibraryMessage::SelectProject(p.id.clone()))
                .style(if is_active { button::primary } else { button::text })
                .width(Length::Fill),
            );
        }
    }

    // Free prompts
    let orphans: Vec<_> = projects.iter().filter(|p| p.workspace_id.is_none()).collect();
    if !orphans.is_empty() {
        col = col.push(text(i18n.t("library.free_prompts")).size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.55)));
        for p in orphans {
            if !search.is_empty() && !p.name.to_lowercase().contains(&search_lower)
                && !p.blocks.iter().any(|b| b.content.to_lowercase().contains(&search_lower))
            {
                continue;
            }
            let is_active = p.id == current_id;
            col = col.push(
                button(
                    text(&p.name).size(12).color(if is_active {
                        iced::Color::from_rgb(0.39, 0.4, 0.95)
                    } else {
                        iced::Color::from_rgb(0.8, 0.8, 0.82)
                    }),
                )
                .on_press(LibraryMessage::SelectProject(p.id.clone()))
                .style(if is_active { button::primary } else { button::text })
                .width(Length::Fill),
            );
        }
    }

    if workspaces.is_empty() && projects.is_empty() {
        col = col.push(
            text(i18n.t("library.empty"))
                .size(12)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.55)),
        );
    }

    container(scrollable(col.padding(8)).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
