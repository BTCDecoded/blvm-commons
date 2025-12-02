//! Internal API Authentication
//!
//! Provides authentication middleware for internal API endpoints used by
//! bllvm-node to forward P2P governance messages to bllvm-commons.

use axum::{
    extract::Request,
    http::{header::HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::env;
use tracing::{debug, warn};

/// API key header name
const API_KEY_HEADER: &str = "X-API-Key";

/// Validate internal API key from request headers
pub fn validate_api_key(headers: &HeaderMap) -> bool {
    let api_key = match headers.get(API_KEY_HEADER) {
        Some(header_value) => match header_value.to_str() {
            Ok(key) => key,
            Err(_) => {
                warn!("Invalid API key header encoding");
                return false;
            }
        },
        None => {
            debug!("Missing API key header");
            return false;
        }
    };

    let expected_key = match env::var("INTERNAL_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            warn!("INTERNAL_API_KEY not configured");
            return false;
        }
    };

    // Constant-time comparison to prevent timing attacks
    if api_key.len() != expected_key.len() {
        return false;
    }

    let mut result = 0u8;
    for (a, b) in api_key.bytes().zip(expected_key.bytes()) {
        result |= a ^ b;
    }
    result == 0
}

/// Middleware to validate internal API key
pub async fn internal_api_auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();

    if !validate_api_key(headers) {
        warn!("Internal API authentication failed");
        return Err(StatusCode::UNAUTHORIZED);
    }

    debug!("Internal API authentication successful");
    Ok(next.run(request).await)
}

