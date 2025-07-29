use ai_gateway_router::{SimpleAIGateway, SimpleAIGatewayBuilder, AIGatewayServer, GatewayConfig, ProviderConfig};
use clap::{Parser, Subcommand};
use serde_json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "ai-gateway")]
#[command(about = "A fast, reliable AI Gateway Router")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8787")]
        port: u16,
        
        /// Configuration file path
        #[arg(short, long)]
        config: Option<String>,
        
        /// OpenAI API key
        #[arg(long)]
        openai_key: Option<String>,
        
        /// Anthropic API key
        #[arg(long)]
        anthropic_key: Option<String>,
        
        /// Enable load balancing
        #[arg(long)]
        load_balance: bool,
    },
    
    /// Test routing with a sample request
    Test {
        /// OpenAI API key
        #[arg(long)]
        openai_key: Option<String>,
        
        /// Anthropic API key
        #[arg(long)]
        anthropic_key: Option<String>,
        
        /// Test message
        #[arg(short, long, default_value = "Hello, world!")]
        message: String,
        
        /// Number of test requests
        #[arg(short, long, default_value = "10")]
        count: usize,
        
        /// Enable load balancing
        #[arg(long)]
        load_balance: bool,
    },
    
    /// Generate a sample configuration file
    Config {
        /// Output file path
        #[arg(short, long, default_value = "gateway-config.json")]
        output: String,
    },
    
    /// Route a request (reads JSON from stdin)
    Route,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Serve { port, config, openai_key, anthropic_key, load_balance } => {
            println!("🚀 Starting AI Gateway Server on port {}", port);
            
            let gateway_config = if let Some(config_path) = config {
                // Load from file
                let config_content = std::fs::read_to_string(config_path)?;
                serde_json::from_str(&config_content)?
            } else {
                // Create from command line args
                let mut providers = Vec::new();
                
                if let Some(key) = openai_key {
                    providers.push(ProviderConfig {
                        name: "openai".to_string(),
                        provider_type: "openai".to_string(),
                        api_key: key,
                        weight: if load_balance { Some(70) } else { None },
                        enabled: Some(true),
                    });
                }
                
                if let Some(key) = anthropic_key {
                    providers.push(ProviderConfig {
                        name: "anthropic".to_string(),
                        provider_type: "anthropic".to_string(),
                        api_key: key,
                        weight: if load_balance { Some(30) } else { None },
                        enabled: Some(true),
                    });
                }
                
                if providers.is_empty() {
                    eprintln!("❌ Error: No API keys provided. Use --openai-key or --anthropic-key");
                    std::process::exit(1);
                }
                
                GatewayConfig {
                    providers,
                    load_balancing: Some(load_balance),
                }
            };
            
            ai_gateway_router::start_gateway_server(port, gateway_config).await?;
        }
        
        Commands::Test { openai_key, anthropic_key, message, count, load_balance } => {
            println!("🧪 Testing AI Gateway Router");
            
            let mut builder = SimpleAIGatewayBuilder::new();
            
            if let Some(key) = openai_key {
                builder = builder.with_openai(&key);
            }
            
            if let Some(key) = anthropic_key {
                builder = builder.with_anthropic(&key);
            }
            
            if load_balance {
                builder = builder.with_load_balancing();
            }
            
            let gateway = builder.build();
            
            if gateway.get_providers().is_empty() {
                eprintln!("❌ Error: No API keys provided. Use --openai-key or --anthropic-key");
                std::process::exit(1);
            }
            
            println!("📊 Running {} test requests...", count);
            
            let mut provider_counts = HashMap::new();
            let mut total_routing_time = 0u64;
            
            for i in 1..=count {
                match gateway.route_request(&message, None) {
                    Ok(result) => {
                        *provider_counts.entry(result.provider.clone()).or_insert(0) += 1;
                        total_routing_time += result.routing_time_ns;
                        
                        println!("✅ Request {}: {} ({:.2}μs routing)", 
                                i, result.provider, result.routing_time_ns as f64 / 1000.0);
                    }
                    Err(e) => {
                        println!("❌ Request {}: Error - {:?}", i, e);
                    }
                }
            }
            
            println!("\n📈 Test Results:");
            println!("   Total Requests: {}", count);
            println!("   Avg Routing Time: {:.2}μs", total_routing_time as f64 / count as f64 / 1000.0);
            println!("   Provider Distribution:");
            for (provider, count) in provider_counts {
                println!("     {}: {} requests", provider, count);
            }
        }
        
        Commands::Config { output } => {
            let sample_config = GatewayConfig {
                providers: vec![
                    ProviderConfig {
                        name: "openai".to_string(),
                        provider_type: "openai".to_string(),
                        api_key: "your-openai-api-key-here".to_string(),
                        weight: Some(70),
                        enabled: Some(true),
                    },
                    ProviderConfig {
                        name: "anthropic".to_string(),
                        provider_type: "anthropic".to_string(),
                        api_key: "your-anthropic-api-key-here".to_string(),
                        weight: Some(30),
                        enabled: Some(true),
                    },
                ],
                load_balancing: Some(true),
            };
            
            let config_json = serde_json::to_string_pretty(&sample_config)?;
            std::fs::write(&output, config_json)?;
            
            println!("✅ Sample configuration written to {}", output);
            println!("📝 Edit the file to add your API keys and customize settings");
        }
        
        Commands::Route => {
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
                "fallback" => builder = builder.with_fallback(),
                _ => {} // Single provider or unsupported strategy
            }
            
            let gateway = builder.build();
            
            // Route the request
            match gateway.route_request(&request.content, Some(request.metadata)) {
                Ok(result) => {
                    let response = RouteResponse {
                        provider: result.provider,
                        name: result.name,
                        api_key: result.api_key,
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
        }
    }
    
    Ok(())
}

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