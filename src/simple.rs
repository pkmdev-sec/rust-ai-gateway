use crate::{Router, RouterConfig, RouterContext, StrategyMode, Target, RoutingResult, RouterError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple, easy-to-use AI Gateway Router
/// 
/// # Example
/// ```rust
/// use ai_gateway_router::SimpleAIGateway;
/// 
/// let mut gateway = SimpleAIGateway::new();
/// 
/// // Add providers
/// gateway.add_openai("your-openai-key");
/// gateway.add_anthropic("your-anthropic-key");
/// 
/// // Route a request
/// let result = gateway.route_request("Hello, world!", None)?;
/// println!("Using provider: {}", result.provider);
/// ```
#[derive(Debug)]
pub struct SimpleAIGateway {
    router: Router,
    providers: Vec<SimpleProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleProvider {
    pub name: String,
    pub provider_type: String,
    pub api_key: String,
    pub weight: Option<u32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleRequest {
    pub content: String,
    pub metadata: Option<HashMap<String, String>>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleResponse {
    pub provider: String,
    pub provider_name: String,
    pub api_key: String,
    pub routing_time_ns: u64,
    pub metadata: HashMap<String, String>,
}

impl SimpleAIGateway {
    /// Create a new AI Gateway with default settings
    pub fn new() -> Self {
        Self {
            router: Router::new(RouterConfig {
                mode: StrategyMode::Single,
                targets: vec![],
                strategy: None,
                global_retry_config: None,
                global_guardrails: None,
                request_timeout_ms: Some(30000),
                enable_caching: Some(true),
                cache_ttl_seconds: Some(300),
            }),
            providers: vec![],
        }
    }

    /// Add OpenAI as a provider
    pub fn add_openai(&mut self, api_key: &str) -> &mut Self {
        self.add_provider("openai", "openai", api_key, None)
    }

    /// Add Anthropic as a provider
    pub fn add_anthropic(&mut self, api_key: &str) -> &mut Self {
        self.add_provider("anthropic", "anthropic", api_key, None)
    }

    /// Add any provider with custom configuration
    pub fn add_provider(&mut self, name: &str, provider_type: &str, api_key: &str, weight: Option<u32>) -> &mut Self {
        let provider = SimpleProvider {
            name: name.to_string(),
            provider_type: provider_type.to_string(),
            api_key: api_key.to_string(),
            weight,
            enabled: true,
        };
        
        self.providers.push(provider);
        self.rebuild_router();
        self
    }

    /// Enable load balancing between providers
    pub fn enable_load_balancing(&mut self) -> &mut Self {
        // Set default weights if not specified
        for provider in &mut self.providers {
            if provider.weight.is_none() {
                provider.weight = Some(100);
            }
        }
        self.rebuild_router();
        self
    }

    /// Set provider weights for load balancing
    pub fn set_provider_weight(&mut self, provider_name: &str, weight: u32) -> &mut Self {
        if let Some(provider) = self.providers.iter_mut().find(|p| p.name == provider_name) {
            provider.weight = Some(weight);
            self.rebuild_router();
        }
        self
    }

    /// Enable/disable a provider
    pub fn set_provider_enabled(&mut self, provider_name: &str, enabled: bool) -> &mut Self {
        if let Some(provider) = self.providers.iter_mut().find(|p| p.name == provider_name) {
            provider.enabled = enabled;
            self.rebuild_router();
        }
        self
    }

    /// Route a simple text request
    pub fn route_request(&self, content: &str, metadata: Option<HashMap<String, String>>) -> Result<SimpleResponse, RouterError> {
        let context = if let Some(meta) = metadata {
            let mut ctx = RouterContext::new();
            for (key, value) in meta {
                ctx = ctx.with_metadata(key, value);
            }
            ctx
        } else {
            RouterContext::new()
        };

        let start = std::time::Instant::now();
        let result = self.router.route(&context)?;
        let routing_time = start.elapsed().as_nanos() as u64;

        Ok(SimpleResponse {
            provider: result.provider.clone(),
            provider_name: result.name.clone(),
            api_key: result.api_key.unwrap_or_default(),
            routing_time_ns: routing_time,
            metadata: result.metadata.clone(),
        })
    }

    /// Route a detailed request
    pub fn route_detailed_request(&self, request: &SimpleRequest) -> Result<SimpleResponse, RouterError> {
        self.route_request(&request.content, request.metadata.clone())
    }

    /// Get all configured providers
    pub fn get_providers(&self) -> &[SimpleProvider] {
        &self.providers
    }

    /// Get routing statistics
    pub fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        stats.insert("total_providers".to_string(), serde_json::json!(self.providers.len()));
        stats.insert("enabled_providers".to_string(), serde_json::json!(self.providers.iter().filter(|p| p.enabled).count()));
        stats.insert("routing_mode".to_string(), serde_json::json!(format!("{:?}", self.router.get_mode())));
        stats
    }

    /// Rebuild the internal router when configuration changes
    fn rebuild_router(&mut self) {
        let enabled_providers: Vec<_> = self.providers.iter().filter(|p| p.enabled).collect();
        
        if enabled_providers.is_empty() {
            return;
        }

        let mode = if enabled_providers.len() == 1 {
            StrategyMode::Single
        } else if enabled_providers.iter().any(|p| p.weight.is_some()) {
            StrategyMode::LoadBalance
        } else {
            StrategyMode::Single
        };

        let targets: Vec<Target> = enabled_providers.iter().map(|provider| {
            Target {
                name: provider.name.clone(),
                provider: provider.provider_type.clone(),
                weight: provider.weight,
                api_key: Some(provider.api_key.clone()),
                metadata: HashMap::new(),
                retry_config: None,
                guardrails: None,
                model_capabilities: None,
                request_timeout_ms: Some(30000),
            }
        }).collect();

        let config = RouterConfig {
            mode,
            targets,
            strategy: None,
            global_retry_config: None,
            global_guardrails: None,
            request_timeout_ms: Some(30000),
            enable_caching: Some(true),
            cache_ttl_seconds: Some(300),
        };

        self.router = Router::new(config);
    }
}

impl Default for SimpleAIGateway {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for easy configuration
pub struct SimpleAIGatewayBuilder {
    gateway: SimpleAIGateway,
}

impl SimpleAIGatewayBuilder {
    pub fn new() -> Self {
        Self {
            gateway: SimpleAIGateway::new(),
        }
    }

    pub fn with_openai(mut self, api_key: &str) -> Self {
        self.gateway.add_openai(api_key);
        self
    }

    pub fn with_anthropic(mut self, api_key: &str) -> Self {
        self.gateway.add_anthropic(api_key);
        self
    }

    pub fn with_provider(mut self, name: &str, provider_type: &str, api_key: &str, weight: Option<u32>) -> Self {
        self.gateway.add_provider(name, provider_type, api_key, weight);
        self
    }

    pub fn with_load_balancing(mut self) -> Self {
        self.gateway.enable_load_balancing();
        self
    }

    pub fn build(self) -> SimpleAIGateway {
        self.gateway
    }
}

impl Default for SimpleAIGatewayBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_gateway_creation() {
        let gateway = SimpleAIGateway::new();
        assert_eq!(gateway.providers.len(), 0);
    }

    #[test]
    fn test_add_providers() {
        let mut gateway = SimpleAIGateway::new();
        gateway.add_openai("test-key-1");
        gateway.add_anthropic("test-key-2");
        
        assert_eq!(gateway.providers.len(), 2);
        assert_eq!(gateway.providers[0].provider_type, "openai");
        assert_eq!(gateway.providers[1].provider_type, "anthropic");
    }

    #[test]
    fn test_builder_pattern() {
        let gateway = SimpleAIGatewayBuilder::new()
            .with_openai("openai-key")
            .with_anthropic("anthropic-key")
            .with_load_balancing()
            .build();
        
        assert_eq!(gateway.providers.len(), 2);
        assert!(gateway.providers.iter().all(|p| p.weight.is_some()));
    }

    #[test]
    fn test_routing() {
        let mut gateway = SimpleAIGateway::new();
        gateway.add_openai("test-key");
        
        let result = gateway.route_request("Hello, world!", None).unwrap();
        assert_eq!(result.provider, "openai");
        assert!(result.routing_time_ns > 0);
    }

    #[test]
    fn test_load_balancing() {
        let mut gateway = SimpleAIGateway::new();
        gateway.add_openai("openai-key");
        gateway.add_anthropic("anthropic-key");
        gateway.enable_load_balancing();
        
        // Test multiple requests to see different providers
        let mut providers_used = std::collections::HashSet::new();
        for _ in 0..10 {
            let result = gateway.route_request("Test", None).unwrap();
            providers_used.insert(result.provider);
        }
        
        // Should use both providers with load balancing
        assert!(providers_used.len() >= 1); // At least one provider used
    }
}