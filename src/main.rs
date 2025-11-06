mod config;
mod cache;
mod logger;
mod controllers;

use axum::{Router, middleware, routing::any};
use tokio::{
    sync::RwLock,
    net::TcpListener,
};
use std::sync::Arc;
use config::Config;
use tracing::{info, warn};

use cache::BannedIpsCache;
use logger::setup_logging;

use crate::cache::cache_refresh_task;
// Shared State
#[derive(Clone)]
struct AppState {
    banned_ips: Arc<RwLock<BannedIpsCache>>,
    config: Config,
}
#[tokio::main]
async fn main() {
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
        .layer(middleware::from_fn(controllers::auth_middleware))
        .layer(axum::Extension(state));

    // Start server
    let addr = format!("{}:{}", config.hostname, config.port);
    info!("Starting server on {}", addr);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind address");
    axum::serve(listener, app).await.expect("Server failed");
}