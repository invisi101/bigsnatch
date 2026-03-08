use crate::model::destination::TrafficZone;
use crate::proto;

#[derive(Debug, Clone)]
pub enum Message {
    // Events from daemon
    ConnectionReceived(proto::ConnectionEvent),
    DaemonConnected,
    DaemonDisconnected(String),
    DaemonError(String),

    // User interactions
    SearchChanged(String),
    ProcessSelected(Option<String>),
    ProtocolFilterChanged(ProtocolFilter),
    TogglePause,
    ToggleAutoScroll,
    ClearConnections,
    Quit,

    // Navigation
    TrafficZoneChanged(TrafficZone),
    DestinationSelected(Option<String>),
    DestSortChanged(DestSortColumn),
    DrillDown(String),
    DrillDownBack,
    ConnectionSelected(Option<u64>),

    // Internal
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestSortColumn {
    Destination,
    Count,
    LastSeen,
    Ports,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolFilter {
    All,
    TcpOnly,
    UdpOnly,
}
