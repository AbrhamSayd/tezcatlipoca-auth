use std::{env, time::Duration};

/// Application configuration loaded from environment variables
#[derive(Clone, Debug)]
pub struct Config {
    pub banned_ips_file: String,
    pub cache_ttl: Duration,
    pub log_file: String,
    pub log_dir: String,
    pub log_rotation: LogRotation,
    pub log_max_files: usize,
    pub port: u16,
    pub hostname: String,
}

/// Log rotation strategy
#[derive(Clone, Debug)]
pub enum LogRotation {
    Hourly,
    Daily,
    Never,
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
        
        let log_dir = env::var("LOG_DIR")
            .unwrap_or_else(|_| ".".to_string());
        
        let log_rotation = match env::var("LOG_ROTATION")
            .unwrap_or_else(|_| "daily".to_string())
            .to_lowercase()
            .as_str()
        {
            "hourly" => LogRotation::Hourly,
            "daily" => LogRotation::Daily,
            "never" => LogRotation::Never,
            _ => LogRotation::Daily,
        };
        
        let log_max_files = env::var("LOG_MAX_FILES")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(7);

        let hostname = env::var("HOSTNAME")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(8199);

        Self {
            banned_ips_file,
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            log_file,
            log_dir,
            log_rotation,
            log_max_files,
            port,
            hostname,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            banned_ips_file: "./banned-ips.txt".to_string(),
            cache_ttl: Duration::from_secs(5),
            log_file: "./traefik-auth.log".to_string(),
            log_dir: ".".to_string(),
            log_rotation: LogRotation::Daily,
            log_max_files: 7,
            port: 8199,
            hostname: "0.0.0.0".to_string(),
        }
    }
}
