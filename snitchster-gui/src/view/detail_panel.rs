use iced::widget::{button, column, container, row, scrollable, text, text::Wrapping};
use iced::{Element, Length};

use crate::message::Message;
use crate::model::connection::ConnectionDisplay;
use crate::model::destination::DestinationSummary;
use crate::theme::colors;

pub fn view<'a>(
    selected_dest: Option<&'a DestinationSummary>,
    selected_conn: Option<&'a ConnectionDisplay>,
    drill_down_active: bool,
) -> Element<'a, Message> {
    if drill_down_active {
        // Show raw connection detail
        match selected_conn {
            Some(conn) => connection_detail(conn),
            None => placeholder("Select a connection to see details"),
        }
    } else {
        // Show destination summary
        match selected_dest {
            Some(dest) => destination_detail(dest),
            None => placeholder("Click a destination to drill down into connections"),
        }
    }
}

fn destination_detail(dest: &DestinationSummary) -> Element<'_, Message> {
    let header = text(format!(
        "{} -> {} ({} connections)",
        dest.process_name, dest.display_dest, dest.connection_count
    ))
    .size(17)
    .wrapping(Wrapping::WordOrGlyph)
    .color(colors::NEON_PINK);

    let ports_str = dest.ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ");
    let proto_str = dest.protocols.iter().cloned().collect::<Vec<_>>().join(", ");

    let ip_line = if !dest.domain.is_empty() && dest.domain != dest.dst_addr {
        format!("{} ({})", dest.domain, dest.dst_addr)
    } else {
        dest.dst_addr.clone()
    };

    let details = row![
        column![
            detail_line("Address", ip_line),
            detail_line("Ports", ports_str),
            detail_line("Protocols", proto_str),
        ]
        .spacing(2)
        .width(Length::FillPortion(3)),
        column![
            detail_line("Connections", dest.connection_count.to_string()),
            detail_line("Zone", format!("{:?}", dest.zone)),
        ]
        .spacing(2)
        .width(Length::FillPortion(1)),
    ]
    .spacing(15);

    let content = column![header, details].spacing(4);

    container(scrollable(content).height(Length::Fill))
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn connection_detail(conn: &ConnectionDisplay) -> Element<'_, Message> {
    let dest_display = if conn.domain.is_empty() {
        format!("{}:{}", conn.dst_addr, conn.dst_port)
    } else {
        format!("{} ({}):{}",  conn.domain, conn.dst_addr, conn.dst_port)
    };

    let header = text(format!(
        "{} (PID {}) -> {} [{}]",
        conn.process_name, conn.pid, dest_display, conn.protocol
    ))
    .size(17)
    .wrapping(Wrapping::WordOrGlyph)
    .color(colors::TEXT_ACCENT);

    let user_info = format!("{} (uid:{})", conn.username, conn.uid);
    let source_info = format!("{}:{}", conn.src_addr, conn.src_port);

    let details = row![
        column![
            detail_line("Path", conn.exe_path.clone()),
            detail_line("Command", conn.cmdline.clone()),
        ]
        .spacing(2)
        .width(Length::FillPortion(3)),
        column![
            detail_line("User", user_info),
            detail_line("Source", source_info),
        ]
        .spacing(2)
        .width(Length::FillPortion(1)),
    ]
    .spacing(15);

    let content = column![header, details].spacing(4);

    container(scrollable(content).height(Length::Fill))
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn placeholder(msg: &str) -> Element<'_, Message> {
    container(
        text(msg)
            .size(15)
            .color(colors::TEXT_SECONDARY),
    )
    .padding(8)
    .width(Length::Fill)
    .center_y(35)
    .into()
}

fn detail_line<'a>(label: &'a str, value: String) -> Element<'a, Message> {
    let value_copy = value.clone();
    row![
        text(format!("{}: ", label))
            .size(15)
            .color(colors::TEXT_SECONDARY),
        button(text(value).size(15).wrapping(Wrapping::WordOrGlyph))
            .on_press(Message::CopyToClipboard(value_copy))
            .padding(0)
            .style(|_theme, _status| button::Style {
                background: None,
                text_color: colors::TEXT_PRIMARY,
                ..Default::default()
            }),
    ]
    .into()
}
