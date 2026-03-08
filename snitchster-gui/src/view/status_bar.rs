use iced::widget::{container, row, text, Space};
use iced::{Element, Length};

use crate::message::Message;
use crate::theme::colors;

pub fn view<'a>(
    is_connected: bool,
    total_connections: usize,
    active_processes: usize,
    events_per_second: f64,
) -> Element<'a, Message> {
    let status_indicator = if is_connected {
        text("● Connected").size(14).color(colors::STATUS_CONNECTED)
    } else {
        text("● Disconnected").size(14).color(colors::STATUS_DISCONNECTED)
    };

    container(
        row![
            status_indicator,
            Space::with_width(20),
            text(format!("{} connections", total_connections))
                .size(14)
                .color(colors::TEXT_SECONDARY),
            Space::with_width(20),
            text(format!("{} processes", active_processes))
                .size(14)
                .color(colors::TEXT_SECONDARY),
            Space::with_width(20),
            text(format!("{:.0} events/sec", events_per_second))
                .size(14)
                .color(colors::TEXT_SECONDARY),
            Space::with_width(Length::Fill),
        ]
        .spacing(5)
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 10])
    .width(Length::Fill)
    .into()
}
