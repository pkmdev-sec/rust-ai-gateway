use crate::{RouterError, RouterResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub attempts: u32,
    pub on_status_codes: Vec<u16>,
    pub use_retry_after_header: Option<bool>,
    pub exponential_backoff: Option<ExponentialBackoffConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExponentialBackoffConfig {
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            attempts: 3,
            on_status_codes: vec![429, 500, 502, 503, 504],
            use_retry_after_header: Some(true),
            exponential_backoff: Some(ExponentialBackoffConfig::default()),
        }
    }
}

impl Default for ExponentialBackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 100,
            max_delay_ms: 30000, // 30 seconds max
            multiplier: 2.0,
            jitter: true,
        }
    }
}

pub struct RetryHandler {
    config: RetryConfig,
}

impl RetryHandler {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub async fn execute_with_retry<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug + Clone,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.config.attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);
                    
                    // Don't sleep after the last attempt
                    if attempt < self.config.attempts {
                        let delay = self.calculate_delay(attempt);
                        sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    pub fn should_retry(&self, status_code: u16) -> bool {
        self.config.on_status_codes.contains(&status_code)
    }

    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        if let Some(backoff) = &self.config.exponential_backoff {
            let base_delay = backoff.initial_delay_ms as f64;
            let delay = base_delay * backoff.multiplier.powi(attempt as i32);
            let delay = delay.min(backoff.max_delay_ms as f64);
            
            if backoff.jitter {
                // Add random jitter (±25%)
                let jitter_factor = 1.0 + (rand::random::<f64>() - 0.5) * 0.5;
                (delay * jitter_factor) as u64
            } else {
                delay as u64
            }
        } else {
            // Simple linear backoff
            (attempt as u64 + 1) * 1000 // 1s, 2s, 3s, etc.
        }
    }

    pub fn parse_retry_after_header(&self, header_value: &str) -> Option<u64> {
        if !self.config.use_retry_after_header.unwrap_or(true) {
            return None;
        }

        // Try parsing as seconds first
        if let Ok(seconds) = header_value.trim().parse::<u64>() {
            return Some(seconds * 1000); // Convert to milliseconds
        }

        // Try parsing as HTTP date (RFC 7231)
        // For simplicity, we'll skip HTTP date parsing in this example
        // In a real implementation, you'd use a proper HTTP date parser
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let retry_handler = RetryHandler::new(RetryConfig::default());
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result = retry_handler.execute_with_retry(|| {
            let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                if count == 0 {
                    Err("First attempt fails")
                } else {
                    Ok("Success!")
                }
            }
        }).await;

        assert_eq!(result, Ok("Success!"));
        assert_eq!(attempt_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_exhaustion() {
        let config = RetryConfig {
            attempts: 2,
            ..Default::default()
        };
        let retry_handler = RetryHandler::new(config);
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result = retry_handler.execute_with_retry(|| {
            attempt_count_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                Err::<(), &str>("Always fails")
            }
        }).await;

        assert_eq!(result, Err("Always fails"));
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[test]
    fn test_should_retry() {
        let retry_handler = RetryHandler::new(RetryConfig::default());
        
        assert!(retry_handler.should_retry(429));
        assert!(retry_handler.should_retry(500));
        assert!(retry_handler.should_retry(502));
        assert!(!retry_handler.should_retry(200));
        assert!(!retry_handler.should_retry(404));
    }

    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig {
            exponential_backoff: Some(ExponentialBackoffConfig {
                initial_delay_ms: 100,
                max_delay_ms: 1000,
                multiplier: 2.0,
                jitter: false,
            }),
            ..Default::default()
        };
        let retry_handler = RetryHandler::new(config);

        assert_eq!(retry_handler.calculate_delay(0), 100);
        assert_eq!(retry_handler.calculate_delay(1), 200);
        assert_eq!(retry_handler.calculate_delay(2), 400);
        assert_eq!(retry_handler.calculate_delay(3), 800);
        assert_eq!(retry_handler.calculate_delay(4), 1000); // Capped at max
    }

    #[test]
    fn test_parse_retry_after_header() {
        let retry_handler = RetryHandler::new(RetryConfig::default());
        
        assert_eq!(retry_handler.parse_retry_after_header("30"), Some(30000));
        assert_eq!(retry_handler.parse_retry_after_header("0"), Some(0));
        assert_eq!(retry_handler.parse_retry_after_header("invalid"), None);
    }
}