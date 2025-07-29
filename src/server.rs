use crate::simple::{SimpleAIGateway, SimpleRequest, SimpleResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "server")]
use {
    axum::{
        extract::{Path, State},
        http::StatusCode,
        response::Json,
        routing::{get, post},
        Router as AxumRouter,
    },
    tower::ServiceBuilder,
    tower_http::cors::CorsLayer,
};

/// HTTP Server wrapper for the AI Gateway
pub struct AIGatewayServer {
    gateway: Arc<RwLock<SimpleAIGateway>>,
    port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
    pub provider: String,
    pub routing_time_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub api_key: String,
    pub weight: Option<u32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub providers: Vec<ProviderConfig>,
    pub load_balancing: Option<bool>,
}

impl AIGatewayServer {
    pub fn new(port: u16) -> Self {
        Self {
            gateway: Arc::new(RwLock::new(SimpleAIGateway::new())),
            port,
        }
    }

    pub async fn configure(&self, config: GatewayConfig) -> Result<(), String> {
        let mut gateway = self.gateway.write().await;
        
        // Clear existing providers
        *gateway = SimpleAIGateway::new();
        
        // Add new providers
        for provider in config.providers {
            gateway.add_provider(
                &provider.name,
                &provider.provider_type,
                &provider.api_key,
                provider.weight,
            );
            
            if let Some(enabled) = provider.enabled {
                gateway.set_provider_enabled(&provider.name, enabled);
            }
        }
        
        // Enable load balancing if requested
        if config.load_balancing.unwrap_or(false) {
            gateway.enable_load_balancing();
        }
        
        Ok(())
    }

    #[cfg(feature = "server")]
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app = AxumRouter::new()
            .route("/", get(health_check))
            .route("/health", get(health_check))
            .route("/v1/chat/completions", post(chat_completions))
            .route("/v1/completions", post(completions))
            .route("/v1/models", get(list_models))
            .route("/gateway/config", post(update_config))
            .route("/gateway/config", get(get_config))
            .route("/gateway/stats", get(get_stats))
            .route("/gateway/providers", get(list_providers))
            .route("/gateway/providers/:name/enable", post(enable_provider))
            .route("/gateway/providers/:name/disable", post(disable_provider))
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
            )
            .with_state(self.gateway.clone());

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        println!("🚀 AI Gateway Server running on http://0.0.0.0:{}", self.port);
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}

// HTTP Handlers
#[cfg(feature = "server")]
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "AI Gateway Router",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

