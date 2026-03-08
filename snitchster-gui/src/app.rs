use std::collections::BTreeMap;
use std::time::{Instant, SystemTime};

use iced::widget::{column, container, horizontal_rule, row, vertical_rule};
use iced::{Element, Length, Subscription};

use crate::message::{Message, ProtocolFilter, SortColumn};
use crate::model::connection::ConnectionDisplay;
use crate::model::process::ProcessSummary;
use crate::subscription as daemon_sub;
use crate::view;

const MAX_CONNECTIONS: usize = 10_000;

pub struct App {
    // Data
    connections: Vec<ConnectionDisplay>,
    /// Pre-filtered view of connections, rebuilt on filter changes
    filtered: Vec<ConnectionDisplay>,
    processes: BTreeMap<String, ProcessSummary>,
    selected_connection_id: Option<u64>,
    selected_process: Option<String>,

    // UI state
    search_query: String,
    sort_column: SortColumn,
    protocol_filter: ProtocolFilter,
    is_paused: bool,
    auto_scroll: bool,
    is_connected: bool,

    // Stats
    total_events: usize,
    events_window: Vec<Instant>,
    events_per_second: f64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            connections: Vec::with_capacity(MAX_CONNECTIONS),
            filtered: Vec::new(),
            processes: BTreeMap::new(),
            selected_connection_id: None,
            selected_process: None,
            search_query: String::new(),
            sort_column: SortColumn::Time,
            protocol_filter: ProtocolFilter::All,
            is_paused: false,
            auto_scroll: true,
            is_connected: false,
            total_events: 0,
            events_window: Vec::new(),
            events_per_second: 0.0,
        }
    }
}

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ConnectionReceived(event) => {
                if self.is_paused {
                    return;
                }

                let conn = ConnectionDisplay::from(event);

                // Update process summary
                let entry = self.processes
                    .entry(conn.process_name.clone())
                    .or_insert_with(|| ProcessSummary {
                        name: conn.process_name.clone(),
                        connection_count: 0,
                        last_seen: SystemTime::now(),
                    });
                entry.connection_count += 1;
                entry.last_seen = SystemTime::now();

                self.connections.push(conn);

                // Enforce max capacity
                if self.connections.len() > MAX_CONNECTIONS {
                    let drain_count = MAX_CONNECTIONS / 10;
                    self.connections.drain(..drain_count);
                }

                // Update stats
                self.total_events += 1;
                let now = Instant::now();
                self.events_window.push(now);
                self.events_window
                    .retain(|t| now.duration_since(*t).as_secs() < 5);
                self.events_per_second = self.events_window.len() as f64 / 5.0;

                self.rebuild_filter();
            }

            Message::DaemonConnected => {
                self.is_connected = true;
            }

            Message::DaemonDisconnected(_) => {
                self.is_connected = false;
            }

            Message::DaemonError(e) => {
                self.is_connected = false;
                tracing::error!("Daemon error: {}", e);
            }

            Message::SearchChanged(query) => {
                self.search_query = query;
                self.rebuild_filter();
            }

            Message::ProcessSelected(process) => {
                self.selected_process = process;
                self.rebuild_filter();
            }

            Message::ConnectionSelected(id) => {
                self.selected_connection_id = id;
            }

            Message::ColumnSortChanged(col) => {
                self.sort_column = col;
                self.rebuild_filter();
            }

            Message::TogglePause => {
                self.is_paused = !self.is_paused;
            }

            Message::ToggleAutoScroll => {
                self.auto_scroll = !self.auto_scroll;
            }

            Message::ProtocolFilterChanged(filter) => {
                self.protocol_filter = filter;
                self.rebuild_filter();
            }

            Message::ClearConnections => {
                self.connections.clear();
                self.filtered.clear();
                self.processes.clear();
                self.selected_connection_id = None;
                self.total_events = 0;
                self.events_per_second = 0.0;
            }

            Message::Tick => {
                let now = Instant::now();
                self.events_window
                    .retain(|t| now.duration_since(*t).as_secs() < 5);
                self.events_per_second = self.events_window.len() as f64 / 5.0;
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let selected_conn = self
            .selected_connection_id
            .and_then(|id| self.connections.iter().find(|c| c.id == id));

        let toolbar = view::toolbar::view(
            &self.search_query,
            self.protocol_filter,
            self.is_paused,
            self.auto_scroll,
            self.filtered.len(),
        );

        let sidebar = view::sidebar::view(&self.processes, &self.selected_process);

        let table = view::connection_table::view(
            &self.filtered,
            self.selected_connection_id,
            self.sort_column,
        );

        let detail = view::detail_panel::view(selected_conn);

        let status = view::status_bar::view(
            self.is_connected,
            self.total_events,
            self.processes.len(),
            self.events_per_second,
        );

        let main_content = row![
            sidebar,
            vertical_rule(1),
            column![table, horizontal_rule(1), detail,]
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .height(Length::Fill);

        container(
            column![
                toolbar,
                horizontal_rule(1),
                main_content,
                horizontal_rule(1),
                status,
            ]
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        daemon_sub::daemon_events()
    }

    fn rebuild_filter(&mut self) {
        let query = self.search_query.to_lowercase();

        self.filtered = self
            .connections
            .iter()
            .filter(|conn| {
                match self.protocol_filter {
                    ProtocolFilter::All => {}
                    ProtocolFilter::TcpOnly => {
                        if conn.protocol != "TCP" {
                            return false;
                        }
                    }
                    ProtocolFilter::UdpOnly => {
                        if conn.protocol != "UDP" {
                            return false;
                        }
                    }
                }

                if let Some(ref proc_name) = self.selected_process {
                    if &conn.process_name != proc_name {
                        return false;
                    }
                }

                if !query.is_empty() {
                    let matches = conn.process_name.to_lowercase().contains(&query)
                        || conn.dst_addr.to_lowercase().contains(&query)
                        || conn.domain.to_lowercase().contains(&query)
                        || conn.dst_port.to_string().contains(&query)
                        || conn.exe_path.to_lowercase().contains(&query);
                    if !matches {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort
        match self.sort_column {
            SortColumn::Time => {}
            SortColumn::Process => {
                self.filtered.sort_by(|a, b| a.process_name.cmp(&b.process_name));
            }
            SortColumn::Pid => {
                self.filtered.sort_by(|a, b| a.pid.cmp(&b.pid));
            }
            SortColumn::Domain => {
                self.filtered.sort_by(|a, b| a.display_dest.cmp(&b.display_dest));
            }
            SortColumn::Port => {
                self.filtered.sort_by(|a, b| a.dst_port.cmp(&b.dst_port));
            }
            SortColumn::Protocol => {
                self.filtered.sort_by(|a, b| a.protocol.cmp(&b.protocol));
            }
        }
    }
}
