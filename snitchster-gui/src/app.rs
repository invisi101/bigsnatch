use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::{Instant, SystemTime};

use iced::widget::{column, container, horizontal_rule, row, vertical_rule};
use iced::{Element, Length, Subscription, Task};

use crate::message::{DestSortColumn, Message, ProtocolFilter};
use crate::model::connection::ConnectionDisplay;
use crate::model::destination::{classify_zone, DestinationSummary, TrafficZone};
use crate::model::process::ProcessSummary;
use crate::subscription as daemon_sub;
use crate::view;

const MAX_CONNECTIONS: usize = 10_000;

pub struct App {
    // Raw event storage
    connections: Vec<ConnectionDisplay>,

    // Process sidebar
    processes: BTreeMap<String, ProcessSummary>,
    selected_process: Option<String>,

    // Aggregated destinations: key = (process_name, display_dest)
    destinations: HashMap<(String, String), DestinationSummary>,
    filtered_destinations: Vec<DestinationSummary>,

    // Navigation
    traffic_zone: TrafficZone,
    selected_destination: Option<String>,
    drill_down_active: bool,
    drill_down_connections: Vec<ConnectionDisplay>,
    selected_connection_id: Option<u64>,

    // UI state
    search_query: String,
    dest_sort_column: DestSortColumn,
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
            processes: BTreeMap::new(),
            selected_process: None,
            destinations: HashMap::new(),
            filtered_destinations: Vec::new(),
            traffic_zone: TrafficZone::External,
            selected_destination: None,
            drill_down_active: false,
            drill_down_connections: Vec::new(),
            selected_connection_id: None,
            search_query: String::new(),
            dest_sort_column: DestSortColumn::LastSeen,
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
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CopyToClipboard(text) => {
                return iced::clipboard::write(text);
            }

            Message::CopySelected => {
                if let Some(text) = self.selected_copy_text() {
                    return Task::done(Message::CopyToClipboard(text));
                }
                return Task::none();
            }

            Message::ConnectionReceived(event) => {
                if self.is_paused {
                    return Task::none();
                }

                let conn = ConnectionDisplay::from(event);

                // Key by (process, IP address) so connections to the same IP always group together
                // even if DNS wasn't resolved yet for the first connection
                let dest_key = (conn.process_name.clone(), conn.dst_addr.clone());
                let is_new_dest = !self.destinations.contains_key(&dest_key);

                // Update process summary
                let entry = self.processes
                    .entry(conn.process_name.clone())
                    .or_insert_with(|| ProcessSummary {
                        name: conn.process_name.clone(),
                        connection_count: 0,
                        destination_count: 0,
                        last_seen: SystemTime::now(),
                    });
                entry.connection_count += 1;
                entry.last_seen = SystemTime::now();
                if is_new_dest {
                    entry.destination_count += 1;
                }

                // Update aggregated destinations
                let zone = classify_zone(&conn.dst_addr);
                self.destinations
                    .entry(dest_key)
                    .and_modify(|dest| {
                        dest.connection_count += 1;
                        dest.ports.insert(conn.dst_port);
                        dest.protocols.insert(conn.protocol.clone());
                        dest.last_seen = SystemTime::now();
                        dest.connection_ids.push(conn.id);
                        // Late domain enrichment: if we now have a domain, update display
                        if dest.domain.is_empty() && !conn.domain.is_empty() {
                            dest.domain = conn.domain.clone();
                            dest.display_dest = conn.display_dest.clone();
                        }
                    })
                    .or_insert_with(|| DestinationSummary {
                        process_name: conn.process_name.clone(),
                        display_dest: conn.display_dest.clone(),
                        dst_addr: conn.dst_addr.clone(),
                        domain: conn.domain.clone(),
                        connection_count: 1,
                        ports: BTreeSet::from([conn.dst_port]),
                        protocols: BTreeSet::from([conn.protocol.clone()]),
                        first_seen: SystemTime::now(),
                        last_seen: SystemTime::now(),
                        zone,
                        connection_ids: vec![conn.id],
                    });

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

                self.rebuild_destinations();

                // If in drill-down, update that too
                if self.drill_down_active {
                    if let Some(ref dest_name) = self.selected_destination {
                        self.rebuild_drill_down(dest_name.clone());
                    }
                }
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
                self.rebuild_destinations();
            }

            Message::ProcessSelected(process) => {
                self.selected_process = process;
                self.selected_destination = None;
                self.drill_down_active = false;
                self.drill_down_connections.clear();
                self.selected_connection_id = None;
                self.rebuild_destinations();
            }

            Message::TrafficZoneChanged(zone) => {
                self.traffic_zone = zone;
                self.selected_destination = None;
                self.drill_down_active = false;
                self.drill_down_connections.clear();
                self.rebuild_destinations();
            }

            Message::DestinationSelected(dest) => {
                self.selected_destination = dest;
            }

            Message::DestSortChanged(col) => {
                self.dest_sort_column = col;
                self.rebuild_destinations();
            }

            Message::DrillDown(dst_addr) => {
                self.drill_down_active = true;
                self.selected_destination = Some(dst_addr.clone());
                self.selected_connection_id = None;
                self.rebuild_drill_down(dst_addr);
            }

            Message::DrillDownBack => {
                self.drill_down_active = false;
                self.drill_down_connections.clear();
                self.selected_connection_id = None;
            }

            Message::ConnectionSelected(id) => {
                self.selected_connection_id = id;
            }

            Message::ProtocolFilterChanged(filter) => {
                self.protocol_filter = filter;
                self.rebuild_destinations();
            }

