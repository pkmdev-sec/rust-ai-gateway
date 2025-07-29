# 🚀 Deployment Guide

## 📋 **Overview**

This guide covers production deployment of the Rust AI Gateway across various environments, from single-server setups to enterprise Kubernetes clusters.

## 🏗️ **Architecture Options**

### 1. **Single Server Deployment**

```
┌─────────────────┐
│   Load Balancer │
└─────────┬───────┘
          │
┌─────────▼───────┐
│  Rust Gateway   │
│   (Port 8788)   │
└─────────────────┘
```

### 2. **High Availability Setup**

```
┌─────────────────┐
│   Load Balancer │
└─────────┬───────┘
          │
    ┌─────┴─────┐
    │           │
┌───▼───┐   ┌───▼───┐
│Gateway│   │Gateway│
│   #1  │   │   #2  │
└───────┘   └───────┘
```

### 3. **Enterprise Kubernetes**

```
┌─────────────────┐
│     Ingress     │
└─────────┬───────┘
          │
┌─────────▼───────┐
│    Service      │
└─────────┬───────┘
          │
    ┌─────┴─────┐
    │           │
┌───▼───┐   ┌───▼───┐   ┌───────┐
│ Pod 1 │   │ Pod 2 │   │ Pod N │
└───────┘   └───────┘   └───────┘
```

## 🐳 **Docker Deployment**

### Dockerfile

```dockerfile
# Multi-stage build for optimal size
FROM rust:1.70-slim as builder

WORKDIR /app
COPY . .

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build optimized release
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false gateway

# Copy binary
COPY --from=builder /app/target/release/opencode-server /usr/local/bin/

# Set ownership
RUN chown gateway:gateway /usr/local/bin/opencode-server

# Switch to non-root user
USER gateway

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8788/health || exit 1

EXPOSE 8788

CMD ["opencode-server"]
```

### Docker Compose

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
      - RUST_LOG=info
    volumes:
      - ./config.yaml:/app/config.yaml:ro
    restart: unless-stopped
    healthcheck:
      test: ['CMD', 'curl', '-f', 'http://localhost:8788/health']
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    deploy:
      resources:
        limits:
          memory: 128M
          cpus: '0.5'
        reservations:
          memory: 64M
          cpus: '0.25'

  # Optional: Nginx reverse proxy
  nginx:
    image: nginx:alpine
    ports:
      - '80:80'
      - '443:443'
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - rust-ai-gateway
    restart: unless-stopped

volumes:
  gateway_data:
```

### Build and Deploy

```bash
# Build the image
docker build -t rust-ai-gateway:latest .

# Run with Docker Compose
docker-compose up -d

# Check status
docker-compose ps
docker-compose logs rust-ai-gateway
```

## ☸️ **Kubernetes Deployment**

### Namespace

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: ai-gateway
  labels:
    name: ai-gateway
```

### ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: gateway-config
  namespace: ai-gateway
data:
  config.yaml: |
    server:
      host: "0.0.0.0"
      port: 8788

    providers:
      openai:
        api_key: "${OPENAI_API_KEY}"
        base_url: "https://api.openai.com/v1"
        timeout: 30
        
      anthropic:
        api_key: "${ANTHROPIC_API_KEY}"
        base_url: "https://api.anthropic.com/v1"
        timeout: 30

    routing:
      strategy: "load_balance"
      health_check_interval: 30
      retry_attempts: 3

    logging:
      level: "info"
```

### Secret

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: api-keys
  namespace: ai-gateway
type: Opaque
stringData:
  openai: 'sk-your-openai-key-here'
  anthropic: 'sk-ant-your-anthropic-key-here'
```

### Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-ai-gateway
  namespace: ai-gateway
  labels:
    app: rust-ai-gateway
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: rust-ai-gateway
  template:
    metadata:
      labels:
        app: rust-ai-gateway
      annotations:
        prometheus.io/scrape: 'true'
        prometheus.io/port: '8788'
        prometheus.io/path: '/metrics'
    spec:
      containers:
        - name: gateway
          image: rust-ai-gateway:latest
          ports:
            - containerPort: 8788
              name: http
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
            - name: RUST_LOG
              value: 'info'
          volumeMounts:
            - name: config
              mountPath: /app/config.yaml
              subPath: config.yaml
          resources:
            requests:
              memory: '64Mi'
              cpu: '50m'
            limits:
              memory: '128Mi'
              cpu: '200m'
          livenessProbe:
            httpGet:
              path: /health
              port: 8788
            initialDelaySeconds: 30
            periodSeconds: 10
            timeoutSeconds: 5
            failureThreshold: 3
          readinessProbe:
            httpGet:
              path: /health
              port: 8788
            initialDelaySeconds: 5
            periodSeconds: 5
            timeoutSeconds: 3
            failureThreshold: 2
          securityContext:
            allowPrivilegeEscalation: false
            runAsNonRoot: true
            runAsUser: 1000
            capabilities:
              drop:
                - ALL
      volumes:
        - name: config
          configMap:
            name: gateway-config
      securityContext:
        fsGroup: 1000
