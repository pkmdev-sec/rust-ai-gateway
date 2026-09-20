# AI Gateway Router

[![CI](https://github.com/pkmdev-sec/rust-ai-gateway/actions/workflows/ci.yml/badge.svg)](https://github.com/pkmdev-sec/rust-ai-gateway/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

AI Gateway Router is an experimental Rust library for selecting a configured model provider. It supports single-target, weighted load-balancing, fallback, and conditional routing.

The library returns the selected target. It does not proxy an LLM request unless a caller or optional server implementation performs that request.

## Status

Version 0.2.0 exposes the core routing library and its unit tests. Optional server binaries and legacy examples are not default Cargo targets because those paths have not completed release qualification.

Model identifiers, capabilities, prices, and provider limits in the registry are snapshots. Verify them against current provider documentation before making routing or billing decisions.

## Use from Git

The crate is not published on crates.io. Reference the repository explicitly:

```toml
[dependencies]
ai_gateway_router = { git = "https://github.com/pkmdev-sec/rust-ai-gateway", branch = "main" }
```

## Basic routing

```rust
use ai_gateway_router::{
    Router, RouterConfig, RouterContext, StrategyMode, Target,
};
use std::collections::HashMap;

let config = RouterConfig {
    mode: StrategyMode::LoadBalance,
    targets: vec![
        Target {
            name: "primary".to_string(),
            provider: "openai".to_string(),
            weight: Some(70),
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            metadata: HashMap::new(),
            retry_config: None,
            guardrails: None,
            model_capabilities: None,
            request_timeout_ms: None,
        },
        Target {
            name: "fallback".to_string(),
            provider: "anthropic".to_string(),
            weight: Some(30),
            api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
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
};

let router = Router::new(config);
let selected = router.route(&RouterContext::new())?;
println!("{} via {}", selected.name, selected.provider);
# Ok::<(), ai_gateway_router::RouterError>(())
```

## Library modules

- `router` and `strategy` implement target selection.
- `retry` defines retry and backoff policies.
- `guardrails` provides configurable input and output checks.
- `models` and `intelligent_router` contain the built-in model registry and heuristic recommendations.
- `multimodal` describes media capabilities used during selection.

These modules are routing components, not a security sandbox.

## Features

| Feature | Purpose |
|---|---|
| `async` | Tokio and Reqwest support |
| `simple` | Simplified builder API |
| `server` | Experimental Axum server dependencies |
| `nodejs` | N-API bindings |
| `full` | Enable every optional feature |

The default feature set is `async`, `simple`, and `uuid`.

## Credentials

Read provider keys from environment variables or a secret manager. Do not place working keys in source, examples, logs, or routing metadata. The library stores configured keys as ordinary Rust strings and does not provide encrypted storage.

## Development

```sh
cargo test --locked --lib
```

The default CI gate covers the core library. Optional servers, provider calls, and legacy examples require separate qualification.

## License

[MIT](LICENSE) © pkmdev-sec
