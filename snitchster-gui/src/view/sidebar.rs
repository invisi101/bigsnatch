use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length};

use crate::message::Message;
use crate::model::process::ProcessSummary;
use crate::theme::colors;

pub fn view<'a>(
    processes: &'a std::collections::BTreeMap<String, ProcessSummary>,
    selected: &Option<String>,
) -> Element<'a, Message> {
    let header = text("Processes").size(18).color(colors::NEON_CYAN);

    let is_all_selected = selected.is_none();
    let all_btn = button(
        text("All").size(15).color(if is_all_selected { colors::NEON_CYAN } else { colors::TEXT_PRIMARY }),
    )
    .width(Length::Fill)
    .on_press(Message::ProcessSelected(None))
    .style(move |_theme, status| {
        let bg = if is_all_selected {
            colors::BG_SELECTED
        } else {
            match status {
                button::Status::Hovered => colors::BG_HOVER,
                _ => iced::Color::TRANSPARENT,
            }
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            text_color: if is_all_selected { colors::NEON_CYAN } else { colors::TEXT_PRIMARY },
            ..Default::default()
        }
    });

    let mut items = column![header, all_btn].spacing(2).width(Length::Fill);

    // Sort by connection count descending
    let mut sorted: Vec<_> = processes.iter().collect();
    sorted.sort_by(|a, b| b.1.connection_count.cmp(&a.1.connection_count));

    for (name, summary) in sorted {
        let is_selected = selected.as_ref() == Some(name);
        let label_text = format!("{} ({})", name, summary.destination_count);
        let name_clone = name.clone();

        let btn = button(
            text(label_text)
                .size(14)
                .color(if is_selected { colors::NEON_PINK } else { colors::TEXT_PRIMARY }),
        )
        .width(Length::Fill)
        .on_press(Message::ProcessSelected(Some(name_clone)))
        .style(move |_theme, status| {
            let bg = if is_selected {
                colors::BG_SELECTED
            } else {
                match status {
                    button::Status::Hovered => colors::BG_HOVER,
                    _ => iced::Color::TRANSPARENT,
                }
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: if is_selected { colors::NEON_PINK } else { colors::TEXT_PRIMARY },
                ..Default::default()
            }
        });

        items = items.push(btn);
    }

    container(scrollable(items).height(Length::Fill))
        .width(180)
        .padding(8)
        .into()
}
