//! Logging configuration and setup.
//!
//! This module configures the tracing subscriber with both file and console output,
//! including log rotation and environment-based log level filtering.

use crate::config::{Config, LogRotation};
use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Sets up logging with file rotation and console output.
///
/// Configures a tracing subscriber with:
/// - File output with configurable rotation (hourly, daily, or never)
/// - Console output to stdout
/// - Log level filtering via `RUST_LOG` environment variable (defaults to "info")
/// - Automatic log file management with maximum file retention
///
/// # Arguments
/// * `config` - Configuration containing log file settings
///
/// # Log File Naming
/// The log files are created based on the configured log_file path:
/// - Prefix: Filename without extension (e.g., "traefik-auth" from "traefik-auth.log")
/// - Suffix: File extension (e.g., "log")
/// - Rotation files are automatically named with timestamps
///
/// # Environment Variables for Log Levels
/// Set `RUST_LOG` to control log verbosity. Examples:
/// - `RUST_LOG=trace` - Most verbose, shows all logs
/// - `RUST_LOG=debug` - Debug and above (debug, info, warn, error)
/// - `RUST_LOG=info` - Info and above (default)
/// - `RUST_LOG=warn` - Warnings and errors only
/// - `RUST_LOG=error` - Errors only
///
/// ## Advanced Filtering
/// You can filter by module or target:
/// - `RUST_LOG=tezcatlipoca_auth=debug` - Debug level for this crate only
/// - `RUST_LOG=info,tezcatlipoca_auth::cache=debug` - Info globally, debug for cache module
/// - `RUST_LOG=warn,tezcatlipoca_auth::controllers=trace` - Warn globally, trace for controllers
///
/// # Example Usage
/// ```bash
/// # Show all info, warn, and error logs
/// RUST_LOG=info cargo run
///
/// # Debug mode for troubleshooting
/// RUST_LOG=debug cargo run
///
/// # Only show warnings and errors
/// RUST_LOG=warn cargo run
///
/// # Fine-grained control
/// RUST_LOG=info,tezcatlipoca_auth::controllers=debug cargo run
/// ```
///
/// # Errors
/// Returns an error if the log file appender cannot be created (e.g., permission issues)
pub fn setup_logging(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
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
        .map_err(|e| {
            eprintln!("ERROR: Failed to create log file appender at directory '{}': {}", config.log_dir, e);
            eprintln!("  Log file would be: {}/{}.{}", config.log_dir, log_prefix, log_suffix);
            eprintln!("  Make sure the directory exists and has write permissions");
            e
        })?;

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(false);

    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(false);

    // Build EnvFilter with fallback to config or "info"
    // Priority: RUST_LOG env var > explicit config > "info" default
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap_or_else(|_| {
            // Fallback if both fail (shouldn't happen with "info")
            EnvFilter::new("info")
        });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    Ok(())
}
