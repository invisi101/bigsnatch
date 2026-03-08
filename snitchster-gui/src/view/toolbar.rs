use iced::widget::{button, container, pick_list, row, text, text_input, Space};
use iced::{Element, Length};

use crate::message::{Message, ProtocolFilter};
use crate::theme::colors;

pub fn view<'a>(
    search_query: &str,
    protocol_filter: ProtocolFilter,
    is_paused: bool,
    _auto_scroll: bool,
    destination_count: usize,
    total_events: usize,
) -> Element<'a, Message> {
    let title = text("Big Snatch")
        .size(46)
        .font(crate::fonts::VEGAN_STYLE)
        .color(colors::NEON_PINK);

    let search = text_input("Search processes, destinations...", search_query)
        .on_input(Message::SearchChanged)
        .width(300);

    let proto_options = vec![ProtocolFilter::All, ProtocolFilter::TcpOnly, ProtocolFilter::UdpOnly];
    let protocol_pick = pick_list(
        proto_options,
        Some(protocol_filter),
        Message::ProtocolFilterChanged,
    )
    .placeholder("Protocol");

    let pause_label = if is_paused { "Resume" } else { "Pause" };
    let pause_btn = button(text(pause_label).size(16))
        .on_press(Message::TogglePause);

    let clear_btn = button(text("Clear").size(16))
        .on_press(Message::ClearConnections);

    let quit_btn = button(
        text("Quit")
            .size(30)
            .font(crate::fonts::VEGAN_STYLE)
            .color(colors::STATUS_DISCONNECTED),
    )
    .on_press(Message::Quit)
    .style(|_theme, status| {
        let bg = match status {
            button::Status::Hovered | button::Status::Pressed => colors::BTN_GREY_HOVER,
            _ => colors::BTN_GREY,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                radius: 10.0.into(),
                color: colors::BORDER,
                width: 1.0,
            },
            text_color: colors::STATUS_DISCONNECTED,
            ..Default::default()
        }
    });

    let stats_text = format!("{} destinations  |  {} events", destination_count, total_events);

    container(
        row![
            title,
            Space::with_width(20),
            search,
            Space::with_width(10),
            protocol_pick,
            Space::with_width(10),
            pause_btn,
            Space::with_width(5),
            clear_btn,
            Space::with_width(Length::Fill),
            text(stats_text).size(15).color(colors::NEON_CYAN),
            Space::with_width(10),
            quit_btn,
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center),
    )
    .padding(10)
    .width(Length::Fill)
    .into()
}

impl std::fmt::Display for ProtocolFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolFilter::All => write!(f, "All"),
            ProtocolFilter::TcpOnly => write!(f, "TCP"),
            ProtocolFilter::UdpOnly => write!(f, "UDP"),
        }
    }
}
