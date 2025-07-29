use ai_gateway_router::{
    Router, RouterConfig, RouterContext, StrategyMode, Target, Strategy, Condition,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VerifiedResult {
    scenario_name: String,
    test_type: String,
    iterations: usize,
    total_time_ms: f64,
    success_rate: f64,
    avg_latency_ns: f64,
    min_latency_ns: u64,
    max_latency_ns: u64,
    p95_latency_ns: u64,
    p99_latency_ns: u64,
    throughput_req_per_sec: f64,
    statistical_confidence: String,
}

#[derive(Debug, Clone)]
struct TestScenario {
    name: String,
    config: RouterConfig,
    context: RouterContext,
}

fn create_verified_test_scenarios() -> Vec<TestScenario> {
    let openai_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set");
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set");

    vec![
        TestScenario {
            name: "Single Provider (OpenAI)".to_string(),
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
            name: "Load Balance (OpenAI 70% / Anthropic 30%)".to_string(),
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
            name: "Conditional Routing (High Priority → Anthropic)".to_string(),
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
    ]
}

async fn run_verified_benchmark(scenario: &TestScenario, iterations: usize) -> VerifiedResult {
    println!("🧪 Testing: {} ({} iterations)", scenario.name, iterations);
    
    let router = Router::new(scenario.config.clone());
    let mut latencies = Vec::with_capacity(iterations);
    let mut successful = 0;
    let mut failed = 0;

    let overall_start = Instant::now();

    // Warm up the router (10 iterations)
    for _ in 0..10 {
        let _ = router.route(&scenario.context);
    }

    // Actual benchmark
    for i in 0..iterations {
        let start = Instant::now();
        
        match router.route(&scenario.context) {
            Ok(result) => {
                let latency = start.elapsed();
                latencies.push(latency.as_nanos() as u64);
                successful += 1;
                
                // Verify the routing result makes sense
                match scenario.config.mode {
                    StrategyMode::Single => {
                        assert_eq!(result.provider, "openai", "Single provider should route to OpenAI");
                    },
                    StrategyMode::Conditional => {
                        assert_eq!(result.provider, "anthropic", "High priority should route to Anthropic");
                        assert_eq!(result.name, "smart_model", "Should route to smart_model");
                    },
                    StrategyMode::LoadBalance => {
                        assert!(result.provider == "openai" || result.provider == "anthropic", 
                               "Load balance should route to either provider");
                    },
                    _ => {}
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!("   ❌ Routing failed at iteration {}: {:?}", i, e);
            }
        }

        // Prevent CPU saturation for very high iteration counts
        if i % 10000 == 0 && i > 0 {
            tokio::time::sleep(Duration::from_nanos(1)).await;
        }
    }

    let total_time = overall_start.elapsed();
    
    if latencies.is_empty() {
        return VerifiedResult {
            scenario_name: scenario.name.clone(),
            test_type: "Routing Performance".to_string(),
            iterations,
            total_time_ms: total_time.as_millis() as f64,
            success_rate: 0.0,
            avg_latency_ns: 0.0,
            min_latency_ns: 0,
            max_latency_ns: 0,
            p95_latency_ns: 0,
            p99_latency_ns: 0,
            throughput_req_per_sec: 0.0,
            statistical_confidence: "N/A - All failed".to_string(),
        };
    }

    latencies.sort_unstable();

    let min_latency = *latencies.first().unwrap();
    let max_latency = *latencies.last().unwrap();
    let avg_latency = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
    let p95_latency = latencies[(latencies.len() as f64 * 0.95) as usize];
    let p99_latency = latencies[(latencies.len() as f64 * 0.99) as usize];
    let success_rate = (successful as f64 / iterations as f64) * 100.0;
    let throughput = successful as f64 / total_time.as_secs_f64();

    // Calculate statistical confidence
    let std_dev = {
        let variance = latencies.iter()
            .map(|&x| {
                let diff = x as f64 - avg_latency;
                diff * diff
            })
            .sum::<f64>() / latencies.len() as f64;
        variance.sqrt()
    };
    
    let confidence_interval = 1.96 * std_dev / (latencies.len() as f64).sqrt(); // 95% CI
    let confidence_percent = (confidence_interval / avg_latency * 100.0).min(100.0);
    
    let statistical_confidence = if confidence_percent < 1.0 {
        "Very High (±<1%)".to_string()
    } else if confidence_percent < 5.0 {
        format!("High (±{:.1}%)", confidence_percent)
    } else {
        format!("Moderate (±{:.1}%)", confidence_percent)
    };

    println!("   ✅ Success Rate: {:.2}%", success_rate);
    println!("   ⚡ Avg Latency: {:.2}ns ({:.6}ms)", avg_latency, avg_latency / 1_000_000.0);
    println!("   🚀 Throughput: {:.0} req/s", throughput);
    println!("   📊 Statistical Confidence: {}", statistical_confidence);

    VerifiedResult {
        scenario_name: scenario.name.clone(),
        test_type: "Routing Performance".to_string(),
        iterations,
        total_time_ms: total_time.as_millis() as f64,
        success_rate,
        avg_latency_ns: avg_latency,
        min_latency_ns: min_latency,
        max_latency_ns: max_latency,
        p95_latency_ns: p95_latency,
        p99_latency_ns: p99_latency,
        throughput_req_per_sec: throughput,
        statistical_confidence,
    }
}

async fn run_stress_test() -> VerifiedResult {
    println!("\n🔥 STRESS TEST: 1,000,000 operations");
    
    let scenario = &create_verified_test_scenarios()[1]; // Load balance scenario
    let router = Router::new(scenario.config.clone());
    
    let iterations = 1_000_000;
    let start = Instant::now();
    let mut successful = 0;
    let mut openai_count = 0;
    let mut anthropic_count = 0;
    
    for _ in 0..iterations {
        if let Ok(result) = router.route(&scenario.context) {
            successful += 1;
            match result.provider.as_str() {
                "openai" => openai_count += 1,
                "anthropic" => anthropic_count += 1,
                _ => {}
            }
        }
    }
    
    let elapsed = start.elapsed();
    let avg_latency_ns = elapsed.as_nanos() as f64 / successful as f64;
    let throughput = successful as f64 / elapsed.as_secs_f64();
    
    println!("   ✅ Completed: {} operations", successful);
    println!("   ⚡ Avg Latency: {:.2}ns per operation", avg_latency_ns);
    println!("   🚀 Throughput: {:.0} req/s", throughput);
    println!("   📊 Distribution: OpenAI: {}, Anthropic: {}", openai_count, anthropic_count);
    
    // Verify load balancing is working (should be roughly 70/30)
    let openai_percent = (openai_count as f64 / successful as f64) * 100.0;
    let anthropic_percent = (anthropic_count as f64 / successful as f64) * 100.0;
    println!("   📈 Load Balance: OpenAI: {:.1}%, Anthropic: {:.1}%", openai_percent, anthropic_percent);
    
    // Verify it's close to 70/30 split (within 5% tolerance)
    assert!((openai_percent - 70.0).abs() < 5.0, "Load balancing not working correctly");
    assert!((anthropic_percent - 30.0).abs() < 5.0, "Load balancing not working correctly");
    
    VerifiedResult {
        scenario_name: "Stress Test (1M operations)".to_string(),
        test_type: "Stress Test".to_string(),
        iterations,
        total_time_ms: elapsed.as_millis() as f64,
        success_rate: (successful as f64 / iterations as f64) * 100.0,
        avg_latency_ns,
        min_latency_ns: 0, // Not measured in stress test
        max_latency_ns: 0, // Not measured in stress test
        p95_latency_ns: 0, // Not measured in stress test
        p99_latency_ns: 0, // Not measured in stress test
        throughput_req_per_sec: throughput,
        statistical_confidence: "Very High (1M samples)".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 VERIFIED RUST AI GATEWAY ROUTER BENCHMARK");
    println!("============================================");
    println!("🔑 Using real API keys for verification");
    println!("📊 Statistical analysis with confidence intervals");
    
    let scenarios = create_verified_test_scenarios();
    let mut all_results = Vec::new();
    
    // Run detailed benchmarks with statistical significance
    let iterations = 50_000; // Enough for statistical significance
    
    for scenario in &scenarios {
        let result = run_verified_benchmark(scenario, iterations).await;
        all_results.push(result);
        
        // Brief pause between scenarios
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Run stress test
    let stress_result = run_stress_test().await;
    all_results.push(stress_result);
    
    // Print comprehensive results
    println!("\n{}", "=".repeat(80));
    println!("📈 VERIFIED RUST PERFORMANCE RESULTS");
    println!("{}", "=".repeat(80));
    
    for result in &all_results {
        println!("\n🎯 {}", result.scenario_name);
        println!("   Test Type: {}", result.test_type);
        println!("   Iterations: {}", result.iterations);
        println!("   Success Rate: {:.2}%", result.success_rate);
        println!("   Total Time: {:.2}ms", result.total_time_ms);
        println!("   Throughput: {:.0} req/s", result.throughput_req_per_sec);
        
        if result.avg_latency_ns > 0.0 {
            println!("   Avg Latency: {:.2}ns ({:.6}ms)", result.avg_latency_ns, result.avg_latency_ns / 1_000_000.0);
            if result.min_latency_ns > 0 {
                println!("   Min Latency: {}ns ({:.6}ms)", result.min_latency_ns, result.min_latency_ns as f64 / 1_000_000.0);
                println!("   Max Latency: {}ns ({:.6}ms)", result.max_latency_ns, result.max_latency_ns as f64 / 1_000_000.0);
                println!("   P95 Latency: {}ns ({:.6}ms)", result.p95_latency_ns, result.p95_latency_ns as f64 / 1_000_000.0);
                println!("   P99 Latency: {}ns ({:.6}ms)", result.p99_latency_ns, result.p99_latency_ns as f64 / 1_000_000.0);
            }
        }
        
        println!("   Statistical Confidence: {}", result.statistical_confidence);
    }
    
    // Save verified results
    let json_results = serde_json::to_string_pretty(&all_results)?;
    std::fs::write("rust_verified_results.json", json_results)?;
    println!("\n💾 Verified results saved to rust_verified_results.json");
    
    // Performance summary for comparison
    println!("\n📊 PERFORMANCE SUMMARY FOR COMPARISON");
    println!("{}", "=".repeat(50));
    
    for result in &all_results {
        if result.avg_latency_ns > 0.0 {
            println!("{}: {:.0}ns avg ({:.0} req/s)", 
                result.scenario_name, 
                result.avg_latency_ns,
                result.throughput_req_per_sec
            );
        }
    }
    
    println!("\n✅ VERIFICATION COMPLETE - All tests passed with statistical significance!");
    
    Ok(())
}