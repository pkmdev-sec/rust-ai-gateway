use ai_gateway_router::{
    IntelligentRouter, IntelligentRoutingContext, TaskType, Urgency, ContentAnalysis, UserPreferences,
    ModelRegistry, TaskRequirements, TaskPriority,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router as AxumRouter,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

/// Enterprise AI Gateway with Intelligent Model Routing
#[derive(Debug)]
pub struct IntelligentGatewayServer {
    intelligent_router: Arc<RwLock<IntelligentRouter>>,
    model_registry: Arc<RwLock<ModelRegistry>>,
    metrics: Arc<RwLock<GatewayMetrics>>,
    port: u16,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct GatewayMetrics {
    pub total_requests: u64,
    pub requests_by_model: HashMap<String, u64>,
    pub avg_routing_time_ms: f64,
    pub success_rate: f64,
    pub cost_savings: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
    pub user_preferences: Option<UserPreferencesRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreferencesRequest {
    pub preferred_provider: Option<String>,
    pub max_latency_ms: Option<u32>,
    pub quality_over_speed: Option<bool>,
    pub cost_conscious: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligentRoutingResponse {
    pub selected_model: String,
    pub model_info: ModelInfo,
    pub routing_reason: String,
    pub estimated_cost: f64,
    pub estimated_latency_ms: u32,
    pub routing_time_ns: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    pub display_name: String,
    pub reasoning_score: u8,
    pub speed_score: u8,
    pub cost_efficiency: u8,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeQuery {
    pub content: String,
    pub task_type: Option<String>,
    pub urgency: Option<String>,
}

impl IntelligentGatewayServer {
    pub fn new(port: u16) -> Self {
        Self {
            intelligent_router: Arc::new(RwLock::new(IntelligentRouter::new())),
            model_registry: Arc::new(RwLock::new(ModelRegistry::new())),
            metrics: Arc::new(RwLock::new(GatewayMetrics::default())),
            port,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app = AxumRouter::new()
            .route("/", get(dashboard))
            .route("/health", get(health_check))
            .route("/v1/chat/completions", post(intelligent_chat_completions))
            .route("/v1/models", get(list_intelligent_models))
            .route("/intelligent/analyze", post(analyze_content))
            .route("/intelligent/route", post(intelligent_route))
            .route("/intelligent/models/recommended", get(get_recommended_models))
            .route("/intelligent/models/complex-tasks", get(get_complex_task_models))
            .route("/intelligent/metrics", get(get_metrics))
            .route("/intelligent/models/:model_id", get(get_model_details))
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
            )
            .with_state(AppState {
                intelligent_router: self.intelligent_router.clone(),
                model_registry: self.model_registry.clone(),
                metrics: self.metrics.clone(),
            });

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        println!("🧠 Intelligent AI Gateway Server running on http://0.0.0.0:{}", self.port);
        println!("📊 Dashboard: http://127.0.0.1:{}/", self.port);
        println!("🔍 Model Analysis: http://127.0.0.1:{}/intelligent/analyze", self.port);
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    intelligent_router: Arc<RwLock<IntelligentRouter>>,
    model_registry: Arc<RwLock<ModelRegistry>>,
    metrics: Arc<RwLock<GatewayMetrics>>,
}

// HTTP Handlers

async fn dashboard() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>🧠 Intelligent AI Gateway</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
        h1 { color: #2563eb; margin-bottom: 30px; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin: 20px 0; }
        .card { background: #f8fafc; padding: 20px; border-radius: 8px; border-left: 4px solid #3b82f6; }
        .card h3 { margin-top: 0; color: #1e40af; }
        .endpoint { background: #ecfdf5; padding: 15px; margin: 10px 0; border-radius: 6px; font-family: monospace; }
        .method { background: #10b981; color: white; padding: 4px 8px; border-radius: 4px; font-size: 12px; margin-right: 10px; }
        .model-list { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 15px; margin: 20px 0; }
        .model-card { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 15px; border-radius: 8px; }
        .model-card h4 { margin: 0 0 10px 0; }
        .score { background: rgba(255,255,255,0.2); padding: 2px 6px; border-radius: 4px; font-size: 12px; margin: 2px; display: inline-block; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🧠 Intelligent AI Gateway Dashboard</h1>
        <p>Advanced AI model routing with automatic selection of Claude 4 and GPT-4 models based on task complexity.</p>
        
        <div class="grid">
            <div class="card">
                <h3>🎯 Intelligent Routing</h3>
                <p>Automatically selects the best model for your task:</p>
                <ul>
                    <li><strong>Complex Rust/Systems:</strong> Claude 3.5 Sonnet</li>
                    <li><strong>Ultra-Complex Reasoning:</strong> Claude 3 Opus</li>
                    <li><strong>Fast Responses:</strong> GPT-4o-mini</li>
                    <li><strong>Multimodal Tasks:</strong> GPT-4o</li>
                    <li><strong>Cost-Conscious:</strong> Claude 3 Haiku</li>
                </ul>
            </div>
            
            <div class="card">
                <h3>🚀 Latest Models</h3>
                <div class="model-list">
                    <div class="model-card">
                        <h4>Claude 3.5 Sonnet</h4>
                        <div class="score">Reasoning: 10/10</div>
                        <div class="score">Speed: 8/10</div>
                        <div class="score">Cost: 7/10</div>
                    </div>
                    <div class="model-card">
                        <h4>GPT-4o</h4>
                        <div class="score">Reasoning: 9/10</div>
                        <div class="score">Speed: 8/10</div>
                        <div class="score">Vision: ✓</div>
                    </div>
                    <div class="model-card">
                        <h4>Claude 3 Opus</h4>
                        <div class="score">Reasoning: 10/10</div>
                        <div class="score">Speed: 6/10</div>
                        <div class="score">Premium</div>
                    </div>
                    <div class="model-card">
                        <h4>GPT-4o-mini</h4>
                        <div class="score">Reasoning: 8/10</div>
                        <div class="score">Speed: 10/10</div>
                        <div class="score">Cost: 10/10</div>
                    </div>
                </div>
            </div>
        </div>

        <h2>🔗 API Endpoints</h2>
        
        <div class="endpoint">
            <span class="method">POST</span>/v1/chat/completions - Intelligent chat completions with automatic model selection
        </div>
        
        <div class="endpoint">
            <span class="method">POST</span>/intelligent/analyze - Analyze content and get routing recommendations
        </div>
        
        <div class="endpoint">
            <span class="method">POST</span>/intelligent/route - Get intelligent model routing for specific content
        </div>
        
        <div class="endpoint">
            <span class="method">GET</span>/intelligent/models/recommended - Get recommended models for complex tasks
        </div>
        
        <div class="endpoint">
            <span class="method">GET</span>/intelligent/models/complex-tasks - Get models optimized for complex tasks
        </div>
        
        <div class="endpoint">
            <span class="method">GET</span>/intelligent/metrics - View routing metrics and performance stats
        </div>

        <h2>💡 Example Usage</h2>
        <div class="endpoint">
curl -X POST http://localhost:8791/intelligent/analyze \\<br>
&nbsp;&nbsp;-H "Content-Type: application/json" \\<br>
&nbsp;&nbsp;-d '{"content": "Implement a concurrent Rust algorithm for enterprise-scale performance optimization"}'
        </div>
    </div>
</body>
</html>
    "#)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "Intelligent AI Gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "features": [
            "intelligent_routing",
            "claude_4_models",
            "gpt_4_models",
            "complexity_analysis",
            "cost_optimization"
        ],
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

async fn intelligent_chat_completions(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    // Extract content from messages
    let content = request.messages
        .iter()
        .map(|msg| format!("{}: {}", msg.role, msg.content))
        .collect::<Vec<_>>()
        .join("\n");

    // Analyze content and get intelligent routing
    let router = state.intelligent_router.read().await;
    let mut context = router.analyze_content(&content);
    
    // Apply user preferences if provided
    if let Some(prefs) = request.user_preferences {
        context.user_preferences.preferred_provider = prefs.preferred_provider;
        context.user_preferences.max_latency_ms = prefs.max_latency_ms;
        context.user_preferences.quality_over_speed = prefs.quality_over_speed.unwrap_or(false);
        context.user_preferences.cost_conscious = prefs.cost_conscious.unwrap_or(false);
    }

    // Route to best model
    let selected_model = match router.route_intelligent(&context, &[]) {
        Ok(model_id) => model_id,
        Err(_) => "gpt-4o-mini".to_string(), // Fallback
    };

    let routing_time = start_time.elapsed();

    // Get model info
    let model_registry = state.model_registry.read().await;
    let model_info = model_registry.get_model(&selected_model);

    // Update metrics
    {
        let mut metrics = state.metrics.write().await;
        metrics.total_requests += 1;
        *metrics.requests_by_model.entry(selected_model.clone()).or_insert(0) += 1;
    }

    let response = serde_json::json!({
        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": selected_model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": format!("🧠 Intelligently routed to {} based on task analysis. Task type: {:?}, Complexity: {:.2}", 
                    selected_model, context.task_type, context.complexity_score)
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": content.len() / 4,
            "completion_tokens": 50,
            "total_tokens": (content.len() / 4) + 50
        },
        "intelligent_routing": {
            "selected_model": selected_model,
            "task_type": format!("{:?}", context.task_type),
            "complexity_score": context.complexity_score,
            "routing_time_ns": routing_time.as_nanos(),
            "model_info": model_info.map(|m| ModelInfo {
                id: m.id.clone(),
                provider: m.provider.clone(),
                display_name: m.display_name.clone(),
                reasoning_score: m.capabilities.reasoning_score,
                speed_score: m.capabilities.speed_score,
                cost_efficiency: m.capabilities.cost_efficiency,
            })
        }
    });

    Ok(Json(response))
}

async fn analyze_content(
    State(state): State<AppState>,
    Json(query): Json<AnalyzeQuery>,
) -> Json<serde_json::Value> {
    let router = state.intelligent_router.read().await;
    let context = router.analyze_content(&query.content);
    
    Json(serde_json::json!({
        "content_analysis": {
            "task_type": format!("{:?}", context.task_type),
            "complexity_score": context.complexity_score,
            "urgency": format!("{:?}", context.urgency),
            "estimated_tokens": context.content_analysis.estimated_tokens,
            "contains_code": context.content_analysis.contains_code,
            "contains_math": context.content_analysis.contains_math,
            "requires_reasoning": context.content_analysis.requires_reasoning,
            "has_images": context.content_analysis.has_images
        },
        "recommendations": {
            "quality_over_speed": context.user_preferences.quality_over_speed,
            "cost_conscious": context.user_preferences.cost_conscious
        }
    }))
}

async fn intelligent_route(
    State(state): State<AppState>,
    Json(query): Json<AnalyzeQuery>,
) -> Result<Json<IntelligentRoutingResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    let router = state.intelligent_router.read().await;
    let context = router.analyze_content(&query.content);
    
    let selected_model = router.route_intelligent(&context, &[])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let routing_time = start_time.elapsed();
    
    // Get model details
    let model_registry = state.model_registry.read().await;
    let model = model_registry.get_model(&selected_model)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let response = IntelligentRoutingResponse {
        selected_model: selected_model.clone(),
        model_info: ModelInfo {
            id: model.id.clone(),
            provider: model.provider.clone(),
            display_name: model.display_name.clone(),
            reasoning_score: model.capabilities.reasoning_score,
            speed_score: model.capabilities.speed_score,
            cost_efficiency: model.capabilities.cost_efficiency,
        },
        routing_reason: format!("Selected based on task type: {:?}, complexity: {:.2}", 
            context.task_type, context.complexity_score),
        estimated_cost: model.pricing.input_cost_per_1k * (context.content_analysis.estimated_tokens as f64 / 1000.0),
        estimated_latency_ms: model.performance.avg_latency_ms,
        routing_time_ns: routing_time.as_nanos() as u64,
    };
    
    Ok(Json(response))
}

async fn get_recommended_models(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let router = state.intelligent_router.read().await;
    let recommended = router.get_recommended_models_for_complex_tasks();
    
    Json(serde_json::json!({
        "recommended_models": recommended,
        "description": "Models recommended for complex tasks requiring high reasoning capabilities"
    }))
}

async fn get_complex_task_models(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let model_registry = state.model_registry.read().await;
    let complex_models = model_registry.get_complex_task_models();
    
    let models_info: Vec<_> = complex_models.iter().map(|model| {
        serde_json::json!({
            "id": model.id,
            "provider": model.provider,
            "display_name": model.display_name,
            "reasoning_score": model.capabilities.reasoning_score,
            "speed_score": model.capabilities.speed_score,
            "cost_efficiency": model.capabilities.cost_efficiency,
            "context_length": model.capabilities.context_length,
            "supports_vision": model.capabilities.supports_vision,
            "use_cases": model.use_cases
        })
    }).collect();
    
    Json(serde_json::json!({
        "complex_task_models": models_info,
        "total_count": models_info.len()
    }))
}

async fn list_intelligent_models(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let model_registry = state.model_registry.read().await;
    let all_models = model_registry.get_all_models();
    
    let models: Vec<_> = all_models.iter().map(|model| {
        serde_json::json!({
            "id": model.id,
            "object": "model",
            "created": chrono::Utc::now().timestamp(),
            "owned_by": model.provider,
            "display_name": model.display_name,
            "capabilities": {
                "reasoning_score": model.capabilities.reasoning_score,
                "speed_score": model.capabilities.speed_score,
                "cost_efficiency": model.capabilities.cost_efficiency,
                "context_length": model.capabilities.context_length,
                "supports_vision": model.capabilities.supports_vision,
                "supports_function_calling": model.capabilities.supports_function_calling
            },
            "pricing": {
                "input_cost_per_1k": model.pricing.input_cost_per_1k,
                "output_cost_per_1k": model.pricing.output_cost_per_1k
            }
        })
    }).collect();
    
    Json(serde_json::json!({
        "object": "list",
        "data": models
    }))
}

async fn get_model_details(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let model_registry = state.model_registry.read().await;
    let model = model_registry.get_model(&model_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(Json(serde_json::json!({
        "id": model.id,
        "provider": model.provider,
        "display_name": model.display_name,
        "capabilities": model.capabilities,
        "pricing": model.pricing,
        "limits": model.limits,
        "performance": model.performance,
        "use_cases": model.use_cases
    })))
}

async fn get_metrics(
    State(state): State<AppState>,
) -> Json<GatewayMetrics> {
    let metrics = state.metrics.read().await;
    Json(metrics.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8791".to_string())
        .parse()
        .unwrap_or(8791);

    let server = IntelligentGatewayServer::new(port);
    server.start().await
}