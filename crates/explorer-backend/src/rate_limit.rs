/// Rate limiting for explorer API
/// حد معدل الطلبات لواجهة برمجة تطبيقات المستكشف

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: usize,
    /// Time window duration
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 60,                     // 60 requests
            window: Duration::from_secs(60),      // per minute
        }
    }
}

/// Request tracking for an IP
#[derive(Debug, Clone)]
struct RequestTracker {
    /// Request timestamps in current window
    requests: Vec<Instant>,
}

impl RequestTracker {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    /// Clean up old requests outside the window
    fn cleanup(&mut self, window: Duration) {
        let cutoff = Instant::now() - window;
        self.requests.retain(|&timestamp| timestamp > cutoff);
    }

    /// Record a new request
    fn record_request(&mut self) {
        self.requests.push(Instant::now());
    }

    /// Get request count in current window
    fn count(&self) -> usize {
        self.requests.len()
    }
}

/// Rate limiter
#[derive(Clone)]
pub struct ExplorerRateLimiter {
    config: RateLimitConfig,
    trackers: Arc<RwLock<HashMap<IpAddr, RequestTracker>>>,
}

impl ExplorerRateLimiter {
    /// Create new rate limiter with default config
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    /// Create new rate limiter with custom config
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config,
            trackers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if request from IP should be allowed
    pub async fn check_rate_limit(&self, ip: IpAddr) -> Result<(), RateLimitError> {
        let mut trackers = self.trackers.write().await;
        let tracker = trackers.entry(ip).or_insert_with(RequestTracker::new);

        // Clean up old requests
        tracker.cleanup(self.config.window);

        // Check if limit exceeded
        if tracker.count() >= self.config.max_requests {
            return Err(RateLimitError::LimitExceeded {
                limit: self.config.max_requests,
                window_secs: self.config.window.as_secs(),
            });
        }

        // Record this request
        tracker.record_request();

        Ok(())
    }
}

/// Rate limit errors
#[derive(Debug)]
pub enum RateLimitError {
    LimitExceeded { limit: usize, window_secs: u64 },
}

#[derive(Serialize)]
struct RateLimitResponse {
    error: String,
    limit: usize,
    window_secs: u64,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        match self {
            RateLimitError::LimitExceeded { limit, window_secs } => {
                let response = RateLimitResponse {
                    error: format!(
                        "Rate limit exceeded: {} requests per {} seconds. Please slow down.",
                        limit, window_secs
                    ),
                    limit,
                    window_secs,
                };
                (StatusCode::TOO_MANY_REQUESTS, Json(response)).into_response()
            }
        }
    }
}

/// Extract IP address from request
fn extract_ip(request: &Request) -> IpAddr {
    // Try to get real IP from X-Forwarded-For header (for reverse proxies)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return ip;
                }
            }
        }
    }

    // Fallback to connection IP
    request
        .extensions()
        .get::<std::net::SocketAddr>()
        .map(|addr| addr.ip())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]))
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    limiter: Arc<ExplorerRateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    let ip = extract_ip(&request);
    limiter.check_rate_limit(ip).await?;
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_rate_limit_allows_within_limit() {
        let config = RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
        };
        let limiter = ExplorerRateLimiter::with_config(config);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));

        for _ in 0..5 {
            assert!(limiter.check_rate_limit(ip).await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_excess() {
        let config = RateLimitConfig {
            max_requests: 3,
            window: Duration::from_secs(60),
        };
        let limiter = ExplorerRateLimiter::with_config(config);
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        for _ in 0..3 {
            assert!(limiter.check_rate_limit(ip).await.is_ok());
        }

        // 4th request should fail
        assert!(limiter.check_rate_limit(ip).await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limit_window_cleanup() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_millis(100),
        };
        let limiter = ExplorerRateLimiter::with_config(config);
        let ip = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));

        limiter.check_rate_limit(ip).await.unwrap();
        limiter.check_rate_limit(ip).await.unwrap();

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should work again
        assert!(limiter.check_rate_limit(ip).await.is_ok());
    }
}
