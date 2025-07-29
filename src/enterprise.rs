// Enterprise Features Module
// Provides advanced enterprise capabilities natively

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Enterprise configuration for the AI Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// Enable enterprise features
    pub enabled: bool,
    
    /// Observability configuration
    pub observability: ObservabilityConfig,
    
    /// Governance configuration
    pub governance: GovernanceConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Caching configuration
    pub caching: CachingConfig,
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            observability: ObservabilityConfig::default(),
            governance: GovernanceConfig::default(),
            security: SecurityConfig::default(),
            caching: CachingConfig::default(),
        }
    }
}

/// Observability and monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Enable detailed logging
    pub logging_enabled: bool,
    
    /// Enable metrics collection
    pub metrics_enabled: bool,
    
    /// Enable tracing
    pub tracing_enabled: bool,
    
    /// Log level (debug, info, warn, error)
    pub log_level: String,
    
    /// Metrics retention period in days
    pub metrics_retention_days: u32,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            logging_enabled: true,
            metrics_enabled: true,
            tracing_enabled: true,
            log_level: "info".to_string(),
            metrics_retention_days: 30,
        }
    }
}

/// Governance and access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Enable budget controls
    pub budget_controls_enabled: bool,
    
    /// Enable rate limiting
    pub rate_limiting_enabled: bool,
    
    /// Enable access control
    pub access_control_enabled: bool,
    
    /// Default budget limit per API key (USD)
    pub default_budget_limit: f64,
    
    /// Default rate limit (requests per minute)
    pub default_rate_limit: u32,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            budget_controls_enabled: true,
            rate_limiting_enabled: true,
            access_control_enabled: true,
            default_budget_limit: 100.0,
            default_rate_limit: 1000,
        }
    }
}

/// Security and guardrails configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable content filtering
    pub content_filtering_enabled: bool,
    
    /// Enable PII detection
    pub pii_detection_enabled: bool,
    
    /// Enable custom guardrails
    pub custom_guardrails_enabled: bool,
    
    /// Blocked content patterns
    pub blocked_patterns: Vec<String>,
    
    /// PII patterns to detect
    pub pii_patterns: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            content_filtering_enabled: true,
            pii_detection_enabled: true,
            custom_guardrails_enabled: true,
            blocked_patterns: vec![
                r"(?i)(password|secret|key|token)\s*[:=]\s*\S+".to_string(),
                r"(?i)(api[_-]?key|access[_-]?token)\s*[:=]\s*\S+".to_string(),
            ],
            pii_patterns: vec![
                r"\b\d{3}-\d{2}-\d{4}\b".to_string(), // SSN
                r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b".to_string(), // Credit card
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(), // Email
            ],
        }
    }
}

/// Caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    /// Enable response caching
    pub enabled: bool,
    
    /// Default TTL in seconds
    pub default_ttl: u64,
    
    /// Maximum cache size in MB
    pub max_size_mb: u64,
    
    /// Cache mode (simple, semantic)
    pub mode: String,
}

impl Default for CachingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: 300, // 5 minutes
            max_size_mb: 100,
            mode: "simple".to_string(),
        }
    }
}

/// Request metadata for enterprise tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// Unique request ID
    pub request_id: String,
    
    /// API key used
    pub api_key: Option<String>,
    
    /// User ID
    pub user_id: Option<String>,
    
    /// Organization ID
    pub org_id: Option<String>,
    
    /// Team/department
    pub team: Option<String>,
    
    /// Environment (dev, staging, prod)
    pub environment: Option<String>,
    
    /// Custom tags
    pub tags: Vec<String>,
    
    /// Request timestamp
    pub timestamp: u64,
    
    /// Client IP address
    pub client_ip: Option<String>,
    
    /// User agent
    pub user_agent: Option<String>,
}

impl RequestMetadata {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            api_key: None,
            user_id: None,
            org_id: None,
            team: None,
            environment: None,
            tags: Vec::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            client_ip: None,
            user_agent: None,
        }
    }
}

