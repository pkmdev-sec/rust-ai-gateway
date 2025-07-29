pub mod config;
pub mod router;
pub mod strategy;
pub mod error;
pub mod retry;
pub mod guardrails;
pub mod multimodal;
pub mod enterprise;
pub mod models;
pub mod intelligent_router;

pub fn greet() {
    println!("Hello from Pure Rust AI Gateway!");
}

#[cfg(feature = "simple")]
pub mod simple;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "nodejs")]
pub mod nodejs;

pub use config::*;
pub use router::*;
pub use strategy::*;
pub use error::*;
pub use retry::*;
pub use guardrails::*;
pub use multimodal::*;
pub use models::*;
pub use intelligent_router::*;

#[cfg(feature = "simple")]
pub use simple::*;

#[cfg(feature = "server")]
pub use server::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_single_strategy() {
        let config = RouterConfig {
            mode: StrategyMode::Single,
            targets: vec![
                Target {
                    name: "openai".to_string(),
                    provider: "openai".to_string(),
                    weight: None,
                    api_key: Some("sk-test".to_string()),
                    metadata: Default::default(),
                    retry_config: None,
                    guardrails: None,
                    model_capabilities: None,
                    request_timeout_ms: None,
                },
            ],
            strategy: None,
            global_retry_config: None,
            global_guardrails: None,
            request_timeout_ms: None,
            enable_caching: None,
            cache_ttl_seconds: None,
        };

        let router = Router::new(config);
        let context = RouterContext::default();
        
        let result = router.route(&context).unwrap();
        assert_eq!(result.provider, "openai");
    }

    #[test]
    fn test_load_balance_strategy() {
        let config = RouterConfig {
            mode: StrategyMode::LoadBalance,
            targets: vec![
                Target {
                    name: "openai".to_string(),
                    provider: "openai".to_string(),
                    weight: Some(70),
                    api_key: Some("sk-test1".to_string()),
                    metadata: Default::default(),
                    retry_config: None,
                    guardrails: None,
                    model_capabilities: None,
                    request_timeout_ms: None,
                },
                Target {
                    name: "anthropic".to_string(),
                    provider: "anthropic".to_string(),
                    weight: Some(30),
                    api_key: Some("sk-test2".to_string()),
                    metadata: Default::default(),
                    retry_config: None,
                    guardrails: None,
                    model_capabilities: None,
                    request_timeout_ms: None,
                },
            ],
            strategy: None,
            global_retry_config: None,
            global_guardrails: None,
            request_timeout_ms: None,
            enable_caching: None,
            cache_ttl_seconds: None,
        };

        let router = Router::new(config);
        let context = RouterContext::default();
        
        // Test multiple times to ensure both providers can be selected
        let mut openai_count = 0;
        let mut anthropic_count = 0;
        
        for _ in 0..100 {
            let result = router.route(&context).unwrap();
            match result.provider.as_str() {
                "openai" => openai_count += 1,
                "anthropic" => anthropic_count += 1,
                _ => panic!("Unexpected provider"),
            }
        }
        
        // With weights 70:30, we expect roughly 70% openai, 30% anthropic
        assert!(openai_count > anthropic_count);
    }

    #[test]
    fn test_conditional_routing() {
        let config = RouterConfig {
            mode: StrategyMode::Conditional,
            targets: vec![
                Target {
                    name: "fast_model".to_string(),
                    provider: "openai".to_string(),
                    weight: None,
                    api_key: Some("sk-test1".to_string()),
                    metadata: Default::default(),
                    retry_config: None,
                    guardrails: None,
                    model_capabilities: None,
                    request_timeout_ms: None,
                },
                Target {
                    name: "smart_model".to_string(),
                    provider: "anthropic".to_string(),
                    weight: None,
                    api_key: Some("sk-test2".to_string()),
                    metadata: Default::default(),
                    retry_config: None,
                    guardrails: None,
                    model_capabilities: None,
                    request_timeout_ms: None,
                },
            ],
            strategy: Some(Strategy {
                conditions: vec![
                    Condition {
                        query: json!({
                            "metadata.priority": { "$eq": "high" }
                        }),
                        then_target: "smart_model".to_string(),
                    },
                ],
                default_target: Some("fast_model".to_string()),
            }),
            global_retry_config: None,
            global_guardrails: None,
            request_timeout_ms: None,
            enable_caching: None,
            cache_ttl_seconds: None,
        };

        let router = Router::new(config);
        
        // Test high priority request
        let mut context = RouterContext::default();
        context.metadata.insert("priority".to_string(), "high".to_string());
        
        let result = router.route(&context).unwrap();
        assert_eq!(result.provider, "anthropic");
        assert_eq!(result.name, "smart_model");
        
        // Test default case
        context.metadata.insert("priority".to_string(), "low".to_string());
        let result = router.route(&context).unwrap();
        assert_eq!(result.provider, "openai");
        assert_eq!(result.name, "fast_model");
    }
}