use std::{
    collections::HashSet,
    time::{Duration, Instant},
};
use tokio::fs::{self, read_to_string};
use tracing::{warn, debug};


pub struct BannedIpsCache {
    pub ips: HashSet<String>,
    pub last_read: Instant,
}

impl BannedIpsCache {
    pub fn new(cache_ttl: Duration) -> Self {
        Self {
            ips: HashSet::new(),
            last_read: Instant::now() - cache_ttl,
        }
    }

    pub fn is_stale(&self, cache_ttl: Duration) -> bool {
        self.last_read.elapsed() >= cache_ttl
    }

    pub async fn refresh(&mut self, banned_ips_file: &str) -> std::io::Result<()> {
        let content = read_to_string(banned_ips_file).await?;
        self.ips = content.lines().map(|line| line.trim().to_string()).collect();
        self.last_read = Instant::now();
        debug!("Banned IPs cache refreshed with {} entries", self.ips.len());
        Ok(())
    }

    pub fn contains(&self, ip: &str) -> bool {
        self.ips.contains(ip)
    }
}


async fn read_banned_ips(banned_ips_file: &str) -> std::io::Result<HashSet<String>> {
   match fs::read_to_string(banned_ips_file).await {
       Ok(content) => {
           let ips: HashSet<String> = content.lines().map(|line| line.trim().to_string()).collect();
           Ok(ips)
       }
       Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
           warn!("Banned IPs file not found: {}", banned_ips_file);
           Ok(HashSet::new())
       }
         Err(e) => Err(e),
   }
}