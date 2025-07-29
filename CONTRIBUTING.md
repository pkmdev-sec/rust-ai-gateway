# 🤝 Contributing to Rust AI Gateway

Thank you for your interest in contributing to the Rust AI Gateway! This document provides guidelines and information for contributors.

## 🎯 **How to Contribute**

### 1. **Fork the Repository**

```bash
# Fork on GitHub, then clone your fork
git clone https://github.com/pkmdev-sec/rust-ai-gateway.git
cd rust-ai-gateway

# Add upstream remote
git remote add upstream https://github.com/original-owner/rust-ai-gateway.git
```

### 2. **Set Up Development Environment**

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development dependencies
rustup component add rustfmt clippy

# Build the project
cargo build

# Run tests
cargo test
```

### 3. **Create a Feature Branch**

```bash
# Create and switch to a new branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-description
```

## 📋 **Development Guidelines**

### Code Style

- **Use `rustfmt`** for consistent formatting:

  ```bash
  cargo fmt
  ```

- **Use `clippy`** for linting:

  ```bash
  cargo clippy -- -D warnings
  ```

- **Follow Rust naming conventions**:
  - `snake_case` for functions and variables
  - `PascalCase` for types and structs
  - `SCREAMING_SNAKE_CASE` for constants

### Code Quality

- **Write tests** for new functionality
- **Update documentation** for public APIs
- **Add examples** for new features
- **Ensure backward compatibility** when possible

### Performance

- **Benchmark critical paths** using `cargo bench`
- **Profile memory usage** for new features
- **Avoid unnecessary allocations** in hot paths
- **Use `Arc` and `Rc` judiciously** for shared data

## 🧪 **Testing**

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let config = RouterConfig::default();
        let router = Router::new(config);
        assert!(router.is_ok());
    }

    #[tokio::test]
    async fn test_async_routing() {
        let router = create_test_router().await;
        let result = router.route(&RouterContext::new()).await;
        assert!(result.is_ok());
    }
}
```

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench routing_performance
```

## 📚 **Documentation**

### Code Documentation

- **Document all public APIs** with `///` comments
- **Include examples** in documentation
- **Use `cargo doc`** to generate documentation

````rust
/// Routes requests to the appropriate AI provider based on configuration.
///
/// # Arguments
///
/// * `context` - The routing context containing request metadata
///
/// # Returns
///
/// Returns a `Result` containing the selected target or an error.
///
/// # Examples
///
/// ```rust
/// use ai_gateway_router::{Router, RouterContext};
///
/// let router = Router::new(config)?;
/// let context = RouterContext::new();
/// let target = router.route(&context)?;
/// ```
pub fn route(&self, context: &RouterContext) -> Result<Target, RouterError> {
    // Implementation
}
````

### README Updates

- **Update feature lists** for new capabilities
- **Add usage examples** for new functionality
- **Update performance benchmarks** if applicable

## 🐛 **Bug Reports**

### Before Reporting

1. **Search existing issues** to avoid duplicates
2. **Test with the latest version**
3. **Reproduce the issue** with minimal code

### Bug Report Template

```markdown
## Bug Description

Brief description of the issue.

## Steps to Reproduce

1. Step one
2. Step two
3. Step three

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened.

## Environment

- OS: [e.g., macOS 14.0]
- Rust version: [e.g., 1.70.0]
- Gateway version: [e.g., 1.0.0]

## Additional Context

Any other relevant information.
```

## 💡 **Feature Requests**

### Feature Request Template

```markdown
## Feature Description

Clear description of the proposed feature.

## Use Case

Why is this feature needed? What problem does it solve?

## Proposed Implementation

High-level approach to implementing the feature.

## Alternatives Considered

Other approaches you've considered.

## Additional Context

Any other relevant information.
```

## 🔄 **Pull Request Process**

### Before Submitting

1. **Ensure tests pass**: `cargo test`
2. **Format code**: `cargo fmt`
3. **Lint code**: `cargo clippy`
4. **Update documentation** if needed
5. **Add changelog entry** if applicable

### Pull Request Template

```markdown
## Description

Brief description of changes.

## Type of Change

- [ ] Bug fix (non-breaking change)
- [ ] New feature (non-breaking change)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing

- [ ] Tests pass locally
- [ ] New tests added for new functionality
- [ ] Manual testing completed

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings introduced
```

### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by maintainers
3. **Testing** in staging environment
4. **Approval** from at least one maintainer
5. **Merge** into main branch

## 🏗️ **Architecture Guidelines**

### Project Structure

```
src/
├── lib.rs              # Main library entry point
├── router.rs           # Core routing logic
├── config.rs           # Configuration handling
├── error.rs            # Error types and handling
├── strategy.rs         # Routing strategies
├── retry.rs            # Retry logic
├── guardrails.rs       # Content filtering
├── multimodal.rs       # Multi-modal support
└── bin/
    └── opencode-server.rs  # HTTP server binary

examples/               # Usage examples
tests/                 # Integration tests
benches/              # Performance benchmarks
```

### Design Principles

1. **Performance First**: Optimize for speed and efficiency
2. **Type Safety**: Leverage Rust's type system
3. **Error Handling**: Use `Result` types consistently
4. **Modularity**: Keep components loosely coupled
5. **Testability**: Design for easy testing

### Adding New Providers

```rust
// 1. Add provider configuration
#[derive(Debug, Clone)]
pub struct NewProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
}

// 2. Implement provider trait
impl Provider for NewProvider {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        // Implementation
    }
}

// 3. Add to router builder
impl RouterBuilder {
    pub fn with_new_provider(mut self, config: NewProviderConfig) -> Self {
        self.providers.insert("new_provider".to_string(), Box::new(NewProvider::new(config)));
        self
    }
}
```

## 🚀 **Release Process**

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Create release tag
- [ ] Publish to crates.io (if applicable)

## 📞 **Getting Help**

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Email**: maintainer@rust-ai-gateway.com

### Code of Conduct

Please note that this project is released with a [Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

## 🙏 **Recognition**

Contributors will be recognized in:

- **README.md** contributors section
- **CHANGELOG.md** for significant contributions
- **GitHub releases** notes

Thank you for contributing to making AI routing faster and more reliable! 🚀
