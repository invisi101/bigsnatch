use crate::proto;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct ConnectionDisplay {
    pub id: u64,
    pub timestamp: SystemTime,
    pub pid: u32,
    pub process_name: String,
    pub exe_path: String,
    pub cmdline: String,
    pub uid: u32,
    pub username: String,
    pub protocol: String,
    pub src_addr: String,
    pub src_port: u32,
    pub dst_addr: String,
    pub dst_port: u32,
    pub domain: String,
    pub display_dest: String, // domain if available, otherwise IP
    pub time_str: String,
}

impl From<proto::ConnectionEvent> for ConnectionDisplay {
    fn from(e: proto::ConnectionEvent) -> Self {
        let protocol = if e.protocol == proto::Protocol::Tcp as i32 {
            "TCP".to_string()
        } else {
            "UDP".to_string()
        };

        let display_dest = if e.domain.is_empty() {
            e.dst_addr.clone()
        } else {
            e.domain.clone()
        };

        let now = chrono::Local::now();
        let time_str = now.format("%H:%M:%S").to_string();

        ConnectionDisplay {
            id: e.id,
            timestamp: SystemTime::now(),
            pid: e.pid,
            process_name: e.process_name,
            exe_path: e.exe_path,
            cmdline: e.cmdline,
            uid: e.uid,
            username: e.username,
            protocol,
            src_addr: e.src_addr,
            src_port: e.src_port,
            dst_addr: e.dst_addr,
            dst_port: e.dst_port,
            domain: e.domain,
            display_dest,
            time_str,
        }
    }
}
