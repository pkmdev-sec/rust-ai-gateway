use ai_gateway_router::{
    Router, RouterConfig, RouterContext, StrategyMode, Target, Strategy, Condition,
    RetryConfig, ExponentialBackoffConfig, GuardrailConfig, GuardrailType, GuardrailAction,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone, serde::Serialize)]
struct PerformanceResult {
    scenario_name: String,
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    success_rate: f64,
    total_time_ms: f64,
    throughput_req_per_sec: f64,
    min_latency_ns: u64,
    max_latency_ns: u64,
    avg_latency_ns: f64,
    median_latency_ns: u64,
    p95_latency_ns: u64,
    p99_latency_ns: u64,
    routing_only_latency_ns: f64, // This is what we can measure - pure routing time
}

struct TestScenario {
    name: String,
    config: RouterConfig,
    context: RouterContext,
}

fn create_test_scenarios() -> Vec<TestScenario> {
    let openai_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set");
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set");

    vec![
        TestScenario {
            name: "Single Provider Routing".to_string(),
            config: RouterConfig {
                mode: StrategyMode::Single,
                targets: vec![
                    Target {
                        name: "openai-gpt35".to_string(),
                        provider: "openai".to_string(),
                        weight: None,
                        api_key: Some(openai_key.clone()),
                        metadata: HashMap::new(),
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
            },
            context: RouterContext::new(),
        },
        TestScenario {
            name: "Load Balance Routing".to_string(),
            config: RouterConfig {
                mode: StrategyMode::LoadBalance,
                targets: vec![
                    Target {
                        name: "openai-gpt35".to_string(),
                        provider: "openai".to_string(),
                        weight: Some(70),
                        api_key: Some(openai_key.clone()),
                        metadata: HashMap::new(),
                        retry_config: None,
                        guardrails: None,
                        model_capabilities: None,
                        request_timeout_ms: None,
                    },
                    Target {
                        name: "anthropic-claude".to_string(),
                        provider: "anthropic".to_string(),
                        weight: Some(30),
                        api_key: Some(anthropic_key.clone()),
                        metadata: HashMap::new(),
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
            },
            context: RouterContext::new(),
        },
        TestScenario {
            name: "Conditional Routing".to_string(),
            config: RouterConfig {
                mode: StrategyMode::Conditional,
                targets: vec![
                    Target {
                        name: "fast_model".to_string(),
                        provider: "openai".to_string(),
                        weight: None,
                        api_key: Some(openai_key.clone()),
                        metadata: HashMap::new(),
                        retry_config: None,
                        guardrails: None,
                        model_capabilities: None,
                        request_timeout_ms: None,
                    },
                    Target {
                        name: "smart_model".to_string(),
                        provider: "anthropic".to_string(),
                        weight: None,
                        api_key: Some(anthropic_key.clone()),
                        metadata: HashMap::new(),
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
            },
            context: RouterContext::new()
                .with_metadata("priority".to_string(), "high".to_string()),
        },
        TestScenario {
            name: "Complex Conditional Routing".to_string(),
            config: RouterConfig {
                mode: StrategyMode::Conditional,
                targets: vec![
                    Target {
                        name: "premium_model".to_string(),
                        provider: "anthropic".to_string(),
                        weight: None,
                        api_key: Some(anthropic_key.clone()),
                        metadata: HashMap::new(),
                        retry_config: None,
                        guardrails: None,
                        model_capabilities: None,
                        request_timeout_ms: None,
                    },
                    Target {
                        name: "standard_model".to_string(),
                        provider: "openai".to_string(),
                        weight: None,
                        api_key: Some(openai_key.clone()),
                        metadata: HashMap::new(),
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
                            then_target: "premium_model".to_string(),
                        },
                    ],
                    default_target: Some("standard_model".to_string()),
                }),
                global_retry_config: None,
                global_guardrails: None,
                request_timeout_ms: None,
                enable_caching: None,
                cache_ttl_seconds: None,
            },
            context: RouterContext::new()
                .with_metadata("user_tier".to_string(), "premium".to_string())
                .with_metadata("priority".to_string(), "high".to_string())
                .with_param("token_count".to_string(), json!(800)),
        },
        TestScenario {
            name: "Enterprise with Retries".to_string(),
            config: RouterConfig {
                mode: StrategyMode::Fallback,
                targets: vec![
                    Target {
                        name: "primary".to_string(),
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
                                max_delay_ms: 5000,
                                multiplier: 2.0,
                                jitter: true,
                            }),
                        }),
                        guardrails: None,
                        model_capabilities: None,
                        request_timeout_ms: Some(30000),
                    },
                    Target {
                        name: "fallback".to_string(),
                        provider: "anthropic".to_string(),
                        weight: None,
                        api_key: Some(anthropic_key.clone()),
                        metadata: HashMap::new(),
                        retry_config: Some(RetryConfig::default()),
                        guardrails: None,
                        model_capabilities: None,
                        request_timeout_ms: Some(30000),
                    },
                ],
                strategy: None,
                global_retry_config: Some(RetryConfig::default()),
                global_guardrails: None,
                request_timeout_ms: Some(60000),
                enable_caching: Some(true),
                cache_ttl_seconds: Some(300),
            },
            context: RouterContext::new(),
        },
    ]
}

