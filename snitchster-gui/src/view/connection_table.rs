use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length};

use crate::message::{DestSortColumn, Message};
use crate::model::connection::ConnectionDisplay;
use crate::model::destination::{DestinationSummary, TrafficZone};
use crate::theme::colors;

// Column proportions for destination view
// With process:    Process(3) Destination(6) Count(1) LastSeen(2) Ports(2) Proto(1) = 15 parts
// Without process: Destination(8) Count(1) LastSeen(2) Ports(2) Proto(1) = 14 parts
const P_PROCESS: u16 = 3;
const P_DEST: u16 = 6;
const P_DEST_WIDE: u16 = 8; // when no process column
const P_COUNT: u16 = 1;
const P_LASTSEEN: u16 = 2;
const P_PORTS: u16 = 2;
const P_PROTO: u16 = 1;

// Drill-down column proportions
const D_TIME: u16 = 2;
const D_PROCESS: u16 = 4;
const D_PID: u16 = 1;
const D_DEST: u16 = 6;
const D_PORT: u16 = 1;
const D_PROTO: u16 = 1;

pub fn view<'a>(
    destinations: &'a [DestinationSummary],
    selected_dest: &Option<String>,
    sort_column: DestSortColumn,
    zone: TrafficZone,
    drill_down_active: bool,
    drill_down_connections: &'a [ConnectionDisplay],
    selected_connection_id: Option<u64>,
    show_process_column: bool,
) -> Element<'a, Message> {
    if drill_down_active {
        drill_down_view(drill_down_connections, selected_connection_id)
    } else {
        destination_view(destinations, selected_dest, sort_column, zone, show_process_column)
    }
}

fn destination_view<'a>(
    destinations: &'a [DestinationSummary],
    selected_dest: &Option<String>,
    sort_column: DestSortColumn,
    zone: TrafficZone,
    show_process: bool,
) -> Element<'a, Message> {
    let ext_btn = zone_tab("External", TrafficZone::External, zone);
    let int_btn = zone_tab("Internal", TrafficZone::Internal, zone);

    let zone_tabs = row![ext_btn, Space::with_width(4), int_btn]
        .spacing(0)
        .padding([6, 8]);

    let dest_portion = if show_process { P_DEST } else { P_DEST_WIDE };

    // Header row
    let mut header_row = row![].spacing(2);
    if show_process {
        header_row = header_row.push(
            text("Process").size(15).color(colors::TEXT_SECONDARY)
                .width(Length::FillPortion(P_PROCESS)),
        );
    }
    header_row = header_row
        .push(dest_header_cell("Destination", DestSortColumn::Destination, sort_column, dest_portion))
        .push(dest_header_cell("Count", DestSortColumn::Count, sort_column, P_COUNT))
        .push(dest_header_cell("Last Seen", DestSortColumn::LastSeen, sort_column, P_LASTSEEN))
        .push(dest_header_cell("Ports", DestSortColumn::Ports, sort_column, P_PORTS))
        .push(text("Protocols").size(15).color(colors::TEXT_SECONDARY)
            .width(Length::FillPortion(P_PROTO)));

    let header = container(header_row).padding([4, 8]);

    let mut rows = column![].spacing(0);
    for dest in destinations {
        let is_selected = selected_dest.as_ref() == Some(&dest.dst_addr);
        rows = rows.push(destination_row(dest, is_selected, show_process, dest_portion));
    }

    let scrollable_rows = scrollable(rows).height(Length::Fill);

    column![zone_tabs, header, scrollable_rows]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn zone_tab(label: &str, tab_zone: TrafficZone, current: TrafficZone) -> Element<'_, Message> {
    let is_active = tab_zone == current;
    button(
        text(label)
            .size(15)
            .color(if is_active { colors::NEON_CYAN } else { colors::TEXT_SECONDARY }),
    )
    .on_press(Message::TrafficZoneChanged(tab_zone))
    .style(move |_theme, status| {
        let bg = if is_active {
            colors::BG_SELECTED
        } else {
            match status {
                button::Status::Hovered => colors::BG_HOVER,
                _ => colors::BG_SECONDARY,
            }
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                radius: 6.0.into(),
                color: if is_active { colors::NEON_CYAN } else { colors::BORDER },
                width: 1.0,
            },
            text_color: if is_active { colors::NEON_CYAN } else { colors::TEXT_SECONDARY },
            ..Default::default()
        }
    })
    .into()
}

fn dest_header_cell<'a>(
    label: &str,
    col: DestSortColumn,
    current_sort: DestSortColumn,
    portion: u16,
) -> Element<'a, Message> {
    let arrow = if col == current_sort { " v" } else { "" };
    let label_text = text(format!("{}{}", label, arrow))
        .size(15)
        .color(colors::TEXT_SECONDARY);

    button(label_text)
        .on_press(Message::DestSortChanged(col))
        .width(Length::FillPortion(portion))
        .into()
}

