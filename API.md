# 🔌 API Documentation

## 📋 **Overview**

The Rust AI Gateway provides a RESTful API that's fully compatible with OpenAI's API specification, while adding ultra-fast routing, load balancing, and automatic failover capabilities.

**Base URL**: `http://127.0.0.1:8788`

## 🚀 **Core Endpoints**

### Chat Completions

Create a chat completion with automatic provider routing.

```http
POST /v1/chat/completions
```

#### Headers

```http
Content-Type: application/json
Authorization: Bearer your-api-key
```

#### Request Body

```json
{
  "model": "gpt-4o-mini",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user",
      "content": "Hello! How are you?"
    }
  ],
  "max_tokens": 150,
  "temperature": 0.7,
  "stream": false
}
```

#### Response

```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-4o-mini",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! I'm doing well, thank you for asking. How can I help you today?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 18,
    "total_tokens": 38
  },
  "_gateway": {
    "provider": "openai",
    "routing_time_us": 94,
    "total_time_ms": 1.2
  }
}
```

#### Streaming Response

Set `"stream": true` for streaming responses:

```bash
curl -X POST http://127.0.0.1:8788/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Count to 5"}],
    "stream": true
  }'
```

Response (Server-Sent Events):

```
data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4o-mini","choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":null}]}

data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4o-mini","choices":[{"index":0,"delta":{"content":"1"},"finish_reason":null}]}

data: {"id":"chatcmpl-abc123","object":"chat.completion.chunk","created":1677652288,"model":"gpt-4o-mini","choices":[{"index":0,"delta":{"content":", 2"},"finish_reason":null}]}

data: [DONE]
```

### Text Completions (Legacy)

```http
POST /v1/completions
```

#### Request Body

```json
{
  "model": "gpt-3.5-turbo-instruct",
  "prompt": "Once upon a time",
  "max_tokens": 50,
  "temperature": 0.7
}
```

### Embeddings

Generate embeddings for text input.

```http
POST /v1/embeddings
```

#### Request Body

```json
{
  "model": "text-embedding-ada-002",
  "input": "The quick brown fox jumps over the lazy dog"
}
```

#### Response

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "embedding": [0.0023064255, -0.009327292, ...],
      "index": 0
    }
  ],
  "model": "text-embedding-ada-002",
  "usage": {
    "prompt_tokens": 8,
    "total_tokens": 8
  },
  "_gateway": {
    "provider": "openai",
    "routing_time_us": 45
  }
}
```

## 🔍 **Monitoring Endpoints**

### Health Check

Simple health check endpoint.

```http
GET /health
```

#### Response

```
OK
```

### Detailed Status

Get detailed gateway and provider status.

```http
GET /status
```

#### Response

```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "version": "1.0.0",
  "providers": {
    "openai": {
      "status": "healthy",
      "last_check": "2024-01-15T10:30:00Z",
      "response_time_ms": 120,
      "success_rate": 0.99
    },
    "anthropic": {
      "status": "healthy",
      "last_check": "2024-01-15T10:30:00Z",
      "response_time_ms": 95,
      "success_rate": 1.0
    }
  },
  "routing": {
    "total_requests": 1500,
    "successful_requests": 1485,
    "failed_requests": 15,
    "average_routing_time_us": 87
  }
}
```

### Metrics

Get performance metrics in Prometheus format.

```http
GET /metrics
```

#### Response

```
# HELP gateway_requests_total Total number of requests
# TYPE gateway_requests_total counter
gateway_requests_total{provider="openai"} 1200
gateway_requests_total{provider="anthropic"} 300

# HELP gateway_request_duration_seconds Request duration in seconds
# TYPE gateway_request_duration_seconds histogram
gateway_request_duration_seconds_bucket{provider="openai",le="0.1"} 1150
gateway_request_duration_seconds_bucket{provider="openai",le="0.5"} 1200
gateway_request_duration_seconds_bucket{provider="openai",le="+Inf"} 1200

