use ai_gateway_router::{
    Condition, Router, RouterConfig, RouterContext, Strategy, StrategyMode, Target,
};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;
use tokio;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiTestResult {
    scenario_name: String,
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    success_rate: f64,
    avg_total_latency_ms: f64,
    avg_routing_latency_ns: f64,
    min_total_latency_ms: f64,
    max_total_latency_ms: f64,
    provider_distribution: HashMap<String, usize>,
    routing_overhead_percent: f64,
}

#[derive(Debug, Clone)]
struct TestScenario {
    name: String,
    config: RouterConfig,
    context: RouterContext,
}

fn create_real_api_test_scenarios() -> Vec<TestScenario> {
    let openai_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

    vec![
        TestScenario {
            name: "Single Provider (OpenAI)".to_string(),
            config: RouterConfig {
                mode: StrategyMode::Single,
                targets: vec![Target {
                    name: "openai-gpt35".to_string(),
                    provider: "openai".to_string(),
                    weight: None,
                    api_key: Some(openai_key.clone()),
                    metadata: HashMap::new(),
                    retry_config: None,
                    guardrails: None,
                    model_capabilities: None,
                    request_timeout_ms: None,
                }],
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
            name: "Single Provider (Anthropic)".to_string(),
            config: RouterConfig {
                mode: StrategyMode::Single,
                targets: vec![Target {
                    name: "anthropic-claude".to_string(),
                    provider: "anthropic".to_string(),
                    weight: None,
                    api_key: Some(anthropic_key.clone()),
                    metadata: HashMap::new(),
                    retry_config: None,
                    guardrails: None,
                    model_capabilities: None,
                    request_timeout_ms: None,
                }],
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
                    conditions: vec![Condition {
                        query: json!({
                            "metadata.priority": { "$eq": "high" }
                        }),
                        then_target: "smart_model".to_string(),
                    }],
                    default_target: Some("fast_model".to_string()),
                }),
                global_retry_config: None,
                global_guardrails: None,
                request_timeout_ms: None,
                enable_caching: None,
                cache_ttl_seconds: None,
            },
            context: RouterContext::new().with_metadata("priority".to_string(), "high".to_string()),
        },
    ]
}

async fn make_openai_api_call(api_key: &str) -> Result<(f64, String), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let start = Instant::now();

    let payload = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {"role": "user", "content": "Say 'Hello' in one word."}
        ],
        "max_tokens": 5
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    let latency = start.elapsed().as_millis() as f64;
    let status = response.status();
    let text = response.text().await?;

    if status.is_success() {
        Ok((latency, "openai".to_string()))
    } else {
        Err(format!("OpenAI API error: {} - {}", status, text).into())
    }
}

async fn make_anthropic_api_call(
    api_key: &str,
) -> Result<(f64, String), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let start = Instant::now();

    let payload = json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 5,
        "messages": [
            {"role": "user", "content": "Say 'Hello' in one word."}
        ]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .json(&payload)
        .send()
        .await?;

    let latency = start.elapsed().as_millis() as f64;
    let status = response.status();
    let text = response.text().await?;

    if status.is_success() {
        Ok((latency, "anthropic".to_string()))
    } else {
        Err(format!("Anthropic API error: {} - {}", status, text).into())
    }
}

