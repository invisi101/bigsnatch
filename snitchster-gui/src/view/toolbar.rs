use iced::widget::{button, container, pick_list, row, text, text_input, Space};
use iced::{Element, Length};

use crate::message::{Message, ProtocolFilter};

pub fn view<'a>(
    search_query: &str,
    protocol_filter: ProtocolFilter,
    is_paused: bool,
    auto_scroll: bool,
    total_connections: usize,
) -> Element<'a, Message> {
    let title = text("Snitchster")
        .size(20)
        .font(iced::Font::with_name("monospace"));

    let search = text_input("Search processes, domains, IPs...", search_query)
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
    let pause_btn = button(text(pause_label).size(13))
        .on_press(Message::TogglePause);

    let scroll_label = if auto_scroll { "Auto-scroll: ON" } else { "Auto-scroll: OFF" };
    let scroll_btn = button(text(scroll_label).size(13))
        .on_press(Message::ToggleAutoScroll);

    let clear_btn = button(text("Clear").size(13))
        .on_press(Message::ClearConnections);

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
            scroll_btn,
            Space::with_width(5),
            clear_btn,
            Space::with_width(Length::Fill),
            text(format!("{} connections", total_connections)).size(13),
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
