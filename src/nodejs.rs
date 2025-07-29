use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value;
use std::collections::HashMap;

use crate::simple::SimpleAIGateway;
use crate::{RouterConfig, Target, StrategyMode, RouterError};

#[napi]
pub struct NodeAIGateway {
    inner: SimpleAIGateway,
}

#[napi]
impl NodeAIGateway {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SimpleAIGateway::new(),
        }
    }

    #[napi]
    pub fn add_openai(&mut self, api_key: String) {
        self.inner.add_openai(&api_key);
    }

    #[napi]
    pub fn add_anthropic(&mut self, api_key: String) {
        self.inner.add_anthropic(&api_key);
    }

    #[napi]
    pub fn enable_load_balancing(&mut self) {
        self.inner.enable_load_balancing();
    }

    #[napi]
    pub fn route_request(&self, message: String, metadata: Option<String>) -> Result<String> {
        let metadata_map = if let Some(meta) = metadata {
            serde_json::from_str::<HashMap<String, String>>(&meta)
                .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid metadata JSON: {}", e)))?
        } else {
            HashMap::new()
        };

        let result = self.inner.route_request(&message, Some(metadata_map))
            .map_err(|e| Error::new(Status::GenericFailure, format!("Routing failed: {}", e)))?;

        serde_json::to_string(&result)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Serialization failed: {}", e)))
    }
}

#[napi]
pub fn create_config_from_json(json_config: String) -> Result<String> {
    let json_value: Value = serde_json::from_str(&json_config)
        .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid JSON config: {}", e)))?;
    
    let config = convert_json_to_rust_config(json_value)?;    
    serde_json::to_string(&config)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Serialization failed: {}", e)))
}

fn convert_json_to_rust_config(json_config: Value) -> Result<RouterConfig> {
    let mut config = RouterConfig {
        mode: StrategyMode::Single,
        targets: Vec::new(),
        strategy: None,
        global_retry_config: None,
        global_guardrails: None,
        request_timeout_ms: None,
        enable_caching: None,
        cache_ttl_seconds: None,
    };

    // Handle single provider case
    if let Some(provider_name) = json_config.get("provider").and_then(|p| p.as_str()) {
        let api_key = json_config.get("apiKey")
            .or_else(|| json_config.get("api_key"))
            .and_then(|k| k.as_str())
            .unwrap_or_default();

        let target = Target {
            name: provider_name.to_string(),
            provider: provider_name.to_string(),
            weight: None,
            api_key: Some(api_key.to_string()),
            metadata: std::collections::HashMap::new(),
            retry_config: None,
            guardrails: None,
            model_capabilities: None,
            request_timeout_ms: None,
        };

        config.targets = vec![target];
        return Ok(config);
    }

    // Handle targets case (multiple providers with strategies)
    if let Some(targets) = json_config.get("targets").and_then(|t| t.as_array()) {
        let mut rust_targets = Vec::new();

        for target in targets {
            if let Some(provider_name) = target.get("provider").and_then(|p| p.as_str()) {
                let api_key = target.get("apiKey")
                    .or_else(|| target.get("api_key"))
                    .and_then(|k| k.as_str())
                    .unwrap_or_default();

                let rust_target = Target {
                    name: target.get("name").and_then(|n| n.as_str()).unwrap_or(provider_name).to_string(),
                    provider: provider_name.to_string(),
                    weight: target.get("weight").and_then(|w| w.as_u64()).map(|w| w as u32),
                    api_key: Some(api_key.to_string()),
                    metadata: std::collections::HashMap::new(),
                    retry_config: None,
                    guardrails: None,
                    model_capabilities: None,
                    request_timeout_ms: None,
                };

                rust_targets.push(rust_target);
            }
        }

        config.targets = rust_targets;

        // Handle strategy
        if let Some(strategy) = json_config.get("strategy") {
            if let Some(mode) = strategy.get("mode").and_then(|m| m.as_str()) {
                config.mode = match mode {
                    "loadbalance" => StrategyMode::LoadBalance,
                    "fallback" => StrategyMode::Fallback,
                    "conditional" => StrategyMode::Conditional,
                    _ => StrategyMode::Single,
                };
            }
        }
    }

    Ok(config)
}

#[napi]
pub fn route_with_json_config(json_config: String, message: String, metadata: Option<String>) -> Result<String> {
    let config_str = create_config_from_json(json_config)?;
    let config: RouterConfig = serde_json::from_str(&config_str)
        .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid config: {}", e)))?;

    let mut gateway = SimpleAIGateway::new();
    // Configure gateway based on config
    // This is a simplified version - in practice you'd need to set up the gateway properly
    
    let metadata_map = if let Some(meta) = metadata {
        serde_json::from_str::<HashMap<String, String>>(&meta)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid metadata JSON: {}", e)))?
    } else {
        HashMap::new()
    };

    let result = gateway.route_request(&message, Some(metadata_map))
        .map_err(|e| Error::new(Status::GenericFailure, format!("Routing failed: {}", e)))?;

    serde_json::to_string(&result)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Serialization failed: {}", e)))
}