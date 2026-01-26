---
title: Remote Cache Server Deployment
description: Complete guide to deploying rninja-cached
tags:
  - caching
  - remote
  - deployment
  - operations
---

# Remote Cache Server Deployment

Complete guide to deploying `rninja-cached` in production.

## Server Requirements

### Hardware

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| RAM | 2 GB | 8+ GB |
| Storage | 50 GB SSD | 200+ GB NVMe |
| Network | 1 Gbps | 10 Gbps |

### Software

- Linux (recommended), macOS, or Windows
- rninja package installed
- systemd (for service management)

## Basic Deployment

### Command Line

```bash
rninja-cached \
    --listen tcp://0.0.0.0:9999 \
    --storage /var/lib/rninja-cache \
    --tokens "token1,token2" \
    --max-size 100G
```

### Environment Variables

```bash
export RNINJA_SERVER_LISTEN=tcp://0.0.0.0:9999
export RNINJA_SERVER_STORAGE=/var/lib/rninja-cache
export RNINJA_SERVER_TOKENS=token1,token2
export RNINJA_SERVER_MAX_SIZE=100G

rninja-cached
```

## Systemd Service

### Create Service File

```ini title="/etc/systemd/system/rninja-cached.service"
[Unit]
Description=rninja Remote Cache Server
After=network.target

[Service]
Type=simple
User=rninja
Group=rninja
ExecStart=/usr/local/bin/rninja-cached \
    --listen tcp://0.0.0.0:9999 \
    --storage /var/lib/rninja-cache \
    --max-size 100G
Environment=RNINJA_SERVER_TOKENS=your-token-here
Restart=always
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/rninja-cache
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

### Set Up User and Directories

```bash
# Create user
sudo useradd -r -s /bin/false rninja

# Create directories
sudo mkdir -p /var/lib/rninja-cache
sudo chown rninja:rninja /var/lib/rninja-cache
```

### Enable and Start

```bash
sudo systemctl daemon-reload
sudo systemctl enable rninja-cached
sudo systemctl start rninja-cached

# Check status
sudo systemctl status rninja-cached
```

## Docker Deployment

### Dockerfile

```dockerfile title="Dockerfile"
FROM rust:1.75-slim as builder
RUN cargo install rninja

FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/rninja-cached /usr/local/bin/

VOLUME /data
EXPOSE 9999

ENV RNINJA_SERVER_LISTEN=tcp://0.0.0.0:9999
ENV RNINJA_SERVER_STORAGE=/data

ENTRYPOINT ["rninja-cached"]
```

### Docker Compose

```yaml title="docker-compose.yml"
version: '3.8'

services:
  rninja-cache:
    build: .
    ports:
      - "9999:9999"
    volumes:
      - cache-data:/data
    environment:
      - RNINJA_SERVER_TOKENS=${CACHE_TOKENS}
      - RNINJA_SERVER_MAX_SIZE=100G
    restart: unless-stopped

volumes:
  cache-data:
```

### Run with Docker

```bash
# Build and run
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

## Kubernetes Deployment

### Deployment

```yaml title="deployment.yaml"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rninja-cache
spec:
  replicas: 1
  selector:
    matchLabels:
      app: rninja-cache
  template:
    metadata:
      labels:
        app: rninja-cache
    spec:
      containers:
        - name: rninja-cached
          image: neullabs/rninja-cached:latest
          ports:
            - containerPort: 9999
          env:
            - name: RNINJA_SERVER_TOKENS
              valueFrom:
                secretKeyRef:
                  name: rninja-cache-secrets
                  key: tokens
            - name: RNINJA_SERVER_MAX_SIZE
              value: "100G"
          volumeMounts:
            - name: cache-storage
              mountPath: /data
          resources:
            requests:
              memory: "2Gi"
              cpu: "1"
            limits:
              memory: "8Gi"
              cpu: "4"
      volumes:
        - name: cache-storage
          persistentVolumeClaim:
            claimName: rninja-cache-pvc
```

### Service

```yaml title="service.yaml"
apiVersion: v1
kind: Service
metadata:
  name: rninja-cache
spec:
  selector:
    app: rninja-cache
  ports:
    - port: 9999
      targetPort: 9999
  type: ClusterIP
```

### PersistentVolumeClaim

```yaml title="pvc.yaml"
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: rninja-cache-pvc
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 200Gi
  storageClassName: fast-ssd
```

### Secret

```bash
kubectl create secret generic rninja-cache-secrets \
    --from-literal=tokens="token1,token2"
```

## Reverse Proxy with TLS

### Nginx Configuration

```nginx title="/etc/nginx/sites-available/rninja-cache"
upstream rninja_cache {
    server 127.0.0.1:9999;
}

server {
    listen 443 ssl;
    server_name cache.example.com;

    ssl_certificate /etc/ssl/certs/cache.example.com.crt;
    ssl_certificate_key /etc/ssl/private/cache.example.com.key;

    location / {
        proxy_pass http://rninja_cache;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_connect_timeout 60s;
        proxy_read_timeout 300s;
        client_max_body_size 100M;
    }
}
```

## Monitoring

### Health Check

```bash
# Simple connectivity check
nc -zv cache.example.com 9999
```

### Prometheus Metrics

(Coming in future version)

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'rninja-cache'
    static_configs:
      - targets: ['cache.example.com:9998']
```

## Backup Strategy

### Regular Backups

```bash
#!/bin/bash
# backup.sh - Run daily via cron

BACKUP_DIR=/backups/rninja-cache
DATE=$(date +%Y%m%d)

# Stop briefly for consistent backup
systemctl stop rninja-cached

# Backup
tar -czf $BACKUP_DIR/rninja-cache-$DATE.tar.gz -C /var/lib rninja-cache

# Restart
systemctl start rninja-cached

# Keep last 7 days
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete
```

### Disaster Recovery

```bash
# Restore from backup
systemctl stop rninja-cached
rm -rf /var/lib/rninja-cache
tar -xzf /backups/rninja-cache-20240101.tar.gz -C /var/lib
systemctl start rninja-cached
```

## Capacity Planning

### Storage Calculation

```
Storage needed = (Avg artifact size) × (Unique artifacts) × (Retention factor)

Example:
- Avg artifact: 500 KB
- Unique artifacts per day: 1000
- Retention: 30 days
- Storage = 500 KB × 1000 × 30 = ~15 GB

Add 3x buffer: 45-50 GB recommended
```

### Network Bandwidth

```
Bandwidth = (Artifacts per build) × (Avg size) × (Builds per hour)

Example:
- 100 artifacts per build
- 500 KB average
- 10 builds per hour
- Bandwidth = 100 × 500 KB × 10 = 500 MB/hour = ~1.1 Mbps sustained
```

## Security Checklist

- [ ] Use strong, unique tokens
- [ ] Enable TLS via reverse proxy
- [ ] Restrict network access (firewall)
- [ ] Run as non-root user
- [ ] Enable systemd security hardening
- [ ] Regular security updates
- [ ] Monitor access logs

## Troubleshooting Deployment

### Service Won't Start

```bash
# Check logs
journalctl -u rninja-cached -n 100

# Common issues:
# - Port already in use
# - Permission denied on storage
# - Invalid configuration
```

### Performance Issues

```bash
# Check disk I/O
iostat -x 1

# Check network
iftop

# Check memory
free -h
```

### Storage Full

```bash
# Check usage
df -h /var/lib/rninja-cache

# Run GC (if implemented)
# Or reduce max_size and restart
```
