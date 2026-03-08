use iced::widget::{button, column, container, scrollable, text, Space};
use iced::{Element, Length};

use crate::message::Message;
use crate::model::process::ProcessSummary;

pub fn view<'a>(
    processes: &'a std::collections::BTreeMap<String, ProcessSummary>,
    selected: &Option<String>,
) -> Element<'a, Message> {
    let header = text("Processes").size(14);

    let all_btn = {
        let label = text("All").size(13);
        let btn = button(label).width(Length::Fill);
        if selected.is_none() {
            btn.on_press(Message::ProcessSelected(None))
        } else {
            btn.on_press(Message::ProcessSelected(None))
        }
    };

    let mut items = column![header, all_btn].spacing(2).width(Length::Fill);

    // Sort by connection count descending
    let mut sorted: Vec<_> = processes.iter().collect();
    sorted.sort_by(|a, b| b.1.connection_count.cmp(&a.1.connection_count));

    for (name, summary) in sorted {
        let label = text(format!("{} ({})", name, summary.connection_count)).size(12);
        let btn = button(label)
            .width(Length::Fill)
            .on_press(Message::ProcessSelected(Some(name.clone())));
        items = items.push(btn);
    }

    container(scrollable(items).height(Length::Fill))
        .width(180)
        .padding(8)
        .into()
}
