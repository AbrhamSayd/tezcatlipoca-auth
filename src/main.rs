mod config;
mod cache;
use std::{collections::HashSet, sync::{Arc, RwLock}, time::Instant};
use config::Config;

struct BannedIpsCache {
    ips: HashSet<String>,
    last_read: Instant,
}

// Shared State
#[derive(Clone)]
struct AppState {
    banned_ips: Arc<RwLock<BannedIpsCache>>,
    config: Config,
}

fn main() {
   let config = Config::from_env();
   print!("Loaded config: {:?}", config);
}