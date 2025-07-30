# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a high-performance AI model router built in Rust, designed to provide intelligent routing to 250+ language models with sub-millisecond latency. The project supports multiple AI providers (OpenAI, Anthropic, etc.) with advanced routing strategies including load balancing, fallback, and conditional routing.

## Common Development Commands

### Build Commands
```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# Build with specific features
cargo build --features="async,server"
cargo build --features="nodejs"
cargo build --no-default-features --features="simple"
```

### Testing Commands
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration

# Run async tests
cargo test --features="async"
```

### Development Quality Commands
```bash
# Format code (must run before commits)
cargo fmt

# Run clippy linting with warnings as errors
cargo clippy -- -D warnings

# Run benchmarks (if available)
cargo bench
```

### Running Examples
```bash
# Basic usage examples
cargo run --example basic_usage
cargo run --example advanced_usage
cargo run --example performance_benchmark
cargo run --example real_api_test
```

### Server Binaries
```bash
# Run different server variants
cargo run --bin intelligent-server --features="server"
cargo run --bin opencode-server --features="server" 
cargo run --bin enterprise-server --features="server"
cargo run --bin simple-router --features="simple"
```

### Node.js Bindings
```bash
# Build Node.js native module
npm run build

# Build debug version for Node.js
npm run build:debug

# Test Node.js bindings
npm test
```

## Architecture Overview

### Core Components

**Router System (`src/router.rs`)**:
- Main routing engine with 4 strategies: Single, LoadBalance, Fallback, Conditional
- Thread-safe routing with `Send + Sync` implementations
- Zero-copy operations where possible for performance

**Strategy Engine (`src/strategy.rs`)**:
- Conditional routing with MongoDB-style query operators ($eq, $gt, $and, $or, etc.)
- Load balancing with weighted random distribution
- Complex query evaluation for metadata-based routing

**Intelligent Router (`src/intelligent_router.rs`)**:
- AI-powered model selection based on task analysis
- Cost optimization and performance prediction
- User preference learning and model capability matching

**Configuration System (`src/config.rs`)**:
- Structured configuration with RouterConfig, Target, and Strategy types
- Support for API keys, weights, metadata, and retry configurations
- Feature-flag based conditional compilation

### Key Features

**Multiple Routing Modes**:
- Single: Route to one provider
- LoadBalance: Weighted distribution across providers
- Fallback: Sequential retry with automatic failover
- Conditional: Complex query-based routing with metadata

**Enterprise Features** (with `server` feature):
- HTTP server with Axum framework
- CORS support and middleware
- Metrics collection and monitoring
- Production-ready deployment options

**Multi-Language Support**:
- Native Rust library (`rlib`)
- Node.js bindings with NAPI (`nodejs` feature)
- WebAssembly compatibility (`cdylib`)

## Feature Flags System

**Default Features**: `["async", "simple", "uuid"]`

**Available Features**:
- `async`: Tokio-based async support with reqwest
- `simple`: Basic synchronous routing
- `server`: HTTP server with Axum, CORS, monitoring
- `nodejs`: Node.js native bindings with NAPI
- `full`: All features enabled

**Feature-Specific Files**:
- `src/simple.rs`: Simple synchronous router (requires `simple`)
- `src/server.rs`: HTTP server implementation (requires `server`)
- `src/nodejs.rs`: Node.js bindings (requires `nodejs`)

## Code Patterns

### Router Configuration Pattern
```rust
let config = RouterConfig {
    mode: StrategyMode::Conditional,
    targets: vec![Target { /* ... */ }],
    strategy: Some(Strategy {
        conditions: vec![Condition { /* ... */ }],
        default_target: Some("fallback".to_string()),
    }),
    // ... other config
};
```

### Conditional Query Patterns
The system uses MongoDB-style queries for conditional routing:
```rust
// Complex AND/OR logic
json!({
    "$or": [
        { "metadata.user_tier": { "$eq": "premium" } },
        {
            "$and": [
                { "metadata.priority": { "$eq": "high" } },
                { "params.token_count": { "$gt": 500 } }
            ]
        }
    ]
})
```

### Error Handling Pattern
All routing operations return `RouterResult<T>` which is `Result<T, RouterError>`. The codebase uses `thiserror` for structured error handling.

## Testing Guidelines

### Unit Tests
- All major components have comprehensive unit tests in their respective files
- Tests cover different routing strategies and edge cases
- Use `RouterContext::default()` or `RouterContext::new()` for test contexts

### Integration Tests
- Examples in `examples/` directory serve as integration tests
- Performance benchmarks validate sub-millisecond routing claims
- Real API tests verify provider integration (when API keys available)

### Test Structure
Tests are embedded in source files using `#[cfg(test)]` modules. Key test files:
- `src/lib.rs`: Core routing strategy tests
- `src/intelligent_router.rs`: AI-powered routing tests
- `src/guardrails.rs`: Content moderation tests
- `src/retry.rs`: Retry logic tests

## Performance Considerations

### Zero-Copy Design
- Minimal memory allocations in hot paths
- Use of references and borrowing where possible
- Efficient weighted random selection for load balancing

### Async Architecture
- Built on Tokio runtime for non-blocking operations
- Uses `reqwest` for HTTP client operations
- All async operations are optional via feature flags

### Benchmarking
- Performance benchmarks available in `examples/performance_benchmark.rs`
- Results stored in `rust_performance_results.json`
- Target: <0.01ms routing decisions, <122KB binary size

## Development Environment Setup

### Prerequisites
- Rust 1.70+
- Node.js (for Node.js bindings)
- API keys for testing (OPENAI_API_KEY, ANTHROPIC_API_KEY)

### Environment Variables
```bash
export OPENAI_API_KEY="sk-your-openai-key"
export ANTHROPIC_API_KEY="sk-ant-your-anthropic-key"
```

### IDE Integration
The project follows standard Rust conventions and works well with rust-analyzer for IDE support.