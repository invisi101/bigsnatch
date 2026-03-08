use std::collections::BTreeSet;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficZone {
    External,
    Internal,
}

#[derive(Debug, Clone)]
pub struct DestinationSummary {
    pub process_name: String,
    pub display_dest: String,
    pub dst_addr: String,
    pub domain: String,
    pub connection_count: usize,
    pub ports: BTreeSet<u32>,
    pub protocols: BTreeSet<String>,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub zone: TrafficZone,
    pub connection_ids: Vec<u64>,
}

pub fn classify_zone(addr: &str) -> TrafficZone {
    // IPv4 loopback
    if addr.starts_with("127.") || addr == "0.0.0.0" {
        return TrafficZone::Internal;
    }
    // IPv4 private
    if addr.starts_with("10.") || addr.starts_with("192.168.") {
        return TrafficZone::Internal;
    }
    // 172.16.0.0/12
    if addr.starts_with("172.") {
        if let Some(second) = addr.split('.').nth(1).and_then(|s| s.parse::<u8>().ok()) {
            if (16..=31).contains(&second) {
                return TrafficZone::Internal;
            }
        }
    }
    // IPv4 link-local
    if addr.starts_with("169.254.") {
        return TrafficZone::Internal;
    }
    // IPv6 loopback and private
    if addr == "::1"
        || addr.starts_with("fe80:")
        || addr.starts_with("fd")
        || addr.starts_with("fc")
    {
        return TrafficZone::Internal;
    }
    TrafficZone::External
}
