use ai_gateway_router::{
    Condition, Router, RouterConfig, RouterContext, Strategy, StrategyMode, Target,
};
use serde_json::json;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("AI Gateway Router Examples\n");

    // Example 1: Single Provider
    println!("=== Single Provider Example ===");
    let single_config = RouterConfig {
        global_retry_config: None,
        global_guardrails: None,
        request_timeout_ms: None,
        enable_caching: None,
        cache_ttl_seconds: None,
        mode: StrategyMode::Single,
        targets: vec![Target {
            name: "openai-gpt4".to_string(),
            provider: "openai".to_string(),
            weight: None,
            api_key: Some("sk-your-openai-key".to_string()),
            metadata: HashMap::new(),
            retry_config: None,
            guardrails: None,
            model_capabilities: None,
            request_timeout_ms: None,
        }],
        strategy: None,
    };

    let router = Router::new(single_config);
    let context = RouterContext::new();
    let result = router.route(&context)?;
    println!("Selected: {} ({})", result.name, result.provider);

    // Example 2: Load Balancing
    println!("\n=== Load Balancing Example ===");
    let lb_config = RouterConfig {
        global_retry_config: None,
        global_guardrails: None,
        request_timeout_ms: None,
        enable_caching: None,
        cache_ttl_seconds: None,
        mode: StrategyMode::LoadBalance,
        targets: vec![
            Target {
                retry_config: None,
                guardrails: None,
                model_capabilities: None,
                request_timeout_ms: None,
                name: "openai-gpt4".to_string(),
                provider: "openai".to_string(),
                weight: Some(70), // 70% traffic
                api_key: Some("sk-openai-key".to_string()),
                metadata: HashMap::new(),
            },
            Target {
                retry_config: None,
                guardrails: None,
                model_capabilities: None,
                request_timeout_ms: None,
                name: "anthropic-claude".to_string(),
                provider: "anthropic".to_string(),
                weight: Some(30), // 30% traffic
                api_key: Some("sk-anthropic-key".to_string()),
                metadata: HashMap::new(),
            },
        ],
        strategy: None,
    };

    let router = Router::new(lb_config);
    let mut openai_count = 0;
    let mut anthropic_count = 0;

    for _ in 0..10 {
        let result = router.route(&context)?;
        match result.provider.as_str() {
            "openai" => openai_count += 1,
            "anthropic" => anthropic_count += 1,
            _ => {}
        }
        println!("Selected: {} ({})", result.name, result.provider);
    }
    println!(
        "Distribution: OpenAI: {}, Anthropic: {}",
        openai_count, anthropic_count
    );

    // Example 3: Conditional Routing
    println!("\n=== Conditional Routing Example ===");
    let conditional_config = RouterConfig {
        global_retry_config: None,
        global_guardrails: None,
        request_timeout_ms: None,
        enable_caching: None,
        cache_ttl_seconds: None,
        mode: StrategyMode::Conditional,
        targets: vec![
            Target {
                retry_config: None,
                guardrails: None,
                model_capabilities: None,
                request_timeout_ms: None,
                name: "fast-model".to_string(),
                provider: "openai".to_string(),
                weight: None,
                api_key: Some("sk-openai-key".to_string()),
                metadata: HashMap::new(),
            },
            Target {
                retry_config: None,
                guardrails: None,
                model_capabilities: None,
                request_timeout_ms: None,
                name: "smart-model".to_string(),
                provider: "anthropic".to_string(),
                weight: None,
                api_key: Some("sk-anthropic-key".to_string()),
                metadata: HashMap::new(),
            },
            Target {
                retry_config: None,
                guardrails: None,
                model_capabilities: None,
                request_timeout_ms: None,
                name: "coding-model".to_string(),
                provider: "openai".to_string(),
                weight: None,
                api_key: Some("sk-openai-key".to_string()),
                metadata: HashMap::new(),
            },
        ],
        strategy: Some(Strategy {
            conditions: vec![
                // Route to smart model for high priority requests
                Condition {
                    query: json!({
                        "metadata.priority": { "$eq": "high" }
                    }),
                    then_target: "smart-model".to_string(),
                },
                // Route to coding model for code-related requests
                Condition {
                    query: json!({
                        "metadata.task_type": { "$eq": "coding" }
                    }),
                    then_target: "coding-model".to_string(),
                },
                // Route to smart model for complex queries (token count > 1000)
                Condition {
                    query: json!({
                        "params.token_count": { "$gt": 1000 }
                    }),
                    then_target: "smart-model".to_string(),
                },
            ],
            default_target: Some("fast-model".to_string()),
        }),
    };

    let router = Router::new(conditional_config);

    // Test different scenarios
    let scenarios = vec![
        (
            "High priority request",
            RouterContext::new().with_metadata("priority".to_string(), "high".to_string()),
        ),
        (
            "Coding request",
            RouterContext::new().with_metadata("task_type".to_string(), "coding".to_string()),
        ),
        (
            "Complex request",
            RouterContext::new().with_param("token_count".to_string(), json!(1500)),
        ),
        (
            "Default request",
            RouterContext::new().with_metadata("priority".to_string(), "low".to_string()),
        ),
    ];

    for (description, context) in scenarios {
        let result = router.route(&context)?;
        println!("{}: {} ({})", description, result.name, result.provider);
    }

    // Example 4: Complex Conditional Logic
    println!("\n=== Complex Conditional Logic Example ===");
    let complex_config = RouterConfig {
        global_retry_config: None,
        global_guardrails: None,
        request_timeout_ms: None,
        enable_caching: None,
        cache_ttl_seconds: None,
        mode: StrategyMode::Conditional,
        targets: vec![
            Target {
                retry_config: None,
                guardrails: None,
                model_capabilities: None,
                request_timeout_ms: None,
                name: "premium-model".to_string(),
                provider: "anthropic".to_string(),
                weight: None,
                api_key: Some("sk-anthropic-key".to_string()),
                metadata: HashMap::new(),
            },
            Target {
                retry_config: None,
                guardrails: None,
                model_capabilities: None,
                request_timeout_ms: None,
                name: "standard-model".to_string(),
                provider: "openai".to_string(),
                weight: None,
                api_key: Some("sk-openai-key".to_string()),
                metadata: HashMap::new(),
            },
        ],
        strategy: Some(Strategy {
            conditions: vec![
                // Complex condition: premium users OR high priority requests with large token counts
                Condition {
                    query: json!({
                        "$or": [
                            { "metadata.user_tier": { "$eq": "premium" } },
                            {
                                "$and": [
                                    { "metadata.priority": { "$eq": "high" } },
                                    { "params.token_count": { "$gt": 500 } }
                                ]
                            }
                        ]
                    }),
                    then_target: "premium-model".to_string(),
                },
            ],
            default_target: Some("standard-model".to_string()),
        }),
    };

    let router = Router::new(complex_config);

    let complex_scenarios = vec![
        (
            "Premium user",
            RouterContext::new().with_metadata("user_tier".to_string(), "premium".to_string()),
        ),
        (
            "High priority + large tokens",
            RouterContext::new()
                .with_metadata("priority".to_string(), "high".to_string())
                .with_param("token_count".to_string(), json!(800)),
        ),
        (
            "High priority + small tokens",
            RouterContext::new()
                .with_metadata("priority".to_string(), "high".to_string())
                .with_param("token_count".to_string(), json!(200)),
        ),
        (
            "Regular user",
            RouterContext::new().with_metadata("user_tier".to_string(), "free".to_string()),
        ),
    ];

    for (description, context) in complex_scenarios {
        let result = router.route(&context)?;
        println!("{}: {} ({})", description, result.name, result.provider);
    }

    Ok(())
}
