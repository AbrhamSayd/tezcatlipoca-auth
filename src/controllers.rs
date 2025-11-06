//! Request handlers and middleware for the Traefik authentication service.
//!
//! This module contains the core HTTP handlers and authentication middleware
//! that integrates with Traefik's ForwardAuth system.

use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::{debug, warn};

use crate::AppState;

/// Authentication middleware that checks if client IP is banned.
///
/// This middleware integrates with Traefik's ForwardAuth to validate incoming requests.
/// It extracts the client IP from various headers (prioritizing Cloudflare and X-Forwarded-For),
/// checks it against the banned IPs cache, and either allows or blocks the request.
///
/// # Logging Strategy
/// - **WARN**: Blocked/banned IP attempts (always logged for security)
/// - **DEBUG**: Allowed connections (only when RUST_LOG=debug for troubleshooting)
/// - This keeps production logs focused on security events while allowing detailed
///   debugging when needed
///
/// # IP Detection Priority
/// 1. `cf-connecting-ip` - Cloudflare's real IP header
/// 2. `x-forwarded-for` - Standard proxy header (uses first IP if multiple)
/// 3. Socket address from connection info (direct connection)
///
/// # Cache Behavior
/// - Automatically refreshes the banned IPs cache if stale
/// - Blocks request with 403 FORBIDDEN if IP is banned
/// - Logs all access attempts based on log level configuration
///
/// # Arguments
/// * `State(state)` - Application state containing banned IPs cache and config
/// * `ConnectInfo(addr)` - Socket address of the client connection
/// * `headers` - HTTP request headers containing client IP information
/// * `req` - The incoming HTTP request
/// * `next` - Next middleware/handler in the chain
///
/// # Returns
/// * `Ok(Response)` - Request is allowed, continues to next handler
/// * `Err(StatusCode::FORBIDDEN)` - Request is blocked due to banned IP
pub async fn auth_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {

    // Extract client IP from headers (Cloudflare, X-Forwarded-For, or direct)
    let client_ip = headers
        .get("cf-connecting-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim())
        .unwrap_or_else(|| {
            // Fall back to socket address if no proxy headers
            addr.ip().to_string().leak()
        });

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
        warn!("🚫 BLOCKED: IP {} attempted to access {} [BANNED]", client_ip, path);
        return Err(StatusCode::FORBIDDEN);
    }

    drop(cache); // Release the lock before continuing

    // Log allowed connections at DEBUG level (won't show in production with RUST_LOG=info)
    debug!("✅ ALLOWED: IP {} accessed {}", client_ip, path);

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