```

### Service

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: rust-ai-gateway-service
  namespace: ai-gateway
  labels:
    app: rust-ai-gateway
spec:
  selector:
    app: rust-ai-gateway
  ports:
    - port: 8788
      targetPort: 8788
      protocol: TCP
      name: http
  type: ClusterIP
```

### Ingress

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: rust-ai-gateway-ingress
  namespace: ai-gateway
  annotations:
    kubernetes.io/ingress.class: 'nginx'
    nginx.ingress.kubernetes.io/ssl-redirect: 'true'
    nginx.ingress.kubernetes.io/proxy-body-size: '10m'
    nginx.ingress.kubernetes.io/proxy-read-timeout: '300'
    nginx.ingress.kubernetes.io/proxy-send-timeout: '300'
    cert-manager.io/cluster-issuer: 'letsencrypt-prod'
spec:
  tls:
    - hosts:
        - ai-gateway.yourdomain.com
      secretName: gateway-tls
  rules:
    - host: ai-gateway.yourdomain.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: rust-ai-gateway-service
                port:
                  number: 8788
```

### HorizontalPodAutoscaler

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: rust-ai-gateway-hpa
  namespace: ai-gateway
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: rust-ai-gateway
  minReplicas: 3
  maxReplicas: 20
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
        - type: Percent
          value: 100
          periodSeconds: 15
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
```

### Deploy to Kubernetes

```bash
# Apply all manifests
kubectl apply -f namespace.yaml
kubectl apply -f secret.yaml
kubectl apply -f configmap.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f ingress.yaml
kubectl apply -f hpa.yaml

# Check deployment status
kubectl get pods -n ai-gateway
kubectl get svc -n ai-gateway
kubectl get ingress -n ai-gateway

# View logs
kubectl logs -f deployment/rust-ai-gateway -n ai-gateway
```

## 🌐 **Cloud Provider Deployments**

### AWS ECS

```json
{
  "family": "rust-ai-gateway",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "256",
  "memory": "512",
  "executionRoleArn": "arn:aws:iam::account:role/ecsTaskExecutionRole",
  "taskRoleArn": "arn:aws:iam::account:role/ecsTaskRole",
  "containerDefinitions": [
    {
      "name": "rust-ai-gateway",
      "image": "your-account.dkr.ecr.region.amazonaws.com/rust-ai-gateway:latest",
      "portMappings": [
        {
          "containerPort": 8788,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "secrets": [
        {
          "name": "OPENAI_API_KEY",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:openai-key"
        },
        {
          "name": "ANTHROPIC_API_KEY",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:anthropic-key"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/rust-ai-gateway",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "ecs"
        }
      },
      "healthCheck": {
        "command": [
          "CMD-SHELL",
          "curl -f http://localhost:8788/health || exit 1"
        ],
        "interval": 30,
        "timeout": 5,
        "retries": 3,
        "startPeriod": 60
      }
    }
  ]
}
```

### Google Cloud Run

```yaml
# cloudrun.yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: rust-ai-gateway
  annotations:
    run.googleapis.com/ingress: all
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/maxScale: '100'
        run.googleapis.com/cpu-throttling: 'false'
        run.googleapis.com/execution-environment: gen2
    spec:
      containerConcurrency: 1000
      timeoutSeconds: 300
      containers:
        - image: gcr.io/your-project/rust-ai-gateway:latest
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
            limits:
              cpu: '1'
              memory: '512Mi'
          livenessProbe:
            httpGet:
              path: /health
              port: 8788
            initialDelaySeconds: 10
            periodSeconds: 10
```

### Azure Container Instances

```yaml
# azure-container.yaml
apiVersion: 2021-03-01
location: eastus
name: rust-ai-gateway
properties:
  containers:
    - name: rust-ai-gateway
      properties:
        image: your-registry.azurecr.io/rust-ai-gateway:latest
        ports:
          - port: 8788
            protocol: TCP
        environmentVariables:
          - name: RUST_LOG
            value: info
          - name: OPENAI_API_KEY
            secureValue: your-openai-key
          - name: ANTHROPIC_API_KEY
            secureValue: your-anthropic-key
        resources:
          requests:
            cpu: 0.5
            memoryInGb: 0.5
          limits:
            cpu: 1
            memoryInGb: 1
  osType: Linux
  restartPolicy: Always
  ipAddress:
    type: Public
    ports:
      - protocol: TCP
        port: 8788
tags:
  environment: production
  service: ai-gateway
```