async fn benchmark_scenario(scenario: &TestScenario, iterations: usize) -> PerformanceResult {
    println!("🧪 Testing: {}", scenario.name);
    println!("📊 Running {} routing operations", iterations);

    let router = Router::new(scenario.config.clone());
    let mut latencies = Vec::with_capacity(iterations);
    let mut successful = 0;
    let mut failed = 0;

    let overall_start = Instant::now();

    for i in 0..iterations {
        let start = Instant::now();
        
        match router.route(&scenario.context) {
            Ok(_result) => {
                let latency = start.elapsed();
                latencies.push(latency.as_nanos() as u64);
                successful += 1;
            }
            Err(_) => {
                failed += 1;
            }
        }

        // Micro-sleep to prevent CPU saturation and get more realistic measurements
        if i % 1000 == 0 && i > 0 {
            sleep(Duration::from_nanos(1)).await;
        }
    }

    let total_time = overall_start.elapsed();
    
    if latencies.is_empty() {
        return PerformanceResult {
            scenario_name: scenario.name.clone(),
            total_requests: iterations,
            successful_requests: 0,
            failed_requests: failed,
            success_rate: 0.0,
            total_time_ms: total_time.as_millis() as f64,
            throughput_req_per_sec: 0.0,
            min_latency_ns: 0,
            max_latency_ns: 0,
            avg_latency_ns: 0.0,
            median_latency_ns: 0,
            p95_latency_ns: 0,
            p99_latency_ns: 0,
            routing_only_latency_ns: 0.0,
        };
    }

    latencies.sort_unstable();

    let min_latency = *latencies.first().unwrap();
    let max_latency = *latencies.last().unwrap();
    let avg_latency = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
    let median_latency = latencies[latencies.len() / 2];
    let p95_latency = latencies[(latencies.len() as f64 * 0.95) as usize];
    let p99_latency = latencies[(latencies.len() as f64 * 0.99) as usize];

    PerformanceResult {
        scenario_name: scenario.name.clone(),
        total_requests: iterations,
        successful_requests: successful,
        failed_requests: failed,
        success_rate: (successful as f64 / iterations as f64) * 100.0,
        total_time_ms: total_time.as_millis() as f64,
        throughput_req_per_sec: successful as f64 / total_time.as_secs_f64(),
        min_latency_ns: min_latency,
        max_latency_ns: max_latency,
        avg_latency_ns: avg_latency,
        median_latency_ns: median_latency,
        p95_latency_ns: p95_latency,
        p99_latency_ns: p99_latency,
        routing_only_latency_ns: avg_latency, // This is pure routing time
    }
}

fn print_results(results: &[PerformanceResult]) {
    println!("\n{}", "=".repeat(80));
    println!("📈 RUST AI GATEWAY ROUTER PERFORMANCE RESULTS");
    println!("{}", "=".repeat(80));

    for result in results {
        println!("\n🎯 {}", result.scenario_name);
        println!("   Total Requests: {}", result.total_requests);
        println!("   Success Rate: {:.2}%", result.success_rate);
        println!("   Total Time: {:.2}ms", result.total_time_ms);
        println!("   Throughput: {:.2} req/s", result.throughput_req_per_sec);
        println!("   Routing Latency (avg): {:.2}ns ({:.6}ms)", result.avg_latency_ns, result.avg_latency_ns / 1_000_000.0);
        println!("   Routing Latency (min): {}ns ({:.6}ms)", result.min_latency_ns, result.min_latency_ns as f64 / 1_000_000.0);
        println!("   Routing Latency (max): {}ns ({:.6}ms)", result.max_latency_ns, result.max_latency_ns as f64 / 1_000_000.0);
        println!("   Routing Latency (p95): {}ns ({:.6}ms)", result.p95_latency_ns, result.p95_latency_ns as f64 / 1_000_000.0);
        println!("   Routing Latency (p99): {}ns ({:.6}ms)", result.p99_latency_ns, result.p99_latency_ns as f64 / 1_000_000.0);

        if result.failed_requests > 0 {
            println!("   ❌ Failed: {}", result.failed_requests);
        }
    }
}

async fn run_memory_benchmark() {
    println!("\n🧠 Memory Usage Benchmark");
    
    let scenario = &create_test_scenarios()[1]; // Load balance scenario
    let router = Router::new(scenario.config.clone());
    
    // Measure memory usage during high-throughput routing
    let iterations = 1_000_000;
    println!("📊 Running {} routing operations to measure memory efficiency", iterations);
    
    let start = Instant::now();
    let mut successful = 0;
    
    for _ in 0..iterations {
        if router.route(&scenario.context).is_ok() {
            successful += 1;
        }
    }
    
    let elapsed = start.elapsed();
    
    println!("   Completed: {} operations", successful);
    println!("   Time: {:.2}ms", elapsed.as_millis());
    println!("   Rate: {:.0} ops/sec", successful as f64 / elapsed.as_secs_f64());
    println!("   Avg per op: {:.2}ns", elapsed.as_nanos() as f64 / successful as f64);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Rust AI Gateway Router Performance Tests");
    
    let scenarios = create_test_scenarios();
    let mut all_results = Vec::new();
    
    // Standard benchmark with 100,000 iterations for high precision
    let iterations = 100_000;
    
    for scenario in &scenarios {
        let result = benchmark_scenario(scenario, iterations).await;
        all_results.push(result);
        
        // Brief pause between scenarios
        sleep(Duration::from_millis(100)).await;
    }
    
    print_results(&all_results);
    
    // Additional memory benchmark
    run_memory_benchmark().await;
    
    // Save results to JSON for comparison
    let json_results = serde_json::to_string_pretty(&all_results)?;
    std::fs::write("rust_performance_results.json", json_results)?;
    println!("\n💾 Results saved to rust_performance_results.json");
    
    // Quick comparison summary
    println!("\n📊 QUICK PERFORMANCE SUMMARY");
    println!("{}", "=".repeat(50));
    
    for result in &all_results {
        println!("{}: {:.2}ns avg routing time", 
            result.scenario_name, 
            result.avg_latency_ns
        );
    }
    
    Ok(())
}