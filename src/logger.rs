use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::filter::EnvFilter;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::Path;
use crate::config::{Config, LogRotation};

/// Sets up logging with file rotation and console output
pub fn setup_logging(config: &Config) {
    // Parse log file name (remove path and extension)
    let log_path = Path::new(&config.log_file);
    let log_prefix = log_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("traefik-auth");
    
    let log_suffix = log_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("log");

    // Determine rotation strategy
    let rotation = match config.log_rotation {
        LogRotation::Hourly => Rotation::HOURLY,
        LogRotation::Daily => Rotation::DAILY,
        LogRotation::Never => Rotation::NEVER,
    };

    // File appender with rotation
    let file_appender = RollingFileAppender::builder()
        .rotation(rotation)
        .filename_prefix(log_prefix)
        .filename_suffix(log_suffix)
        .max_log_files(config.log_max_files)
        .build(&config.log_dir)
        .expect("Failed to create log file appender");

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(false);

    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(false);

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with(file_layer)
        .with(console_layer)
        .init();
}
