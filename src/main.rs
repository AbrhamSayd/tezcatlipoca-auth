//! Tezcatlipoca Authentication Service
//!
//! A high-performance authentication service for Traefik's ForwardAuth system.
//! Provides IP-based access control with automatic cache management and
//! comprehensive logging.
//!
//! # Features
//! - IP-based authentication and blocking
//! - Automatic cache refresh for banned IPs
//! - Cloudflare and proxy support (X-Forwarded-For)
//! - Configurable log rotation
//! - Health check endpoint with metrics
//! - Zero-downtime cache updates
//!
//! # Environment Variables
//! See `config` module for full list of configuration options.
//!
//! # Architecture
//! - `controllers`: HTTP handlers and authentication middleware
//! - `cache`: In-memory IP cache with background refresh
//! - `config`: Configuration management
//! - `logger`: Structured logging setup

mod cache;
mod config;
mod controllers;
mod logger;

use axum::{middleware, routing::any, Router};
use config::Config;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::RwLock};
use tracing::{info, warn};

use cache::BannedIpsCache;
use logger::setup_logging;

use crate::cache::cache_refresh_task;

/// Shared application state accessible across all handlers.
///
/// Contains the banned IPs cache and configuration, wrapped in Arc
/// for efficient cloning across async tasks.
#[derive(Clone)]
struct AppState {
    /// Thread-safe cache of banned IP addresses
    banned_ips: Arc<RwLock<BannedIpsCache>>,
    /// Application configuration
    config: Config,
}

/// Application entry point.
///
/// Initializes the service by:
/// 1. Loading .env file if present
/// 2. Loading configuration from environment
/// 3. Setting up structured logging
/// 4. Initializing the banned IPs cache
/// 5. Spawning background cache refresh task
/// 6. Starting the HTTP server with authentication middleware
///
/// # Panics
/// - If the server address cannot be bound
/// - If the server fails to start
#[tokio::main]
async fn main() {
    // Load .env file if present (fails silently if not found)
    dotenvy::dotenv().ok();

    let config = Config::from_env();

    //setup loggin
    setup_logging(&config);

    info!("Configuration loaded:");
    info!("  Banned IPs file: {}", config.banned_ips_file);
    info!("  Cache TTL: {:?}", config.cache_ttl);
    info!("  Log file: {}", config.log_file);
    info!("  Log dir: {}", config.log_dir);
    info!("  Log rotation: {:?}", config.log_rotation);
    info!("  Log max files: {}", config.log_max_files);
    info!("  Port: {}", config.port);
    info!("  Hostname: {}", config.hostname);

    // Initialize state
    let state = AppState {
        banned_ips: Arc::new(RwLock::new(BannedIpsCache::new(config.cache_ttl))),
        config: config.clone(),
    };

    //load initial banned Ips
    {
        let mut cache = state.banned_ips.write().await;
        if let Err(e) = cache.refresh(&state.config.banned_ips_file).await {
            warn!("Failed to load initial banned IPs: {}", e);
        }
    }

    // spawn background cache refresh task
    let refresh_state = state.clone();
    tokio::spawn(async move {
        cache_refresh_task(refresh_state).await;
    });

    //build router with middleware
    let app = Router::new()
        .route("/health", any(controllers::health_check))
        .with_state(state.clone())
        .route("/{*path}", any(controllers::handler))
        .route("/", any(controllers::handler))
        .layer(middleware::from_fn_with_state(state.clone(), controllers::auth_middleware));

    // Start server
    let addr = format!("{}:{}", config.hostname, config.port);
    info!("Starting server on {}", addr);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");
    
    // Use into_make_service_with_connect_info to access socket address
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("Server failed");
}