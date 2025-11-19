/// Rate limiting middleware to prevent DOS attacks
/// حد معدل الطلبات لمنع هجمات رفض الخدمة

use axum::{
    extract::{Request, State},
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
/// تكوين حد المعدل
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: usize,
    /// Time window duration
    pub window: Duration,
    /// Ban duration after exceeding limit
    pub ban_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,           // 100 requests
            window: Duration::from_secs(60), // per minute
            ban_duration: Duration::from_secs(300), // 5 minute ban
        }
    }
}

/// Request tracking for an IP
/// تتبع الطلبات لعنوان IP
#[derive(Debug, Clone)]
struct RequestTracker {
    /// Request timestamps in current window
    requests: Vec<Instant>,
    /// When the IP was banned (if applicable)
    banned_until: Option<Instant>,
}

impl RequestTracker {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            banned_until: None,
        }
    }

    /// Check if IP is currently banned
    fn is_banned(&self) -> bool {
        if let Some(banned_until) = self.banned_until {
            Instant::now() < banned_until
        } else {
            false
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

    /// Ban this IP
    fn ban(&mut self, duration: Duration) {
        self.banned_until = Some(Instant::now() + duration);
    }
}

/// Rate limiter
/// محدد المعدل
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    trackers: Arc<RwLock<HashMap<IpAddr, RequestTracker>>>,
}

impl RateLimiter {
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
    /// التحقق من أن الطلب من IP يجب السماح به
    pub async fn check_rate_limit(&self, ip: IpAddr) -> Result<(), RateLimitError> {
        let mut trackers = self.trackers.write().await;
        let tracker = trackers.entry(ip).or_insert_with(RequestTracker::new);

        // Check if IP is banned
        if tracker.is_banned() {
            return Err(RateLimitError::Banned);
        }

        // Clean up old requests
        tracker.cleanup(self.config.window);

        // Check if limit exceeded
        if tracker.count() >= self.config.max_requests {
            // Ban the IP
            tracker.ban(self.config.ban_duration);
            return Err(RateLimitError::LimitExceeded {
                limit: self.config.max_requests,
                window_secs: self.config.window.as_secs(),
            });
        }

        // Record this request
        tracker.record_request();

        Ok(())
    }

    /// Get current request count for an IP
    pub async fn get_request_count(&self, ip: IpAddr) -> usize {
        let trackers = self.trackers.read().await;
        trackers.get(&ip).map(|t| t.count()).unwrap_or(0)
    }

    /// Manually ban an IP
    /// حظر عنوان IP يدوياً
    pub async fn ban_ip(&self, ip: IpAddr, duration: Duration) {
        let mut trackers = self.trackers.write().await;
        let tracker = trackers.entry(ip).or_insert_with(RequestTracker::new);
        tracker.ban(duration);
    }

    /// Unban an IP
    /// إلغاء حظر عنوان IP
    pub async fn unban_ip(&self, ip: IpAddr) {
        let mut trackers = self.trackers.write().await;
        if let Some(tracker) = trackers.get_mut(&ip) {
            tracker.banned_until = None;
        }
    }
}

/// Rate limit errors
#[derive(Debug)]
pub enum RateLimitError {
    LimitExceeded { limit: usize, window_secs: u64 },
    Banned,
}

#[derive(Serialize)]
struct RateLimitResponse {
    error: String,
    limit: Option<usize>,
    window_secs: Option<u64>,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        let (status, response) = match self {
            RateLimitError::LimitExceeded { limit, window_secs } => (
                StatusCode::TOO_MANY_REQUESTS,
                RateLimitResponse {
                    error: format!(
                        "Rate limit exceeded: {} requests per {} seconds",
                        limit, window_secs
                    ),
                    limit: Some(limit),
                    window_secs: Some(window_secs),
                },
            ),
            RateLimitError::Banned => (
                StatusCode::FORBIDDEN,
                RateLimitResponse {
                    error: "IP address is temporarily banned due to excessive requests".to_string(),
                    limit: None,
                    window_secs: None,
                },
            ),
        };

        (status, Json(response)).into_response()
    }
}

/// Rate limiting middleware
/// برمجية وسيطة لتحديد المعدل
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    // Extract IP address from request
    // In production, should check X-Forwarded-For for proxied requests
    let ip = request
        .extensions()
        .get::<std::net::SocketAddr>()
        .map(|addr| addr.ip())
        .unwrap_or(IpAddr::from([127, 0, 0, 1])); // Fallback to localhost

    // Check rate limit
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
            ban_duration: Duration::from_secs(300),
        };
        let limiter = RateLimiter::with_config(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Should allow first 5 requests
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(ip).await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_excess_requests() {
        let config = RateLimitConfig {
            max_requests: 3,
            window: Duration::from_secs(60),
            ban_duration: Duration::from_secs(300),
        };
        let limiter = RateLimiter::with_config(config);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));

        // First 3 requests should succeed
        for _ in 0..3 {
            assert!(limiter.check_rate_limit(ip).await.is_ok());
        }

        // 4th request should fail
        let result = limiter.check_rate_limit(ip).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rate_limit_window_cleanup() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_millis(100),
            ban_duration: Duration::from_secs(300),
        };
        let limiter = RateLimiter::with_config(config);
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        // Make 2 requests
        limiter.check_rate_limit(ip).await.unwrap();
        limiter.check_rate_limit(ip).await.unwrap();

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be able to make requests again
        assert!(limiter.check_rate_limit(ip).await.is_ok());
    }

    #[tokio::test]
    async fn test_ban_and_unban() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));

        // Ban the IP
        limiter.ban_ip(ip, Duration::from_secs(60)).await;

        // Requests should be blocked
        assert!(limiter.check_rate_limit(ip).await.is_err());

        // Unban the IP
        limiter.unban_ip(ip).await;

        // Requests should work again
        assert!(limiter.check_rate_limit(ip).await.is_ok());
    }

    #[tokio::test]
    async fn test_different_ips_independent() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(60),
            ban_duration: Duration::from_secs(300),
        };
        let limiter = RateLimiter::with_config(config);

        let ip1 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));

        // Exhaust limit for IP1
        limiter.check_rate_limit(ip1).await.unwrap();
        limiter.check_rate_limit(ip1).await.unwrap();
        assert!(limiter.check_rate_limit(ip1).await.is_err());

        // IP2 should still work
        assert!(limiter.check_rate_limit(ip2).await.is_ok());
    }
}
