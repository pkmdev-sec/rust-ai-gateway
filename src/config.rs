use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{RetryConfig, GuardrailConfig, ModelCapabilities};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrategyMode {
    Single,
    LoadBalance,
    Fallback,
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub provider: String,
    pub weight: Option<u32>,
    pub api_key: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    pub retry_config: Option<RetryConfig>,
    pub guardrails: Option<Vec<GuardrailConfig>>,
    pub model_capabilities: Option<ModelCapabilities>,
    pub request_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub query: serde_json::Value,
    #[serde(rename = "then")]
    pub then_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub conditions: Vec<Condition>,
    #[serde(rename = "default")]
    pub default_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub mode: StrategyMode,
    pub targets: Vec<Target>,
    pub strategy: Option<Strategy>,
    pub global_retry_config: Option<RetryConfig>,
    pub global_guardrails: Option<Vec<GuardrailConfig>>,
    pub request_timeout_ms: Option<u64>,
    pub enable_caching: Option<bool>,
    pub cache_ttl_seconds: Option<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct RouterContext {
    pub metadata: HashMap<String, String>,
    pub params: HashMap<String, serde_json::Value>,
}

impl RouterContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn with_param(mut self, key: String, value: serde_json::Value) -> Self {
        self.params.insert(key, value);
        self
    }
}