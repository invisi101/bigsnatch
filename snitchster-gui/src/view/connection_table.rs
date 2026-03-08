use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length};

use crate::message::{Message, SortColumn};
use crate::model::connection::ConnectionDisplay;
use crate::theme::colors;

pub fn view<'a>(
    connections: &'a [ConnectionDisplay],
    selected_id: Option<u64>,
    sort_column: SortColumn,
) -> Element<'a, Message> {
    // Header row
    let header = container(
        row![
            header_cell("Time", SortColumn::Time, sort_column, 80),
            header_cell("Process", SortColumn::Process, sort_column, 130),
            header_cell("PID", SortColumn::Pid, sort_column, 65),
            header_cell("Destination", SortColumn::Domain, sort_column, 280),
            header_cell("Port", SortColumn::Port, sort_column, 65),
            header_cell("Proto", SortColumn::Protocol, sort_column, 55),
        ]
        .spacing(2),
    )
    .padding([4, 8]);

    // Connection rows
    let mut rows = column![].spacing(0);

    for conn in connections.iter().rev().take(2000) {
        let is_selected = selected_id == Some(conn.id);
        let row_element = connection_row(conn, is_selected);
        rows = rows.push(row_element);
    }

    let scrollable_rows = scrollable(rows).height(Length::Fill);

    column![header, scrollable_rows]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn header_cell<'a>(
    label: &str,
    col: SortColumn,
    current_sort: SortColumn,
    width: u16,
) -> Element<'a, Message> {
    let arrow = if col == current_sort { " v" } else { "" };
    let label_text = text(format!("{}{}", label, arrow))
        .size(12)
        .color(colors::TEXT_SECONDARY);

    button(label_text)
        .on_press(Message::ColumnSortChanged(col))
        .width(width)
        .into()
}

fn connection_row(conn: &ConnectionDisplay, is_selected: bool) -> Element<'_, Message> {
    let proto_color = if conn.protocol == "TCP" {
        colors::TCP_COLOR
    } else {
        colors::UDP_COLOR
    };

    // Highlight DNS traffic
    let dest_color = if conn.dst_port == 53 {
        colors::DNS_COLOR
    } else if !conn.domain.is_empty() {
        colors::TEXT_ACCENT
    } else {
        colors::TEXT_PRIMARY
    };

    let row_content = row![
        text(&conn.time_str).size(12).width(80),
        text(&conn.process_name).size(12).width(130),
        text(conn.pid.to_string()).size(12).width(65),
        text(&conn.display_dest).size(12).width(280).color(dest_color),
        text(conn.dst_port.to_string()).size(12).width(65),
        text(&conn.protocol).size(12).width(55).color(proto_color),
    ]
    .spacing(2);

    let conn_id = conn.id;

    button(row_content)
        .on_press(Message::ConnectionSelected(Some(conn_id)))
        .width(Length::Fill)
        .padding([2, 8])
        .into()
}
