use ai_gateway_router::{SimpleAIGateway, SimpleAIGatewayBuilder};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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
    println!("🚀 Starting Improved OpenCode AI Gateway Server...");
    
    let openai_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| "sk-test-key".to_string());
    
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY")
        .unwrap_or_else(|_| "sk-ant-test-key".to_string());
    
    let gateway = Arc::new(SimpleAIGatewayBuilder::new()
        .with_openai(&openai_key)
        .with_anthropic(&anthropic_key)
        .with_load_balancing()
        .build());
    
    println!("✅ AI Gateway configured with {} providers", gateway.get_providers().len());
    
    let listener = TcpListener::bind("127.0.0.1:8788")?;
    println!("🌐 Improved server listening on http://127.0.0.1:8788");
    println!("🔧 Fixed socket hangup issues with proper connection handling");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let gateway_clone = Arc::clone(&gateway);
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, gateway_clone) {
                        eprintln!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    
    Ok(())
}

fn handle_connection(mut stream: TcpStream, gateway: Arc<SimpleAIGateway>) -> Result<(), Box<dyn std::error::Error>> {
    // Set timeouts to prevent hanging
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    
    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 3 {
        send_error_response(&mut stream, 400, "Bad Request")?;
        return Ok(());
    }
    
    let method = parts[0];
    let path = parts[1];
    
    // Read headers
    let mut headers = std::collections::HashMap::new();
    let mut content_length = 0;
    
    loop {
        let mut header_line = String::new();
        reader.read_line(&mut header_line)?;
        
        if header_line.trim().is_empty() {
            break;
        }
        
        if let Some(colon_pos) = header_line.find(':') {
            let key = header_line[..colon_pos].trim().to_lowercase();
            let value = header_line[colon_pos + 1..].trim();
            
            if key == "content-length" {
                content_length = value.parse().unwrap_or(0);
            }
            
            headers.insert(key, value.to_string());
        }
    }
    
    // Handle CORS preflight
    if method == "OPTIONS" {
        send_cors_response(&mut stream)?;
        return Ok(());
    }
    
    // Handle health check
    if method == "GET" && (path == "/" || path == "/health") {
        send_success_response(&mut stream, "OK")?;
        return Ok(());
    }
    
    // Handle chat completions
    if method == "POST" && path == "/v1/chat/completions" {
        // Read body
        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            std::io::Read::read_exact(&mut reader, &mut body)?;
        }
        
        let body_str = String::from_utf8_lossy(&body);
        handle_chat_completion(&mut stream, &body_str, &gateway)?;
        return Ok(());
    }
    
    send_error_response(&mut stream, 404, "Not Found")?;
    Ok(())
}

fn handle_chat_completion(
    stream: &mut TcpStream, 
    body: &str, 
    gateway: &Arc<SimpleAIGateway>
) -> Result<(), Box<dyn std::error::Error>> {
    let chat_request: ChatRequest = match serde_json::from_str(body.trim()) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Failed to parse chat request: {}", e);
            send_error_response(stream, 400, "Bad Request")?;
            return Ok(());
        }
    };
    
    let user_message = chat_request.messages
        .iter()
        .find(|msg| msg.role == "user")
        .map(|msg| msg.content.clone())
        .unwrap_or_default();
    
    let start_time = std::time::Instant::now();
    
    match gateway.route_request(&user_message, None) {
        Ok(result) => {
            let routing_time = start_time.elapsed();
            
            println!("🚀 Routed to {} in {:?} ({}μs)", 
                    result.provider, routing_time, result.routing_time_ns / 1000);
            
            let response = ChatResponse {
                id: format!("chatcmpl-{}", generate_id()),
                object: "chat.completion".to_string(),
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                model: chat_request.model.clone(),
                choices: vec![Choice {
                    index: 0,
                    message: ResponseMessage {
                        role: "assistant".to_string(),
                        content: format!(
                            "Hello! I'm responding via the Improved Rust AI Gateway. Your request for model '{}' was routed to {} in {}μs. Socket hangup issues have been fixed with proper connection handling!", 
                            chat_request.model, result.provider, result.routing_time_ns / 1000
                        ),
                    },
                    finish_reason: "stop".to_string(),
                }],
                usage: Usage {
                    prompt_tokens: user_message.len() as u32 / 4,
                    completion_tokens: 50,
                    total_tokens: (user_message.len() as u32 / 4) + 50,
                },
            };
            
            let json = serde_json::to_string(&response)?;
            send_json_response(stream, &json)?;
        }
        Err(e) => {
            eprintln!("Routing failed: {:?}", e);
            send_error_response(stream, 500, "Routing Error")?;
        }
    }
    
    Ok(())
}

fn send_cors_response(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let response = "HTTP/1.1 200 OK\r\n\
                   Access-Control-Allow-Origin: *\r\n\
                   Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
                   Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
                   Content-Length: 0\r\n\
                   Connection: close\r\n\
                   \r\n";
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn send_success_response(stream: &mut TcpStream, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/plain\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        message.len(),
        message
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn send_json_response(stream: &mut TcpStream, json: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/json\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        json.len(),
        json
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn send_error_response(stream: &mut TcpStream, status: u16, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let body = format!("{{\"error\": \"{}\"}}", message);
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: application/json\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        status, message, body.len(), body
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}