## 🔧 **Configuration Management**

### Environment-Specific Configs

#### Development

```yaml
# config/development.yaml
server:
  host: '127.0.0.1'
  port: 8788

providers:
  openai:
    api_key: '${OPENAI_API_KEY}'
    timeout: 10

logging:
  level: 'debug'
  format: 'pretty'
```

#### Staging

```yaml
# config/staging.yaml
server:
  host: '0.0.0.0'
  port: 8788

providers:
  openai:
    api_key: '${OPENAI_API_KEY}'
    timeout: 20
  anthropic:
    api_key: '${ANTHROPIC_API_KEY}'
    timeout: 20

routing:
  strategy: 'load_balance'

logging:
  level: 'info'
  format: 'json'
```

#### Production

```yaml
# config/production.yaml
server:
  host: '0.0.0.0'
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
  strategy: 'load_balance'
  health_check_interval: 30
  retry_attempts: 3

logging:
  level: 'warn'
  format: 'json'

security:
  rate_limit: 1000
  timeout: 30
```

## 📊 **Monitoring & Observability**

### Prometheus Metrics

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'rust-ai-gateway'
    static_configs:
      - targets: ['localhost:8788']
    metrics_path: /metrics
    scrape_interval: 10s
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Rust AI Gateway",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(gateway_requests_total[5m])",
            "legendFormat": "{{provider}}"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(gateway_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "singlestat",
        "targets": [
          {
            "expr": "rate(gateway_requests_total{status=~\"5..\"}[5m]) / rate(gateway_requests_total[5m])",
            "legendFormat": "Error Rate"
          }
        ]
      }
    ]
  }
}
```

### Logging Configuration

```yaml
# logging.yaml
version: 1
formatters:
  json:
    format: '{"timestamp": "%(asctime)s", "level": "%(levelname)s", "message": "%(message)s"}'

handlers:
  console:
    class: logging.StreamHandler
    formatter: json

  file:
    class: logging.handlers.RotatingFileHandler
    filename: /var/log/gateway.log
    maxBytes: 10485760
    backupCount: 5
    formatter: json

loggers:
  rust_ai_gateway:
    level: INFO
    handlers: [console, file]
    propagate: false

root:
  level: INFO
  handlers: [console]
```

## 🔒 **Security Hardening**

### TLS Configuration

```nginx
# nginx-ssl.conf
server {
    listen 443 ssl http2;
    server_name ai-gateway.yourdomain.com;

    ssl_certificate /etc/ssl/certs/gateway.crt;
    ssl_certificate_key /etc/ssl/private/gateway.key;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;

    location / {
        proxy_pass http://127.0.0.1:8788;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Firewall Rules

```bash
# UFW rules
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow from 10.0.0.0/8 to any port 8788  # Internal only
ufw enable
```

### Container Security

```dockerfile
# Security-hardened Dockerfile
FROM rust:1.70-slim as builder
# ... build steps ...

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/opencode-server /
USER 65534:65534
EXPOSE 8788
ENTRYPOINT ["/opencode-server"]
```

## 🚀 **Performance Tuning**

### System Optimization

```bash
# /etc/sysctl.conf
net.core.somaxconn = 65536
net.ipv4.tcp_max_syn_backlog = 65536
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 1200
net.ipv4.tcp_max_tw_buckets = 1440000
vm.swappiness = 10
```

### Container Limits

```yaml
resources:
  requests:
    memory: '64Mi'
    cpu: '50m'
  limits:
    memory: '128Mi'
    cpu: '200m'
```

## 📋 **Deployment Checklist**

### Pre-Deployment

- [ ] API keys configured and tested
- [ ] Configuration files validated
- [ ] SSL certificates installed
- [ ] Monitoring setup configured
- [ ] Backup procedures in place
- [ ] Security hardening applied

### Deployment

- [ ] Build and test Docker image
- [ ] Deploy to staging environment
- [ ] Run integration tests
- [ ] Performance testing completed
- [ ] Deploy to production
- [ ] Verify health checks

### Post-Deployment

- [ ] Monitor metrics and logs
- [ ] Verify API functionality
- [ ] Check error rates
- [ ] Validate performance
- [ ] Update documentation
- [ ] Notify stakeholders

---

**🎉 Your Rust AI Gateway is now production-ready with enterprise-grade deployment!** 🚀
