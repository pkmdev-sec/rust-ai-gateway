use ai_gateway_router::{
    Router, RouterConfig, RouterContext, StrategyMode, Target, Strategy, Condition,
    RetryConfig, ExponentialBackoffConfig, GuardrailConfig, GuardrailType, GuardrailAction,
    GuardrailEngine, ContentModerationGuardrail, PIIDetectionGuardrail, TokenLimitGuardrail,
    ModelCapabilities, ModalityType, ImageFormat, MultiModalProcessor, ProcessingOptions,
};
use serde_json::json;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced AI Gateway Router Examples\n");

    // Get API keys from environment
    let openai_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-test-openai".to_string());
    let anthropic_key = env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| "sk-test-anthropic".to_string());

    // Example 1: Enterprise Router with Retries and Guardrails
    println!("=== Enterprise Router with Retries & Guardrails ===");
    
    let enterprise_config = RouterConfig {
        mode: StrategyMode::Conditional,
        targets: vec![
            Target {
                name: "production-gpt4".to_string(),
                provider: "openai".to_string(),
                weight: None,
                api_key: Some(openai_key.clone()),
                metadata: HashMap::new(),
                retry_config: Some(RetryConfig {
                    attempts: 3,
                    on_status_codes: vec![429, 500, 502, 503, 504],
                    use_retry_after_header: Some(true),
                    exponential_backoff: Some(ExponentialBackoffConfig {
                        initial_delay_ms: 100,
                        max_delay_ms: 10000,
                        multiplier: 2.0,
                        jitter: true,
                    }),
                }),
                guardrails: Some(vec![
                    GuardrailConfig {
                        name: "content_filter".to_string(),
                        enabled: true,
                        guardrail_type: GuardrailType::Both,
                        action: GuardrailAction::Block,
                        settings: {
                            let mut map = HashMap::new();
                            map.insert("blocked_patterns".to_string(), json!(["harmful", "inappropriate"]));
                            map
                        },
                    },
                    GuardrailConfig {
                        name: "pii_protection".to_string(),
                        enabled: true,
                        guardrail_type: GuardrailType::Both,
                        action: GuardrailAction::Modify,
                        settings: HashMap::new(),
                    },
                ]),
                model_capabilities: Some(ModelCapabilities {
                    supported_modalities: vec![ModalityType::Text, ModalityType::Image],
                    max_image_size: Some(20 * 1024 * 1024), // 20MB
                    ..Default::default()
                }),
                request_timeout_ms: Some(30000), // 30 seconds
            },
            Target {
                name: "fallback-claude".to_string(),
                provider: "anthropic".to_string(),
                weight: None,
                api_key: Some(anthropic_key.clone()),
                metadata: HashMap::new(),
                retry_config: Some(RetryConfig::default()),
                guardrails: None,
                model_capabilities: Some(ModelCapabilities {
                    supported_modalities: vec![ModalityType::Text],
                    ..Default::default()
                }),
                request_timeout_ms: Some(30000),
            },
        ],
        strategy: Some(Strategy {
            conditions: vec![
                // Route high-priority requests to GPT-4
                Condition {
                    query: json!({
                        "metadata.priority": { "$eq": "high" }
                    }),
                    then_target: "production-gpt4".to_string(),
                },
                // Route multimodal requests to GPT-4
                Condition {
                    query: json!({
                        "metadata.has_images": { "$eq": "true" }
                    }),
                    then_target: "production-gpt4".to_string(),
                },
                // Route large token requests to Claude
                Condition {
                    query: json!({
                        "params.estimated_tokens": { "$gt": 8000 }
                    }),
                    then_target: "fallback-claude".to_string(),
                },
            ],
            default_target: Some("production-gpt4".to_string()),
        }),
        global_retry_config: Some(RetryConfig::default()),
        global_guardrails: Some(vec![
            GuardrailConfig {
                name: "global_token_limit".to_string(),
                enabled: true,
                guardrail_type: GuardrailType::Input,
                action: GuardrailAction::Block,
                settings: {
                    let mut map = HashMap::new();
                    map.insert("max_tokens".to_string(), json!(32000));
                    map
                },
            },
        ]),
        request_timeout_ms: Some(60000), // 1 minute global timeout
        enable_caching: Some(true),
        cache_ttl_seconds: Some(300), // 5 minutes
    };

    let router = Router::new(enterprise_config);

    // Test different routing scenarios
    let scenarios = vec![
        ("High priority text request", RouterContext::new()
            .with_metadata("priority".to_string(), "high".to_string())
            .with_param("estimated_tokens".to_string(), json!(1000))),
        
        ("Multimodal request", RouterContext::new()
            .with_metadata("has_images".to_string(), "true".to_string())
            .with_param("estimated_tokens".to_string(), json!(2000))),
        
        ("Large context request", RouterContext::new()
            .with_metadata("priority".to_string(), "normal".to_string())
            .with_param("estimated_tokens".to_string(), json!(15000))),
        
        ("Standard request", RouterContext::new()
            .with_metadata("priority".to_string(), "normal".to_string())
            .with_param("estimated_tokens".to_string(), json!(500))),
    ];

    for (description, context) in scenarios {
        let result = router.route(&context)?;
        println!("📍 {}: {} ({})", description, result.name, result.provider);
    }

    // Example 2: Guardrails Engine Demo
    println!("\n=== Guardrails Engine Demo ===");
    
    let mut guardrail_engine = GuardrailEngine::new();
    
    // Add content moderation
    let content_config = GuardrailConfig {
        name: "content_moderation".to_string(),
        enabled: true,
        guardrail_type: GuardrailType::Both,
        action: GuardrailAction::Block,
        settings: {
            let mut map = HashMap::new();
            map.insert("blocked_patterns".to_string(), json!(["badword", "spam", "harmful"]));
            map
        },
    };
    guardrail_engine.add_guardrail(Box::new(ContentModerationGuardrail::new(content_config)?));

    // Add PII detection
    let pii_config = GuardrailConfig {
        name: "pii_detection".to_string(),
        enabled: true,
        guardrail_type: GuardrailType::Both,
        action: GuardrailAction::Modify,
        settings: HashMap::new(),
    };
    guardrail_engine.add_guardrail(Box::new(PIIDetectionGuardrail::new(pii_config)?));

    // Add token limit
    let token_config = GuardrailConfig {
        name: "token_limit".to_string(),
        enabled: true,
        guardrail_type: GuardrailType::Input,
        action: GuardrailAction::Block,
        settings: {
            let mut map = HashMap::new();
            map.insert("max_tokens".to_string(), json!(1000));
            map
        },
    };
    guardrail_engine.add_guardrail(Box::new(TokenLimitGuardrail::new(token_config)?));

    // Test guardrails
    let test_inputs = vec![
        "Hello, this is a clean message",
        "This contains a badword that should be blocked",
        "Contact me at john.doe@example.com or call 555-123-4567",
        "This is a very long message that should exceed the token limit because it contains many words and characters that will be counted as tokens by the estimation algorithm",
    ];

    for input in test_inputs {
        println!("\n🔍 Testing: \"{}\"", input);
        let results = guardrail_engine.check_input(input, &HashMap::new())?;
        
        for result in &results {
            if !result.passed {
                println!("  ❌ Failed: {} - {}", 
                    match result.action {
                        GuardrailAction::Block => "BLOCKED",
                        GuardrailAction::Modify => "MODIFIED",
                        GuardrailAction::Warn => "WARNING",
                        GuardrailAction::Log => "LOGGED",
                    },
                    result.message.as_ref().unwrap_or(&"No message".to_string())
                );
            } else {
                println!("  ✅ Passed guardrail check");
            }
        }

        if guardrail_engine.should_block(&results) {
            println!("  🚫 Request would be BLOCKED");
        } else {
            let modified = guardrail_engine.get_modified_content(&results, input);
            if modified != input {
                println!("  ✏️  Modified content: \"{}\"", modified);
            }
        }
    }

    // Example 3: Multi-Modal Processing
    println!("\n=== Multi-Modal Processing Demo ===");
    
    let multimodal_capabilities = ModelCapabilities {
        supported_modalities: vec![
            ModalityType::Text,
            ModalityType::Image,
            ModalityType::Audio,
        ],
        max_image_size: Some(10 * 1024 * 1024), // 10MB
        max_audio_duration: Some(5 * 60 * 1000), // 5 minutes
        supported_image_formats: vec![ImageFormat::Jpeg, ImageFormat::Png],
        ..Default::default()
    };

    let processing_options = ProcessingOptions {
        ocr_enabled: true,
        speech_to_text: true,
        ..Default::default()
    };

    let processor = MultiModalProcessor::new(multimodal_capabilities, processing_options);

    // Simulate different content types for routing hints
    let multimodal_scenarios = vec![
        ("Text-only request", vec![ModalityType::Text]),
        ("Image analysis request", vec![ModalityType::Text, ModalityType::Image]),
        ("Audio transcription", vec![ModalityType::Audio]),
        ("Complex multimodal", vec![ModalityType::Text, ModalityType::Image, ModalityType::Audio]),
    ];

    for (description, modalities) in multimodal_scenarios {
        // Create a mock request for demonstration
        let request = create_mock_multimodal_request(modalities);
        let hints = processor.get_model_routing_hints(&request);
        let estimated_time = processor.estimate_processing_time(&request);
        
        println!("🎭 {}", description);
        println!("   Primary modality: {}", hints.get("primary_modality").unwrap_or(&"Unknown".to_string()));
        println!("   Complexity: {}", hints.get("complexity").unwrap_or(&"low".to_string()));
        println!("   Estimated tokens: {}", hints.get("estimated_tokens").unwrap_or(&"0".to_string()));
        println!("   Processing time: {}ms", estimated_time);
        
        if let Some(preferred) = hints.get("preferred_provider") {
            println!("   Preferred provider: {}", preferred);
        }
    }

    // Example 4: Performance Optimized Router
    println!("\n=== Performance Optimized Router (<1ms target) ===");
    
    let fast_config = RouterConfig {
        mode: StrategyMode::LoadBalance,
        targets: vec![
            Target {
                name: "fast-gpt-3.5".to_string(),
                provider: "openai".to_string(),
                weight: Some(80),
                api_key: Some(openai_key),
                metadata: HashMap::new(),
                retry_config: Some(RetryConfig {
                    attempts: 1, // Minimal retries for speed
                    on_status_codes: vec![429], // Only retry rate limits
                    use_retry_after_header: Some(false),
                    exponential_backoff: Some(ExponentialBackoffConfig {
                        initial_delay_ms: 50, // Very fast backoff
                        max_delay_ms: 200,
                        multiplier: 1.5,
                        jitter: false, // No jitter for predictable timing
                    }),
                }),
                guardrails: None, // No guardrails for maximum speed
                model_capabilities: Some(ModelCapabilities {
                    supported_modalities: vec![ModalityType::Text],
                    ..Default::default()
                }),
                request_timeout_ms: Some(5000), // 5 second timeout
            },
            Target {
                name: "fast-claude-haiku".to_string(),
                provider: "anthropic".to_string(),
                weight: Some(20),
                api_key: Some(anthropic_key),
                metadata: HashMap::new(),
                retry_config: Some(RetryConfig {
                    attempts: 1,
                    on_status_codes: vec![429],
                    use_retry_after_header: Some(false),
                    exponential_backoff: Some(ExponentialBackoffConfig {
                        initial_delay_ms: 50,
                        max_delay_ms: 200,
                        multiplier: 1.5,
                        jitter: false,
                    }),
                }),
                guardrails: None,
                model_capabilities: Some(ModelCapabilities {
                    supported_modalities: vec![ModalityType::Text],
                    ..Default::default()
                }),
                request_timeout_ms: Some(5000),
            },
        ],
        strategy: None,
        global_retry_config: None,
        global_guardrails: None,
        request_timeout_ms: Some(5000),
        enable_caching: Some(true), // Enable caching for speed
        cache_ttl_seconds: Some(60), // Short TTL for fresh responses
    };

    let fast_router = Router::new(fast_config);
    let context = RouterContext::new();

    // Simulate routing performance test
    println!("🏃‍♂️ Testing routing performance (10 requests):");
    let start = std::time::Instant::now();
    
    for i in 1..=10 {
        let result = fast_router.route(&context)?;
        println!("   Request {}: {} ({})", i, result.name, result.provider);
    }
    
    let elapsed = start.elapsed();
    println!("⚡ Total time: {:?} ({:.2}ms per request)", elapsed, elapsed.as_millis() as f64 / 10.0);

    println!("\n🎉 Advanced examples completed!");
    Ok(())
}

// Helper function to create mock multimodal requests
fn create_mock_multimodal_request(modalities: Vec<ModalityType>) -> ai_gateway_router::MultiModalRequest {
    use ai_gateway_router::{MultiModalRequest, MediaContent, MediaData};
    
    let contents = modalities.into_iter().map(|modality| {
        let data = match modality {
            ModalityType::Text => MediaData::Text { content: "Sample text".to_string() },
            ModalityType::Image => MediaData::Image {
                format: ImageFormat::Jpeg,
                data: vec![0; 1000], // 1KB mock image
                width: Some(512),
                height: Some(512),
            },
            ModalityType::Audio => MediaData::Audio {
                format: ai_gateway_router::AudioFormat::Mp3,
                data: vec![0; 5000], // 5KB mock audio
                duration_ms: Some(10000), // 10 seconds
                sample_rate: Some(44100),
            },
            _ => MediaData::Text { content: "Mock content".to_string() },
        };

        MediaContent {
            modality,
            data,
            metadata: HashMap::new(),
        }
    }).collect();

    MultiModalRequest {
        contents,
        model_capabilities: ModelCapabilities::default(),
        processing_options: ProcessingOptions::default(),
    }
}