#[cfg(feature = "server")]
async fn chat_completions(
    State(gateway): State<Arc<RwLock<SimpleAIGateway>>>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, StatusCode> {
    let gateway = gateway.read().await;
    
    // Extract content from messages
    let content = request.messages
        .iter()
        .map(|msg| format!("{}: {}", msg.role, msg.content))
        .collect::<Vec<_>>()
        .join("\n");
    
    // Route the request
    match gateway.route_request(&content, None) {
        Ok(result) => {
            let response = ChatCompletionResponse {
                id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
                object: "chat.completion".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                model: request.model.unwrap_or_else(|| result.provider.clone()),
                choices: vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: "assistant".to_string(),
                        content: format!("Routed to {} provider", result.provider),
                    },
                    finish_reason: "stop".to_string(),
                }],
                usage: Usage {
                    prompt_tokens: content.len() as u32 / 4, // Rough estimate
                    completion_tokens: 10,
                    total_tokens: (content.len() as u32 / 4) + 10,
                },
                provider: result.provider,
                routing_time_ns: result.routing_time_ns,
            };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[cfg(feature = "server")]
async fn completions(
    State(gateway): State<Arc<RwLock<SimpleAIGateway>>>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let gateway = gateway.read().await;
    
    let prompt = request.get("prompt")
        .and_then(|p| p.as_str())
        .unwrap_or("Hello");
    
    match gateway.route_request(prompt, None) {
        Ok(result) => {
            Ok(Json(serde_json::json!({
                "id": format!("cmpl-{}", uuid::Uuid::new_v4()),
                "object": "text_completion",
                "created": chrono::Utc::now().timestamp(),
                "model": request.get("model").unwrap_or(&serde_json::json!("default")),
                "choices": [{
                    "text": format!("Routed to {} provider", result.provider),
                    "index": 0,
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": prompt.len() / 4,
                    "completion_tokens": 10,
                    "total_tokens": (prompt.len() / 4) + 10
                },
                "provider": result.provider,
                "routing_time_ns": result.routing_time_ns
            })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[cfg(feature = "server")]
async fn list_models(
    State(gateway): State<Arc<RwLock<SimpleAIGateway>>>,
) -> Json<serde_json::Value> {
    let gateway = gateway.read().await;
    let providers = gateway.get_providers();
    
    let models: Vec<serde_json::Value> = providers.iter().map(|provider| {
        serde_json::json!({
            "id": format!("{}-model", provider.provider_type),
            "object": "model",
            "created": chrono::Utc::now().timestamp(),
            "owned_by": provider.provider_type,
            "provider": provider.name,
            "enabled": provider.enabled
        })
    }).collect();
    
    Json(serde_json::json!({
        "object": "list",
        "data": models
    }))
}

#[cfg(feature = "server")]
async fn update_config(
    State(gateway): State<Arc<RwLock<SimpleAIGateway>>>,
    Json(config): Json<GatewayConfig>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let server = AIGatewayServer {
        gateway: gateway.clone(),
        port: 0, // Not used in this context
    };
    
    match server.configure(config).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": "Configuration updated successfully"
        }))),
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[cfg(feature = "server")]
async fn get_config(
    State(gateway): State<Arc<RwLock<SimpleAIGateway>>>,
) -> Json<serde_json::Value> {
    let gateway = gateway.read().await;
    let providers = gateway.get_providers();
    
    Json(serde_json::json!({
        "providers": providers,
        "stats": gateway.get_stats()
    }))
}

#[cfg(feature = "server")]
async fn get_stats(
    State(gateway): State<Arc<RwLock<SimpleAIGateway>>>,
) -> Json<serde_json::Value> {
    let gateway = gateway.read().await;
    Json(serde_json::json!(gateway.get_stats()))
}

#[cfg(feature = "server")]
async fn list_providers(
    State(gateway): State<Arc<RwLock<SimpleAIGateway>>>,
) -> Json<serde_json::Value> {
    let gateway = gateway.read().await;
    Json(serde_json::json!({
        "providers": gateway.get_providers()
    }))
}

#[cfg(feature = "server")]
async fn enable_provider(
    State(gateway): State<Arc<RwLock<SimpleAIGateway>>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut gateway = gateway.write().await;
    gateway.set_provider_enabled(&name, true);
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Provider '{}' enabled", name)
    })))
}

#[cfg(feature = "server")]
async fn disable_provider(
    State(gateway): State<Arc<RwLock<SimpleAIGateway>>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut gateway = gateway.write().await;
    gateway.set_provider_enabled(&name, false);
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Provider '{}' disabled", name)
    })))
}

// Convenience function to start a server with configuration
pub async fn start_gateway_server(port: u16, config: GatewayConfig) -> Result<(), Box<dyn std::error::Error>> {
    let server = AIGatewayServer::new(port);
    server.configure(config).await?;
    server.start().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = AIGatewayServer::new(8080);
        assert_eq!(server.port, 8080);
    }

    #[tokio::test]
    async fn test_configuration() {
        let server = AIGatewayServer::new(8080);
        
        let config = GatewayConfig {
            providers: vec![
                ProviderConfig {
                    name: "openai".to_string(),
                    provider_type: "openai".to_string(),
                    api_key: "test-key".to_string(),
                    weight: Some(70),
                    enabled: Some(true),
                },
            ],
            load_balancing: Some(false),
        };
        
        assert!(server.configure(config).await.is_ok());
    }
}