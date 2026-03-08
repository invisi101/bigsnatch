#[derive(Debug, Clone)]
pub struct ProcessSummary {
    pub name: String,
    pub connection_count: usize,
    pub last_seen: std::time::SystemTime,
}