            Message::TogglePause => {
                self.is_paused = !self.is_paused;
            }

            Message::ToggleAutoScroll => {
                self.auto_scroll = !self.auto_scroll;
            }

            Message::Quit => {
                std::process::exit(0);
            }

            Message::ClearConnections => {
                self.connections.clear();
                self.destinations.clear();
                self.filtered_destinations.clear();
                self.processes.clear();
                self.selected_process = None;
                self.selected_destination = None;
                self.drill_down_active = false;
                self.drill_down_connections.clear();
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
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Find selected destination summary for detail panel (keyed by dst_addr)
        let selected_dest = self.selected_destination.as_ref().and_then(|dst_addr| {
            self.filtered_destinations.iter().find(|d| &d.dst_addr == dst_addr)
        });

        // Find selected connection for drill-down detail
        let selected_conn = self.selected_connection_id.and_then(|id| {
            self.drill_down_connections.iter().find(|c| c.id == id)
        });

        let toolbar = view::toolbar::view(
            &self.search_query,
            self.protocol_filter,
            self.is_paused,
            self.auto_scroll,
            self.filtered_destinations.len(),
            self.total_events,
        );

        let sidebar = view::sidebar::view(&self.processes, &self.selected_process);

        let show_process_column = self.selected_process.is_none();
        let table = view::connection_table::view(
            &self.filtered_destinations,
            &self.selected_destination,
            self.dest_sort_column,
            self.traffic_zone,
            self.drill_down_active,
            &self.drill_down_connections,
            self.selected_connection_id,
            show_process_column,
        );

        let detail = view::detail_panel::view(
            selected_dest,
            selected_conn,
            self.drill_down_active,
        );

        let status = view::status_bar::view(
            self.is_connected,
            self.total_events,
            self.processes.len(),
            self.destinations.len(),
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
        Subscription::batch([
            daemon_sub::daemon_events(),
            iced::keyboard::on_key_press(|key, modifiers| {
                if modifiers.control() {
                    if let iced::keyboard::Key::Character(c) = &key {
                        if c.as_str() == "c" {
                            return Some(Message::CopySelected);
                        }
                    }
                }
                None
            }),
        ])
    }

    fn selected_copy_text(&self) -> Option<String> {
        if self.drill_down_active {
            self.selected_connection_id.and_then(|id| {
                self.drill_down_connections.iter().find(|c| c.id == id).map(|conn| {
                    let dest = if conn.domain.is_empty() {
                        format!("{}:{}", conn.dst_addr, conn.dst_port)
                    } else {
                        format!("{} ({}):{}",  conn.domain, conn.dst_addr, conn.dst_port)
                    };
                    format!("{}\t{}\t{}\t{}\t{}", conn.process_name, conn.pid, dest, conn.protocol, conn.username)
                })
            })
        } else {
            self.selected_destination.as_ref().and_then(|dst_addr| {
                self.filtered_destinations.iter().find(|d| &d.dst_addr == dst_addr).map(|dest| {
                    let ports = dest.ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");
                    let protos = dest.protocols.iter().cloned().collect::<Vec<_>>().join("/");
                    format!("{}\t{}\t{}\t{}\t{}", dest.process_name, dest.display_dest, dest.dst_addr, ports, protos)
                })
            })
        }
    }

    fn rebuild_destinations(&mut self) {
        let query = self.search_query.to_lowercase();

        self.filtered_destinations = self.destinations
            .values()
            .filter(|dest| {
                // Process filter
                if let Some(ref proc_name) = self.selected_process {
                    if &dest.process_name != proc_name {
                        return false;
                    }
                }

                // Zone filter
                if dest.zone != self.traffic_zone {
                    return false;
                }

                // Protocol filter
                match self.protocol_filter {
                    ProtocolFilter::All => {}
                    ProtocolFilter::TcpOnly => {
                        if !dest.protocols.contains("TCP") {
                            return false;
                        }
                    }
                    ProtocolFilter::UdpOnly => {
                        if !dest.protocols.contains("UDP") {
                            return false;
                        }
                    }
                }

                // Search
                if !query.is_empty() {
                    let matches = dest.display_dest.to_lowercase().contains(&query)
                        || dest.dst_addr.to_lowercase().contains(&query)
                        || dest.domain.to_lowercase().contains(&query)
                        || dest.process_name.to_lowercase().contains(&query);
                    if !matches {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort
        match self.dest_sort_column {
            DestSortColumn::Destination => {
                self.filtered_destinations.sort_by(|a, b| a.display_dest.cmp(&b.display_dest));
            }
            DestSortColumn::Count => {
                self.filtered_destinations.sort_by(|a, b| b.connection_count.cmp(&a.connection_count));
            }
            DestSortColumn::LastSeen => {
                self.filtered_destinations.sort_by(|a, b| b.last_seen.cmp(&a.last_seen));
            }
            DestSortColumn::Ports => {
                self.filtered_destinations.sort_by(|a, b| b.ports.len().cmp(&a.ports.len()));
            }
        }
    }

    fn rebuild_drill_down(&mut self, dest_key: String) {
        let process = self.selected_process.clone();

        // dest_key is the dst_addr (IP), match connections by IP
        self.drill_down_connections = self.connections
            .iter()
            .filter(|c| {
                if let Some(ref proc_name) = process {
                    if &c.process_name != proc_name {
                        return false;
                    }
                }
                c.dst_addr == dest_key
            })
            .cloned()
            .collect();
    }
}
