use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub exe_path: String,
    pub cmdline: String,
    pub uid: u32,
    pub username: String,
}

pub struct ProcessCache {
    cache: Mutex<HashMap<u32, ProcessInfo>>,
}

impl ProcessCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_or_lookup(&self, pid: u32, uid: u32, comm: &str) -> ProcessInfo {
        let mut cache = self.cache.lock().unwrap();

        if let Some(info) = cache.get(&pid) {
            // Verify it's still the same process (comm matches)
            if info.name == comm || comm.is_empty() {
                return info.clone();
            }
            // PID was reused, remove stale entry
            cache.remove(&pid);
        }

        // Look up from /proc
        let info = Self::lookup_process(pid, uid, comm);
        cache.insert(pid, info.clone());
        info
    }

    pub fn remove(&self, pid: u32) {
        self.cache.lock().unwrap().remove(&pid);
    }

    pub fn active_count(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    fn lookup_process(pid: u32, uid: u32, comm: &str) -> ProcessInfo {
        let proc_dir = PathBuf::from(format!("/proc/{}", pid));

        let exe_path = fs::read_link(proc_dir.join("exe"))
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();

        let cmdline = fs::read_to_string(proc_dir.join("cmdline"))
            .map(|s| s.replace('\0', " ").trim().to_string())
            .unwrap_or_default();

        let name = if !comm.is_empty() {
            comm.to_string()
        } else {
            // Fall back to /proc/pid/comm
            fs::read_to_string(proc_dir.join("comm"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| format!("<pid:{}>", pid))
        };

        let username = resolve_username(uid);

        ProcessInfo {
            pid,
            name,
            exe_path,
            cmdline,
            uid,
            username,
        }
    }
}

fn resolve_username(uid: u32) -> String {
    // Read /etc/passwd to resolve uid -> username
    if let Ok(passwd) = fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 3 {
                if let Ok(entry_uid) = fields[2].parse::<u32>() {
                    if entry_uid == uid {
                        return fields[0].to_string();
                    }
                }
            }
        }
    }
    format!("{}", uid)
}
