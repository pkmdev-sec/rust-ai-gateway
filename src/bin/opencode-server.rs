use ai_gateway_router::{SimpleAIGateway, SimpleAIGatewayBuilder};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

#[derive(Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    stream: Option<bool>,
}

#[derive(Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Serialize)]
struct Choice {
    index: u32,
    message: ResponseMessage,
    finish_reason: String,
}

#[derive(Serialize)]
struct ResponseMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting OpenCode AI Gateway Server with Rust Router...");
    
    // Set up the AI Gateway with API keys from environment
    let openai_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable is required");
    
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
        .unwrap_or_else(|_| {
            println!("⚠️  ANTHROPIC_API_KEY not set - Anthropic provider will be unavailable");
            String::new()
        });
    
    let gateway = Arc::new(SimpleAIGatewayBuilder::new()
        .with_openai(&openai_key)
        .with_anthropic(&anthropic_key)
        .with_load_balancing()
        .build());
    
    println!("✅ AI Gateway configured with {} providers", gateway.get_providers().len());
    
    // Start HTTP server
    let listener = TcpListener::bind("127.0.0.1:8788")?;
    println!("🌐 Server listening on http://127.0.0.1:8788");
    println!("📡 OpenCode can now use this endpoint directly!");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let gateway_clone = Arc::clone(&gateway);
                thread::spawn(move || {
                    handle_request(stream, gateway_clone);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    
    Ok(())
}

fn handle_request(mut stream: TcpStream, gateway: Arc<SimpleAIGateway>) {
    let mut buffer = [0; 4096];
    
    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);
            
            // Parse HTTP request
            if let Some(response) = process_http_request(&request, &gateway) {
                let _ = stream.write_all(response.as_bytes());
            } else {
                let error_response = create_error_response(500, "Internal Server Error");
                let _ = stream.write_all(error_response.as_bytes());
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}

fn process_http_request(request: &str, gateway: &Arc<SimpleAIGateway>) -> Option<String> {
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() {
        return None;
    }
    
    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    
    if parts.len() < 3 {
        return None;
    }
    
    let method = parts[0];
    let path = parts[1];
    
    // Handle CORS preflight
    if method == "OPTIONS" {
        return Some(create_cors_response());
    }
    
    // Handle health check
    if method == "GET" && (path == "/" || path == "/health") {
        return Some(create_success_response("OK"));
    }
    
    // Handle chat completions
    if method == "POST" && path == "/v1/chat/completions" {
        // Find the JSON body
        if let Some(body_start) = request.find("\r\n\r\n") {
            let body = &request[body_start + 4..];
            return handle_chat_completion(body, gateway);
        }
    }
    
    Some(create_error_response(404, "Not Found"))
}

fn handle_chat_completion(body: &str, gateway: &Arc<SimpleAIGateway>) -> Option<String> {
    // Parse the chat request
    let chat_request: ChatRequest = match serde_json::from_str(body.trim_end_matches('\0')) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Failed to parse chat request: {}", e);
            return Some(create_error_response(400, "Bad Request"));
        }
    };
    
    // Extract the user message
    let user_message = chat_request.messages
        .iter()
        .find(|msg| msg.role == "user")
        .map(|msg| msg.content.clone())
        .unwrap_or_default();
    
    // Route the request using our Rust AI Gateway
    let start_time = std::time::Instant::now();
    
    match gateway.route_request(&user_message, None) {
        Ok(result) => {
            let routing_time = start_time.elapsed();
            
            println!("🚀 Routed to {} in {:?} ({}μs)", 
                    result.provider, routing_time, result.routing_time_ns / 1000);
            
            // Create a mock response (in a real implementation, you'd call the actual API)
            let response = ChatResponse {
                id: format!("chatcmpl-{}", generate_id()),
                object: "chat.completion".to_string(),
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                model: chat_request.model,
                choices: vec![Choice {
                    index: 0,
                    message: ResponseMessage {
                        role: "assistant".to_string(),
                        content: format!("Hello! I'm responding via the Rust AI Gateway Router. Your request was routed to {} in {}μs. This demonstrates the ultra-fast routing capabilities!", 
                                       result.provider, result.routing_time_ns / 1000),
                    },
                    finish_reason: "stop".to_string(),
                }],
                usage: Usage {
                    prompt_tokens: user_message.len() as u32 / 4, // Rough estimate
                    completion_tokens: 50,
                    total_tokens: (user_message.len() as u32 / 4) + 50,
                },
            };
            
            match serde_json::to_string(&response) {
                Ok(json) => Some(create_json_response(&json)),
                Err(e) => {
                    eprintln!("Failed to serialize response: {}", e);
                    Some(create_error_response(500, "Internal Server Error"))
                }
            }
        }
        Err(e) => {
            eprintln!("Routing failed: {:?}", e);
            Some(create_error_response(500, "Routing Error"))
        }
    }
}

fn create_cors_response() -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
         Content-Length: 0\r\n\
         \r\n"
    )
}

fn create_success_response(message: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/plain\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {}",
        message.len(),
        message
    )
}

fn create_json_response(json: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/json\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {}",
        json.len(),
        json
    )
}

fn create_error_response(status: u16, message: &str) -> String {
    let body = format!("{{\"error\": \"{}\"}}", message);
    format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {}",
        status, message, body.len(), body
    )
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}