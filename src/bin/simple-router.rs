use ai_gateway_router::{SimpleAIGateway, SimpleAIGatewayBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Read};

#[derive(Deserialize)]
struct RouteRequest {
    config: RouteConfig,
    content: String,
    metadata: HashMap<String, String>,
}

#[derive(Deserialize)]
struct RouteConfig {
    mode: String,
    targets: Vec<RouteTarget>,
    strategy: Option<RouteStrategy>,
}

#[derive(Deserialize)]
struct RouteTarget {
    name: String,
    provider: String,
    weight: Option<u32>,
    api_key: Option<String>,
    metadata: HashMap<String, String>,
}

#[derive(Deserialize)]
struct RouteStrategy {
    conditions: Option<serde_json::Value>,
    default_target: Option<String>,
}

#[derive(Serialize)]
struct RouteResponse {
    provider: String,
    name: String,
    api_key: Option<String>,
    metadata: HashMap<String, String>,
    routing_time_ns: u64,
    success: bool,
    error: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read JSON input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    
    let request: RouteRequest = serde_json::from_str(&input)?;
    
    // Convert config to SimpleAIGateway
    let mut builder = SimpleAIGatewayBuilder::new();
    
    for target in &request.config.targets {
        match target.provider.as_str() {
            "openai" => {
                if let Some(api_key) = &target.api_key {
                    builder = builder.with_openai(api_key);
                }
            }
            "anthropic" => {
                if let Some(api_key) = &target.api_key {
                    builder = builder.with_anthropic(api_key);
                }
            }
            _ => {} // Skip unsupported providers
        }
    }
    
    // Apply strategy
    match request.config.mode.as_str() {
        "loadbalance" => builder = builder.with_load_balancing(),
        _ => {} // Single provider or unsupported strategy
    }
    
    let gateway = builder.build();
    
    // Route the request
    match gateway.route_request(&request.content, Some(request.metadata)) {
        Ok(result) => {
            let response = RouteResponse {
                provider: result.provider.clone(),
                name: result.provider_name.clone(),
                api_key: Some(result.api_key.clone()),
                metadata: result.metadata,
                routing_time_ns: result.routing_time_ns,
                success: true,
                error: None,
            };
            
            println!("{}", serde_json::to_string(&response)?);
        }
        Err(e) => {
            let response = RouteResponse {
                provider: "error".to_string(),
                name: "error".to_string(),
                api_key: None,
                metadata: HashMap::new(),
                routing_time_ns: 0,
                success: false,
                error: Some(format!("{:?}", e)),
            };
            
            println!("{}", serde_json::to_string(&response)?);
        }
    }
    
    Ok(())
}