/// Comprehensive metrics for enterprise monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseMetrics {
    /// Request metrics
    pub requests: RequestMetrics,
    
    /// Cost metrics
    pub costs: CostMetrics,
    
    /// Performance metrics
    pub performance: PerformanceMetrics,
    
    /// Error metrics
    pub errors: ErrorMetrics,
    
    /// Provider metrics
    pub providers: HashMap<String, ProviderMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Total requests
    pub total: u64,
    
    /// Successful requests
    pub successful: u64,
    
    /// Failed requests
    pub failed: u64,
    
    /// Requests per second
    pub rps: f64,
    
    /// Requests by model
    pub by_model: HashMap<String, u64>,
    
    /// Requests by user
    pub by_user: HashMap<String, u64>,
    
    /// Requests by team
    pub by_team: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    /// Total cost in USD
    pub total_usd: f64,
    
    /// Cost by model
    pub by_model: HashMap<String, f64>,
    
    /// Cost by user
    pub by_user: HashMap<String, f64>,
    
    /// Cost by team
    pub by_team: HashMap<String, f64>,
    
    /// Token usage
    pub tokens: TokenMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    /// Total input tokens
    pub input_tokens: u64,
    
    /// Total output tokens
    pub output_tokens: u64,
    
    /// Total tokens
    pub total_tokens: u64,
    
    /// Average tokens per request
    pub avg_tokens_per_request: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// P50 response time
    pub p50_response_time_ms: f64,
    
    /// P95 response time
    pub p95_response_time_ms: f64,
    
    /// P99 response time
    pub p99_response_time_ms: f64,
    
    /// Average routing time in microseconds
    pub avg_routing_time_us: f64,
    
    /// Throughput (requests per second)
    pub throughput_rps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total errors
    pub total: u64,
    
    /// Errors by status code
    pub by_status_code: HashMap<u16, u64>,
    
    /// Errors by provider
    pub by_provider: HashMap<String, u64>,
    
    /// Error rate percentage
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    /// Provider name
    pub name: String,
    
    /// Total requests to this provider
    pub requests: u64,
    
    /// Successful requests
    pub successful: u64,
    
    /// Failed requests
    pub failed: u64,
    
    /// Average response time
    pub avg_response_time_ms: f64,
    
    /// Success rate
    pub success_rate: f64,
    
    /// Total cost
    pub total_cost_usd: f64,
}

/// Budget control for API keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetControl {
    /// API key
    pub api_key: String,
    
    /// Budget limit in USD
    pub limit_usd: f64,
    
    /// Current spend in USD
    pub current_spend_usd: f64,
    
    /// Budget period (daily, weekly, monthly)
    pub period: String,
    
    /// Period start timestamp
    pub period_start: u64,
    
    /// Enabled/disabled
    pub enabled: bool,
    
    /// Alert thresholds (percentages)
    pub alert_thresholds: Vec<f64>,
}

impl BudgetControl {
    pub fn new(api_key: String, limit_usd: f64) -> Self {
        Self {
            api_key,
            limit_usd,
            current_spend_usd: 0.0,
            period: "monthly".to_string(),
            period_start: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            enabled: true,
            alert_thresholds: vec![50.0, 80.0, 95.0],
        }
    }
    
    pub fn is_over_budget(&self) -> bool {
        self.enabled && self.current_spend_usd >= self.limit_usd
    }
    
    pub fn remaining_budget(&self) -> f64 {
        (self.limit_usd - self.current_spend_usd).max(0.0)
    }
    
    pub fn usage_percentage(&self) -> f64 {
        if self.limit_usd > 0.0 {
            (self.current_spend_usd / self.limit_usd * 100.0).min(100.0)
        } else {
            0.0
        }
    }
}

/// Rate limiting for API keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// API key
    pub api_key: String,
    
    /// Requests per minute limit
    pub requests_per_minute: u32,
    
    /// Current request count in window
    pub current_requests: u32,
    
    /// Window start timestamp
    pub window_start: u64,
    
    /// Enabled/disabled
    pub enabled: bool,
}

impl RateLimit {
    pub fn new(api_key: String, requests_per_minute: u32) -> Self {
        Self {
            api_key,
            requests_per_minute,
            current_requests: 0,
            window_start: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            enabled: true,
        }
    }
    
    pub fn is_rate_limited(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Reset window if more than 1 minute has passed
        if now - self.window_start >= 60 {
            self.window_start = now;
            self.current_requests = 0;
        }
        
        if self.current_requests >= self.requests_per_minute {
            return true;
        }
        
        self.current_requests += 1;
        false
    }
}

/// Enterprise manager for all enterprise features
pub struct EnterpriseManager {
    /// Configuration
    config: EnterpriseConfig,
    
    /// Metrics storage
    metrics: Arc<Mutex<EnterpriseMetrics>>,
    
    /// Budget controls by API key
    budget_controls: Arc<Mutex<HashMap<String, BudgetControl>>>,
    
    /// Rate limits by API key
    rate_limits: Arc<Mutex<HashMap<String, RateLimit>>>,
    
    /// Request logs
    request_logs: Arc<Mutex<Vec<RequestLog>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    /// Request metadata
    pub metadata: RequestMetadata,
    
    /// Request details
    pub request: RequestDetails,
    
    /// Response details
    pub response: ResponseDetails,
    
    /// Performance metrics
    pub performance: RequestPerformance,
    
