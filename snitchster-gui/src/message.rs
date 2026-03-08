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
    ConnectionSelected(Option<u64>),
    ColumnSortChanged(SortColumn),
    TogglePause,
    ToggleAutoScroll,
    ProtocolFilterChanged(ProtocolFilter),
    ClearConnections,

    // Internal
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Time,
    Process,
    Pid,
    Domain,
    Port,
    Protocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolFilter {
    All,
    TcpOnly,
    UdpOnly,
}
