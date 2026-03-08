use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::message::Message;
use crate::model::connection::ConnectionDisplay;
use crate::theme::colors;

pub fn view<'a>(connection: Option<&'a ConnectionDisplay>) -> Element<'a, Message> {
    match connection {
        Some(conn) => {
            let dest_display = if conn.domain.is_empty() {
                format!("{}:{}", conn.dst_addr, conn.dst_port)
            } else {
                format!("{} ({}):{}",  conn.domain, conn.dst_addr, conn.dst_port)
            };

            let header = text(format!(
                "{} (PID {}) -> {} [{}]",
                conn.process_name, conn.pid, dest_display, conn.protocol
            ))
            .size(13)
            .color(colors::TEXT_ACCENT);

            let user_info = format!("{} (uid:{})", conn.username, conn.uid);
            let source_info = format!("{}:{}", conn.src_addr, conn.src_port);

            let details = row![
                column![
                    detail_line("Path", conn.exe_path.clone()),
                    detail_line("Command", conn.cmdline.clone()),
                ]
                .spacing(2),
                iced::widget::Space::with_width(30),
                column![
                    detail_line("User", user_info),
                    detail_line("Source", source_info),
                ]
                .spacing(2),
            ]
            .spacing(5);

            container(column![header, details].spacing(4))
                .padding(8)
                .width(Length::Fill)
                .into()
        }
        None => {
            container(
                text("Select a connection to see details")
                    .size(12)
                    .color(colors::TEXT_SECONDARY),
            )
            .padding(8)
            .width(Length::Fill)
            .center_y(35)
            .into()
        }
    }
}

fn detail_line<'a>(label: &'a str, value: String) -> Element<'a, Message> {
    row![
        text(format!("{}: ", label))
            .size(11)
            .color(colors::TEXT_SECONDARY),
        text(value).size(11),
    ]
    .into()
}
