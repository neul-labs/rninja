---
title: TLS Configuration
description: Securing rninja with TLS
tags:
  - operations
  - security
---

# TLS Configuration

Encrypting rninja traffic with TLS.

## Reverse Proxy Setup

rninja-cached doesn't handle TLS directly. Use a reverse proxy:

### Nginx

```nginx
server {
    listen 443 ssl;
    server_name cache.example.com;

    ssl_certificate /etc/ssl/certs/cache.crt;
    ssl_certificate_key /etc/ssl/private/cache.key;
    ssl_protocols TLSv1.2 TLSv1.3;

    location / {
        proxy_pass http://127.0.0.1:9999;
        proxy_connect_timeout 60s;
        proxy_read_timeout 300s;
    }
}
```

### HAProxy

```haproxy
frontend cache_tls
    bind *:443 ssl crt /etc/ssl/cache.pem
    default_backend cache_backend

backend cache_backend
    server cache1 127.0.0.1:9999
```

## Certificate Management

### Let's Encrypt

```bash
certbot certonly --nginx -d cache.example.com
```

### Self-Signed (Testing)

```bash
openssl req -x509 -nodes -days 365 \
    -newkey rsa:2048 \
    -keyout cache.key \
    -out cache.crt
```

## Client Configuration

Clients connect through the TLS endpoint:

```bash
# Note: TCP transport, not HTTPS
# TLS termination happens at proxy
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.example.com:443
```
