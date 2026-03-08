use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct DnsEntry {
    domain: String,
    inserted_at: Instant,
    ttl: Duration,
}

pub struct DnsCache {
    cache: Mutex<HashMap<IpAddr, DnsEntry>>,
    max_entries: usize,
}

impl DnsCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            max_entries,
        }
    }

    pub fn insert(&self, ip: IpAddr, domain: String, ttl_secs: u32) {
        let mut cache = self.cache.lock().unwrap();

        // Evict expired entries if we're at capacity
        if cache.len() >= self.max_entries {
            let now = Instant::now();
            cache.retain(|_, entry| now.duration_since(entry.inserted_at) < entry.ttl);
        }

        // Use a minimum TTL of 60 seconds to handle very short DNS TTLs
        let ttl = Duration::from_secs(ttl_secs.max(60) as u64);

        cache.insert(
            ip,
            DnsEntry {
                domain,
                inserted_at: Instant::now(),
                ttl,
            },
        );
    }

    pub fn lookup(&self, ip: &IpAddr) -> Option<String> {
        let cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.get(ip) {
            if entry.inserted_at.elapsed() < entry.ttl {
                return Some(entry.domain.clone());
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.cache.lock().unwrap().len()
    }
}
