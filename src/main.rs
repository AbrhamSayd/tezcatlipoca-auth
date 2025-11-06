mod config;
mod cache;
mod logger;


use std::{collections::HashSet, sync::{Arc, RwLock}, time::Instant};
use config::Config;
use tracing::info;

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

   
   //setup loggin
   logger::setup_logging(&config);
    
    info!("Configuration loaded:");
    info!("  Banned IPs file: {}", config.banned_ips_file);
    info!("  Cache TTL: {:?}", config.cache_ttl);
    info!("  Log file: {}", config.log_file);
    info!("  Log dir: {}", config.log_dir);
    info!("  Log rotation: {:?}", config.log_rotation);
    info!("  Log max files: {}", config.log_max_files);
    info!("  Port: {}", config.port);
   
}