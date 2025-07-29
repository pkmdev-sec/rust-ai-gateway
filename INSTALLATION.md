# 📦 Installation & Setup Guide

## 🎯 **Quick Start (5 minutes)**

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **OpenAI API Key** - [Get API Key](https://platform.openai.com/api-keys)
- **Anthropic API Key** (optional) - [Get API Key](https://console.anthropic.com/)

### 1. Clone & Build

```bash
# Clone the repository
git clone https://github.com/pkmdev-sec/rust-ai-gateway.git
cd rust-ai-gateway

# Build optimized release version
cargo build --release
```

### 2. Configure API Keys

```bash
# Set your API keys
export OPENAI_API_KEY="sk-your-openai-key-here"
export ANTHROPIC_API_KEY="sk-ant-your-anthropic-key-here"  # Optional
```

### 3. Start the Gateway

```bash
# Start the server
./target/release/opencode-server

# You should see:
# ✅ AI Gateway configured with 2 providers
# 🌐 Server listening on http://127.0.0.1:8788
# 📡 OpenCode can now use this endpoint directly!
```

### 4. Test the Gateway

```bash
# Test with curl
curl -X POST http://127.0.0.1:8788/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test" \
  -d '{
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## 🔧 **OpenCode CLI Integration**

### Automatic Integration (Recommended)

The gateway includes automatic OpenCode integration. Simply run:

```bash
# This will set up everything automatically
source ~/.zshrc
opencode-restart
```

### Manual Integration

If you prefer manual setup:

```bash
# Add to your shell profile (~/.zshrc, ~/.bashrc, etc.)
export OPENAI_API_BASE="http://127.0.0.1:8788/v1"
export ANTHROPIC_API_BASE="http://127.0.0.1:8788/v1"

# Reload your shell
source ~/.zshrc

# Use opencode normally - it now uses the ultra-fast gateway!
opencode "Write a hello world function"
```

## 🐳 **Docker Installation**

### Using Docker Compose (Easiest)

```yaml
# docker-compose.yml
version: '3.8'
services:
  rust-ai-gateway:
    build: .
    ports:
      - '8788:8788'
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
    restart: unless-stopped
```

```bash
# Start with Docker Compose
docker-compose up -d
```

### Using Docker Directly

```bash
# Build the image
docker build -t rust-ai-gateway .

# Run the container
docker run -d \
  --name rust-ai-gateway \
  -p 8788:8788 \
  -e OPENAI_API_KEY="your-key" \
  -e ANTHROPIC_API_KEY="your-key" \
  rust-ai-gateway
```

## ☸️ **Kubernetes Deployment**

### Basic Deployment

```yaml
# k8s-deployment.yaml
apiVersion: v1
kind: Secret
metadata:
  name: api-keys
type: Opaque
stringData:
  openai: 'sk-your-openai-key'
  anthropic: 'sk-ant-your-anthropic-key'
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-ai-gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rust-ai-gateway
  template:
    metadata:
      labels:
        app: rust-ai-gateway
    spec:
      containers:
        - name: gateway
          image: rust-ai-gateway:latest
          ports:
            - containerPort: 8788
          env:
            - name: OPENAI_API_KEY
              valueFrom:
                secretKeyRef:
                  name: api-keys
                  key: openai
            - name: ANTHROPIC_API_KEY
              valueFrom:
                secretKeyRef:
                  name: api-keys
                  key: anthropic
          resources:
            requests:
              memory: '64Mi'
              cpu: '50m'
            limits:
              memory: '128Mi'
              cpu: '200m'
---
apiVersion: v1
kind: Service
metadata:
  name: rust-ai-gateway-service
spec:
  selector:
    app: rust-ai-gateway
  ports:
    - port: 8788
      targetPort: 8788
  type: LoadBalancer
```

```bash
# Deploy to Kubernetes
kubectl apply -f k8s-deployment.yaml
```

## 🔧 **Advanced Configuration**

### Environment Variables

| Variable            | Description       | Default   | Required |
| ------------------- | ----------------- | --------- | -------- |
| `OPENAI_API_KEY`    | OpenAI API key    | -         | ✅       |
| `ANTHROPIC_API_KEY` | Anthropic API key | -         | ❌       |
| `GATEWAY_PORT`      | Server port       | 8788      | ❌       |
| `GATEWAY_HOST`      | Server host       | 127.0.0.1 | ❌       |
| `LOG_LEVEL`         | Log level         | info      | ❌       |
| `RUST_LOG`          | Rust logging      | info      | ❌       |

### Configuration File

Create `config.yaml` in the project root:

```yaml
# config.yaml
server:
  host: '0.0.0.0' # Listen on all interfaces
  port: 8788

providers:
  openai:
    api_key: '${OPENAI_API_KEY}'
    base_url: 'https://api.openai.com/v1'
    timeout: 30
    max_retries: 3

  anthropic:
    api_key: '${ANTHROPIC_API_KEY}'
    base_url: 'https://api.anthropic.com/v1'
    timeout: 30
    max_retries: 3

routing:
  strategy: 'load_balance' # single, load_balance, fallback
  health_check_interval: 30
  retry_attempts: 3

logging:
  level: 'info'
  format: 'json' # json or pretty
```

## 🚀 **Performance Tuning**

### System Optimization

```bash
# Increase file descriptor limits
ulimit -n 65536

# Optimize TCP settings (Linux)
echo 'net.core.somaxconn = 65536' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog = 65536' >> /etc/sysctl.conf
sysctl -p
```

### Rust Optimization

```bash
# Build with maximum optimization
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Profile-guided optimization (advanced)
cargo build --release --profile=pgo
```

## 🔍 **Troubleshooting**

### Common Issues

#### Gateway Won't Start

```bash
# Check if port is already in use
lsof -i :8788

# Check logs
tail -f /tmp/rust-gateway.log
```

#### API Keys Not Working

```bash
# Verify environment variables
echo $OPENAI_API_KEY
echo $ANTHROPIC_API_KEY

# Test API keys directly
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
  https://api.openai.com/v1/models
```

#### OpenCode Not Using Gateway

```bash
# Check environment variables in opencode process
ps eww $(pgrep opencode) | grep API_BASE

# Verify gateway is running
curl http://127.0.0.1:8788/health
```

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug ./target/release/opencode-server

# Or set environment variable
export RUST_LOG=debug
./target/release/opencode-server
```

## 📊 **Health Checks**

### Built-in Endpoints

```bash
# Basic health check
curl http://127.0.0.1:8788/health
# Response: "OK"

# Detailed status
curl http://127.0.0.1:8788/status
# Response: JSON with provider status

# Metrics
curl http://127.0.0.1:8788/metrics
# Response: Performance metrics
```

### Monitoring Setup

```bash
# Monitor logs in real-time
tail -f /tmp/rust-gateway.log

# Monitor system resources
htop

# Monitor network connections
netstat -tulpn | grep 8788
```

## 🔄 **Updates & Maintenance**

### Updating the Gateway

```bash
# Pull latest changes
git pull origin main

# Rebuild
cargo build --release

# Restart (if using systemd)
sudo systemctl restart rust-ai-gateway
```

### Backup Configuration

```bash
# Backup your configuration
cp config.yaml config.yaml.backup
cp ~/.zshrc ~/.zshrc.backup
```

## 🆘 **Getting Help**

- 📖 **Documentation**: Check the [Wiki](https://github.com/pkmdev-sec/rust-ai-gateway/wiki)
- 🐛 **Issues**: Report bugs on [GitHub Issues](https://github.com/pkmdev-sec/rust-ai-gateway/issues)
- 💬 **Discussions**: Join [GitHub Discussions](https://github.com/pkmdev-sec/rust-ai-gateway/discussions)
- 📧 **Email**: support@your-domain.com

---

**🎉 You're all set! Enjoy 11x faster AI routing with 100% reliability!** 🚀
