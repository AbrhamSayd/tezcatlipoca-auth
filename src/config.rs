use std::{env, time::Duration};

#[derive(Clone, Debug)]
pub struct Config {
    banned_ips_file: String,
    cache_ttl: Duration,
    log_file: String,
    port: u16,
}

impl Config {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Self {
        let banned_ips_file = env::var("BANNED_IPS_FILE")
            .unwrap_or_else(|_| "./banned-ips.txt".to_string());
        
        let cache_ttl_secs = env::var("CACHE_TTL_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(5);
        
        let log_file = env::var("LOG_FILE")
            .unwrap_or_else(|_| "./traefik-auth.log".to_string());
        
        let port = env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(8199);

        Self {
            banned_ips_file,
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            log_file,
            port,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            banned_ips_file: "./banned-ips.txt".to_string(),
            cache_ttl: Duration::from_secs(5),
            log_file: "./traefik-auth.log".to_string(),
            port: 8199,
        }
    }
}