# HELP gateway_routing_duration_microseconds Routing decision time in microseconds
# TYPE gateway_routing_duration_microseconds histogram
gateway_routing_duration_microseconds_bucket{le="50"} 800
gateway_routing_duration_microseconds_bucket{le="100"} 1400
gateway_routing_duration_microseconds_bucket{le="+Inf"} 1500
```

## 🔧 **Configuration Endpoints**

### Get Configuration

```http
GET /config
```

#### Response

```json
{
  "routing_strategy": "load_balance",
  "providers": ["openai", "anthropic"],
  "health_check_interval": 30,
  "retry_attempts": 3,
  "timeout_seconds": 30
}
```

### Update Configuration (Admin)

```http
PUT /config
Authorization: Bearer admin-token
```

#### Request Body

```json
{
  "routing_strategy": "fallback",
  "health_check_interval": 60,
  "retry_attempts": 5
}
```

## 📊 **Request/Response Headers**

### Request Headers

| Header               | Description                    | Required |
| -------------------- | ------------------------------ | -------- |
| `Authorization`      | Bearer token (any value works) | ✅       |
| `Content-Type`       | Must be `application/json`     | ✅       |
| `X-Gateway-Provider` | Force specific provider        | ❌       |
| `X-Gateway-Timeout`  | Override timeout (seconds)     | ❌       |
| `X-Gateway-Retries`  | Override retry count           | ❌       |

### Response Headers

| Header                   | Description                          |
| ------------------------ | ------------------------------------ |
| `X-Gateway-Provider`     | Provider that handled the request    |
| `X-Gateway-Routing-Time` | Routing decision time (microseconds) |
| `X-Gateway-Total-Time`   | Total request time (milliseconds)    |
| `X-Gateway-Attempt`      | Attempt number (for retries)         |

## 🚨 **Error Handling**

### Error Response Format

```json
{
  "error": {
    "message": "The model 'invalid-model' does not exist",
    "type": "invalid_request_error",
    "param": "model",
    "code": "model_not_found"
  },
  "_gateway": {
    "provider": "openai",
    "routing_time_us": 23,
    "attempts": 1
  }
}
```

### HTTP Status Codes

| Code  | Description                              |
| ----- | ---------------------------------------- |
| `200` | Success                                  |
| `400` | Bad Request - Invalid parameters         |
| `401` | Unauthorized - Invalid API key           |
| `429` | Too Many Requests - Rate limited         |
| `500` | Internal Server Error - Gateway error    |
| `502` | Bad Gateway - Provider error             |
| `503` | Service Unavailable - All providers down |
| `504` | Gateway Timeout - Request timeout        |

### Provider-Specific Errors

The gateway preserves original error responses from providers:

#### OpenAI Error

```json
{
  "error": {
    "message": "You exceeded your current quota",
    "type": "insufficient_quota",
    "param": null,
    "code": "insufficient_quota"
  },
  "_gateway": {
    "provider": "openai",
    "routing_time_us": 45,
    "will_retry": true,
    "next_provider": "anthropic"
  }
}
```

#### Anthropic Error

```json
{
  "error": {
    "type": "invalid_request_error",
    "message": "messages: field required"
  },
  "_gateway": {
    "provider": "anthropic",
    "routing_time_us": 32,
    "attempts": 2
  }
}
```

## 🔄 **Load Balancing & Failover**

### Provider Selection

The gateway automatically selects providers based on:

1. **Health Status** - Unhealthy providers are skipped
2. **Load Balancing** - Weighted distribution across healthy providers
3. **Failover** - Automatic retry with different provider on failure

### Force Provider Selection

Use the `X-Gateway-Provider` header to force a specific provider:

```bash
curl -X POST http://127.0.0.1:8788/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test" \
  -H "X-Gateway-Provider: anthropic" \
  -d '{
    "model": "claude-3-sonnet-20240229",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

## 🎯 **Model Mapping**

The gateway automatically maps model names to appropriate providers:

| Model Pattern      | Provider  | Notes             |
| ------------------ | --------- | ----------------- |
| `gpt-*`            | OpenAI    | All GPT models    |
| `claude-*`         | Anthropic | All Claude models |
| `text-embedding-*` | OpenAI    | Embedding models  |
| `dall-e-*`         | OpenAI    | Image generation  |

### Custom Model Mapping

Override model routing with configuration:

```yaml
model_mapping:
  'custom-gpt': 'openai'
  'custom-claude': 'anthropic'
  'fast-model': 'openai'
  'smart-model': 'anthropic'
```

## 📈 **Performance Optimization**

### Request Batching

Send multiple requests concurrently for better throughput:

```bash
# Send 10 concurrent requests
for i in {1..10}; do
  curl -X POST http://127.0.0.1:8788/v1/chat/completions \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer test" \
    -d '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"Hello '$i'"}]}' &
done
wait
```

### Connection Reuse

Use HTTP/1.1 keep-alive or HTTP/2 for better performance:

```bash
# Enable keep-alive
curl --http1.1 --keepalive-time 60 ...
```

## 🔐 **Security**

### API Key Validation

The gateway accepts any non-empty Bearer token and uses configured provider API keys internally:

```bash
# These all work the same
curl -H "Authorization: Bearer test" ...
curl -H "Authorization: Bearer my-app-token" ...
curl -H "Authorization: Bearer $(uuidgen)" ...
```

### Rate Limiting

Built-in rate limiting per client:

```http
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1677652348

{
  "error": {
    "message": "Rate limit exceeded. Try again in 60 seconds.",
    "type": "rate_limit_exceeded"
  }
}
```

## 🧪 **Testing**

### Health Check Script

```bash
#!/bin/bash
# health-check.sh

GATEWAY_URL="http://127.0.0.1:8788"

# Basic health check
if curl -f "$GATEWAY_URL/health" > /dev/null 2>&1; then
    echo "✅ Gateway is healthy"
else
    echo "❌ Gateway is down"
    exit 1
fi

# Test chat completion
response=$(curl -s -X POST "$GATEWAY_URL/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Say hello"}],
    "max_tokens": 10
  }')

if echo "$response" | jq -e '.choices[0].message.content' > /dev/null 2>&1; then
    echo "✅ Chat completion working"
else
    echo "❌ Chat completion failed"
    echo "$response"
    exit 1
fi

echo "🎉 All tests passed!"
```

### Load Testing

```bash
# Install hey (HTTP load testing tool)
go install github.com/rakyll/hey@latest

# Run load test
hey -n 1000 -c 10 -m POST \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test" \
  -d '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"Hello"}],"max_tokens":10}' \
  http://127.0.0.1:8788/v1/chat/completions
```

---

**🚀 Ready to build amazing AI applications with ultra-fast routing!**

For more examples and advanced usage, check out the [examples directory](./examples/) in the repository.