fn destination_row(dest: &DestinationSummary, is_selected: bool, show_process: bool, dest_portion: u16) -> Element<'_, Message> {
    let dest_color = if !dest.domain.is_empty() {
        colors::TEXT_ACCENT
    } else {
        colors::TEXT_PRIMARY
    };

    let ports_str = {
        let ports: Vec<_> = dest.ports.iter().take(4).collect();
        if ports.len() <= 3 {
            ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")
        } else {
            let shown: Vec<_> = ports[..3].iter().map(|p| p.to_string()).collect();
            format!("{} +{}", shown.join(", "), dest.ports.len() - 3)
        }
    };

    let proto_str: String = dest.protocols.iter().cloned().collect::<Vec<_>>().join("/");
    let proto_color = if dest.protocols.len() > 1 {
        colors::TEXT_ACCENT
    } else if dest.protocols.contains("TCP") {
        colors::TCP_COLOR
    } else {
        colors::UDP_COLOR
    };

    let last_seen_str = {
        let elapsed = dest.last_seen.elapsed().unwrap_or_default();
        if elapsed.as_secs() < 5 {
            "just now".to_string()
        } else if elapsed.as_secs() < 60 {
            format!("{}s ago", elapsed.as_secs())
        } else if elapsed.as_secs() < 3600 {
            format!("{}m ago", elapsed.as_secs() / 60)
        } else {
            format!("{}h ago", elapsed.as_secs() / 3600)
        }
    };

    let mut row_content = row![].spacing(2);
    if show_process {
        row_content = row_content.push(
            text(&dest.process_name).size(15)
                .width(Length::FillPortion(P_PROCESS))
                .color(colors::NEON_PINK),
        );
    }
    row_content = row_content
        .push(text(&dest.display_dest).size(15)
            .width(Length::FillPortion(dest_portion)).color(dest_color))
        .push(text(dest.connection_count.to_string()).size(15)
            .width(Length::FillPortion(P_COUNT)).color(colors::NEON_CYAN))
        .push(text(last_seen_str).size(15)
            .width(Length::FillPortion(P_LASTSEEN)).color(colors::TEXT_SECONDARY))
        .push(text(ports_str).size(15)
            .width(Length::FillPortion(P_PORTS)).color(colors::TEXT_SECONDARY))
        .push(text(proto_str).size(15)
            .width(Length::FillPortion(P_PROTO)).color(proto_color));

    let dst_addr = dest.dst_addr.clone();

    button(row_content)
        .on_press(Message::DrillDown(dst_addr))
        .width(Length::Fill)
        .padding([3, 8])
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
                    radius: 2.0.into(),
                    ..Default::default()
                },
                text_color: colors::TEXT_PRIMARY,
                ..Default::default()
            }
        })
        .into()
}

// --- Drill-down: raw connection events for one destination ---

fn drill_down_view<'a>(
    connections: &'a [ConnectionDisplay],
    selected_id: Option<u64>,
) -> Element<'a, Message> {
    let back_btn = button(
        text("<  Back to destinations").size(15).color(colors::NEON_CYAN),
    )
    .on_press(Message::DrillDownBack)
    .style(|_theme, status| {
        let bg = match status {
            button::Status::Hovered => colors::BG_HOVER,
            _ => iced::Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            text_color: colors::NEON_CYAN,
            ..Default::default()
        }
    });

    let header = container(
        row![
            text("Time").size(15).color(colors::TEXT_SECONDARY)
                .width(Length::FillPortion(D_TIME)),
            text("Process").size(15).color(colors::TEXT_SECONDARY)
                .width(Length::FillPortion(D_PROCESS)),
            text("PID").size(15).color(colors::TEXT_SECONDARY)
                .width(Length::FillPortion(D_PID)),
            text("Destination").size(15).color(colors::TEXT_SECONDARY)
                .width(Length::FillPortion(D_DEST)),
            text("Port").size(15).color(colors::TEXT_SECONDARY)
                .width(Length::FillPortion(D_PORT)),
            text("Proto").size(15).color(colors::TEXT_SECONDARY)
                .width(Length::FillPortion(D_PROTO)),
        ]
        .spacing(2),
    )
    .padding([4, 8]);

    let mut rows = column![].spacing(0);
    for conn in connections.iter().rev().take(2000) {
        let is_selected = selected_id == Some(conn.id);
        rows = rows.push(connection_row(conn, is_selected));
    }

    let scrollable_rows = scrollable(rows).height(Length::Fill);

    column![
        container(back_btn).padding([6, 8]),
        header,
        scrollable_rows,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn connection_row(conn: &ConnectionDisplay, is_selected: bool) -> Element<'_, Message> {
    let proto_color = if conn.protocol == "TCP" {
        colors::TCP_COLOR
    } else {
        colors::UDP_COLOR
    };

    let dest_color = if conn.dst_port == 53 {
        colors::DNS_COLOR
    } else if !conn.domain.is_empty() {
        colors::TEXT_ACCENT
    } else {
        colors::TEXT_PRIMARY
    };

    let row_content = row![
        text(&conn.time_str).size(15)
            .width(Length::FillPortion(D_TIME)),
        text(&conn.process_name).size(15)
            .width(Length::FillPortion(D_PROCESS)),
        text(conn.pid.to_string()).size(15)
            .width(Length::FillPortion(D_PID)),
        text(&conn.display_dest).size(15)
            .width(Length::FillPortion(D_DEST)).color(dest_color),
        text(conn.dst_port.to_string()).size(15)
            .width(Length::FillPortion(D_PORT)),
        text(&conn.protocol).size(15)
            .width(Length::FillPortion(D_PROTO)).color(proto_color),
    ]
    .spacing(2);

    let conn_id = conn.id;

    button(row_content)
        .on_press(Message::ConnectionSelected(Some(conn_id)))
        .width(Length::Fill)
        .padding([2, 8])
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
                    radius: 2.0.into(),
                    ..Default::default()
                },
                text_color: colors::TEXT_PRIMARY,
                ..Default::default()
            }
        })
        .into()
}