    /// Cost information
    pub cost: RequestCost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDetails {
    /// HTTP method
    pub method: String,
    
    /// Request path
    pub path: String,
    
    /// Model used
    pub model: String,
    
    /// Provider used
    pub provider: String,
    
    /// Request size in bytes
    pub size_bytes: u64,
    
    /// Input tokens
    pub input_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDetails {
    /// HTTP status code
    pub status_code: u16,
    
    /// Response size in bytes
    pub size_bytes: u64,
    
    /// Output tokens
    pub output_tokens: u32,
    
    /// Success/failure
    pub success: bool,
    
    /// Error message if any
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPerformance {
    /// Total request time in milliseconds
    pub total_time_ms: f64,
    
    /// Routing time in microseconds
    pub routing_time_us: u64,
    
    /// Provider response time in milliseconds
    pub provider_time_ms: f64,
    
    /// Queue time in milliseconds
    pub queue_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestCost {
    /// Input cost in USD
    pub input_cost_usd: f64,
    
    /// Output cost in USD
    pub output_cost_usd: f64,
    
    /// Total cost in USD
    pub total_cost_usd: f64,
    
    /// Cost per token
    pub cost_per_token: f64,
}

impl EnterpriseManager {
    pub fn new(config: EnterpriseConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(EnterpriseMetrics::default())),
            budget_controls: Arc::new(Mutex::new(HashMap::new())),
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
            request_logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Check if request is allowed (budget and rate limits)
    pub fn is_request_allowed(&self, api_key: &str) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Check budget limits
        if self.config.governance.budget_controls_enabled {
            let budget_controls = self.budget_controls.lock().unwrap();
            if let Some(budget) = budget_controls.get(api_key) {
                if budget.is_over_budget() {
                    return Err("Budget limit exceeded".to_string());
                }
            }
        }
        
        // Check rate limits
        if self.config.governance.rate_limiting_enabled {
            let mut rate_limits = self.rate_limits.lock().unwrap();
            if let Some(rate_limit) = rate_limits.get_mut(api_key) {
                if rate_limit.is_rate_limited() {
                    return Err("Rate limit exceeded".to_string());
                }
            }
        }
        
        Ok(())
    }
    
    /// Log a request
    pub fn log_request(&self, log: RequestLog) {
        if !self.config.observability.logging_enabled {
            return;
        }
        
        let mut logs = self.request_logs.lock().unwrap();
        logs.push(log);
        
        // Keep only recent logs (last 10,000)
        if logs.len() > 10_000 {
            logs.drain(0..1_000);
        }
    }
    
    /// Update metrics
    pub fn update_metrics(&self, log: &RequestLog) {
        if !self.config.observability.metrics_enabled {
            return;
        }
        
        let mut metrics = self.metrics.lock().unwrap();
        
        // Update request metrics
        metrics.requests.total += 1;
        if log.response.success {
            metrics.requests.successful += 1;
        } else {
            metrics.requests.failed += 1;
        }
        
        // Update cost metrics
        metrics.costs.total_usd += log.cost.total_cost_usd;
        *metrics.costs.by_model.entry(log.request.model.clone()).or_insert(0.0) += log.cost.total_cost_usd;
        
        // Update performance metrics
        // This is simplified - in practice you'd maintain running averages
        metrics.performance.avg_response_time_ms = log.performance.total_time_ms;
        metrics.performance.avg_routing_time_us = log.performance.routing_time_us as f64;
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> EnterpriseMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    /// Add budget control for API key
    pub fn add_budget_control(&self, api_key: String, limit_usd: f64) {
        let mut budget_controls = self.budget_controls.lock().unwrap();
        budget_controls.insert(api_key.clone(), BudgetControl::new(api_key, limit_usd));
    }
    
    /// Add rate limit for API key
    pub fn add_rate_limit(&self, api_key: String, requests_per_minute: u32) {
        let mut rate_limits = self.rate_limits.lock().unwrap();
        rate_limits.insert(api_key.clone(), RateLimit::new(api_key, requests_per_minute));
    }
}

impl Default for EnterpriseMetrics {
    fn default() -> Self {
        Self {
            requests: RequestMetrics {
                total: 0,
                successful: 0,
                failed: 0,
                rps: 0.0,
                by_model: HashMap::new(),
                by_user: HashMap::new(),
                by_team: HashMap::new(),
            },
            costs: CostMetrics {
                total_usd: 0.0,
                by_model: HashMap::new(),
                by_user: HashMap::new(),
                by_team: HashMap::new(),
                tokens: TokenMetrics {
                    input_tokens: 0,
                    output_tokens: 0,
                    total_tokens: 0,
                    avg_tokens_per_request: 0.0,
                },
            },
            performance: PerformanceMetrics {
                avg_response_time_ms: 0.0,
                p50_response_time_ms: 0.0,
                p95_response_time_ms: 0.0,
                p99_response_time_ms: 0.0,
                avg_routing_time_us: 0.0,
                throughput_rps: 0.0,
            },
            errors: ErrorMetrics {
                total: 0,
                by_status_code: HashMap::new(),
                by_provider: HashMap::new(),
                error_rate: 0.0,
            },
            providers: HashMap::new(),
        }
    }
}