async fn test_scenario_with_real_apis(scenario: &TestScenario, iterations: usize) -> ApiTestResult {
    println!(
        "🧪 Testing: {} ({} real API calls)",
        scenario.name, iterations
    );

    let router = Router::new(scenario.config.clone());
    let mut total_latencies = Vec::new();
    let mut routing_latencies = Vec::new();
    let mut successful = 0;
    let mut failed = 0;
    let mut provider_distribution = HashMap::new();

    for i in 0..iterations {
        // Measure routing time
        let routing_start = Instant::now();
        let routing_result = router.route(&scenario.context);
        let routing_time = routing_start.elapsed().as_nanos() as f64;

        match routing_result {
            Ok(route) => {
                routing_latencies.push(routing_time);

                // Make actual API call based on routing decision
                let api_result = match route.provider.as_str() {
                    "openai" => {
                        if let Some(api_key) = &route.api_key {
                            make_openai_api_call(api_key).await
                        } else {
                            Err("No API key for OpenAI".into())
                        }
                    }
                    "anthropic" => {
                        if let Some(api_key) = &route.api_key {
                            make_anthropic_api_call(api_key).await
                        } else {
                            Err("No API key for Anthropic".into())
                        }
                    }
                    _ => Err(format!("Unknown provider: {}", route.provider).into()),
                };

                match api_result {
                    Ok((total_latency, provider)) => {
                        total_latencies.push(total_latency);
                        successful += 1;
                        *provider_distribution.entry(provider).or_insert(0) += 1;
                        println!(
                            "   ✅ Request {}: {:.2}ms total, {:.0}ns routing ({})",
                            i + 1,
                            total_latency,
                            routing_time,
                            route.provider
                        );
                    }
                    Err(e) => {
                        failed += 1;
                        println!("   ❌ Request {}: API call failed - {}", i + 1, e);
                    }
                }
            }
            Err(e) => {
                failed += 1;
                println!("   ❌ Request {}: Routing failed - {:?}", i + 1, e);
            }
        }

        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let success_rate = (successful as f64 / iterations as f64) * 100.0;
    let avg_total_latency = if !total_latencies.is_empty() {
        total_latencies.iter().sum::<f64>() / total_latencies.len() as f64
    } else {
        0.0
    };
    let avg_routing_latency = if !routing_latencies.is_empty() {
        routing_latencies.iter().sum::<f64>() / routing_latencies.len() as f64
    } else {
        0.0
    };

    let min_total_latency = total_latencies.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_total_latency = total_latencies.iter().fold(0.0f64, |a, &b| a.max(b));

    let routing_overhead_percent = if avg_total_latency > 0.0 {
        (avg_routing_latency / 1_000_000.0) / avg_total_latency * 100.0
    } else {
        0.0
    };

    println!("   📊 Success Rate: {:.1}%", success_rate);
    println!("   ⚡ Avg Total Latency: {:.2}ms", avg_total_latency);
    println!(
        "   🔀 Avg Routing Latency: {:.0}ns ({:.6}ms)",
        avg_routing_latency,
        avg_routing_latency / 1_000_000.0
    );
    println!(
        "   📈 Routing Overhead: {:.4}% of total time",
        routing_overhead_percent
    );
    println!("   🎯 Provider Distribution: {:?}", provider_distribution);

    ApiTestResult {
        scenario_name: scenario.name.clone(),
        total_requests: iterations,
        successful_requests: successful,
        failed_requests: failed,
        success_rate,
        avg_total_latency_ms: avg_total_latency,
        avg_routing_latency_ns: avg_routing_latency,
        min_total_latency_ms: min_total_latency,
        max_total_latency_ms: max_total_latency,
        provider_distribution,
        routing_overhead_percent,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 RUST AI GATEWAY ROUTER - REAL API CALL TEST");
    println!("===============================================");
    println!("🔑 Using real OpenAI and Anthropic API keys");
    println!("📞 Making actual API calls to measure total performance");
    println!("🔀 Measuring routing overhead vs total request time");

    let scenarios = create_real_api_test_scenarios();
    let mut all_results = Vec::new();

    // Test with 5 real API calls per scenario (to avoid rate limits and costs)
    let iterations = 5;

    for scenario in &scenarios {
        let result = test_scenario_with_real_apis(scenario, iterations).await;
        all_results.push(result);

        // Longer pause between scenarios to respect rate limits
        println!("   ⏳ Waiting 5 seconds before next test...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    // Print comprehensive results
    println!("\n{}", "=".repeat(80));
    println!("📈 RUST REAL API CALL TEST RESULTS");
    println!("{}", "=".repeat(80));

    for result in &all_results {
        println!("\n🎯 {}", result.scenario_name);
        println!("   Total Requests: {}", result.total_requests);
        println!(
            "   Successful: {} ({:.1}%)",
            result.successful_requests, result.success_rate
        );
        println!("   Failed: {}", result.failed_requests);

        if result.successful_requests > 0 {
            println!("   Avg Total Latency: {:.2}ms", result.avg_total_latency_ms);
            println!(
                "   Avg Routing Latency: {:.0}ns ({:.6}ms)",
                result.avg_routing_latency_ns,
                result.avg_routing_latency_ns / 1_000_000.0
            );
            println!("   Min Total Latency: {:.2}ms", result.min_total_latency_ms);
            println!("   Max Total Latency: {:.2}ms", result.max_total_latency_ms);
            println!(
                "   Routing Overhead: {:.4}% of total request time",
                result.routing_overhead_percent
            );
            println!(
                "   Provider Distribution: {:?}",
                result.provider_distribution
            );
        }
    }

    // Save results
    let json_results = serde_json::to_string_pretty(&all_results)?;
    std::fs::write("rust_real_api_results.json", json_results)?;
    println!("\n💾 Results saved to rust_real_api_results.json");

    // Summary
    println!("\n📊 REAL API PERFORMANCE SUMMARY");
    println!("{}", "=".repeat(50));

    for result in &all_results {
        if result.successful_requests > 0 {
            println!(
                "{}: {:.2}ms total ({:.0}ns routing = {:.4}% overhead)",
                result.scenario_name,
                result.avg_total_latency_ms,
                result.avg_routing_latency_ns,
                result.routing_overhead_percent
            );
        } else {
            println!("{}: All requests failed", result.scenario_name);
        }
    }

    println!("\n✅ REAL API TEST COMPLETE!");
    println!("🔍 This shows the actual routing overhead in real-world usage");

    Ok(())
}
