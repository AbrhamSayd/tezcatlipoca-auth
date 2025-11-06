use axum::{
    extract::{Request, State}, 
    http::{HeaderMap, StatusCode}, 
    middleware::Next, 
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::{info, warn};

use crate::AppState;

// === Auth Middleware ===
pub async fn auth_middleware(
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let state = req.extensions().get::<AppState>().cloned().unwrap();
    
    // Extract client IP from headers (Cloudflare, X-Forwarded-For, or direct)
    let client_ip = headers
        .get("cf-connecting-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim())
        .unwrap_or("unknown");

    let path = req.uri().path();

    // Check if IP is banned
    let mut cache = state.banned_ips.write().await;
    
    // Refresh cache if needed
    if cache.is_stale(state.config.cache_ttl) {
        if let Err(e) = cache.refresh(&state.config.banned_ips_file).await {
            warn!("Failed to refresh banned IPs cache: {}", e);
        }
    }

    if cache.contains(client_ip) {
        warn!("Blocked IP: {} tried to access {}", client_ip, path);
        return Err(StatusCode::FORBIDDEN);
    }

    drop(cache); // Release the lock before continuing

    info!("Allowed IP: {} accessed {}", client_ip, path);
    
    Ok(next.run(req).await)
}

// === Handler for all routes ===
pub async fn handler() -> impl IntoResponse {
    StatusCode::OK
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    banned_ip_count: usize,
}

// === Health check handler ===
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let cache = state.banned_ips.read().await;
    let count = cache.ips.len();
    drop(cache);
    
    Json(HealthResponse {
        status: "ok".to_string(),
        banned_ip_count: count,
    })
}