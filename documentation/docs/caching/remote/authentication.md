---
title: Remote Cache Authentication
description: Configuring authentication for remote cache
tags:
  - caching
  - remote
  - security
  - authentication
---

# Remote Cache Authentication

Secure your remote cache with token-based authentication.

## Overview

rninja uses token-based authentication for remote cache access:

- Server maintains a list of valid tokens
- Clients present a token with each request
- Invalid tokens are rejected

## Server Configuration

### Setting Tokens

#### Command Line

```bash
rninja-cached \
    --listen tcp://0.0.0.0:9999 \
    --storage /var/lib/rninja-cache \
    --tokens "token1,token2,token3"
```

#### Environment Variable

```bash
export RNINJA_SERVER_TOKENS=token1,token2,token3
rninja-cached
```

### Generating Secure Tokens

```bash
# Generate random token
openssl rand -hex 32

# Or using /dev/urandom
head -c 32 /dev/urandom | base64

# Example output:
# a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef12345678
```

### Token Best Practices

1. **Use long, random tokens** (at least 32 characters)
2. **Unique tokens per use case** (team, CI, read-only)
3. **Rotate tokens periodically**
4. **Never share tokens in plain text**

## Client Configuration

### Setting the Token

```bash
export RNINJA_CACHE_TOKEN=your-secret-token
```

### Secure Token Storage

#### Environment File (Development)

```bash title="~/.rninja-env"
export RNINJA_CACHE_TOKEN=your-secret-token
```

```bash
# In ~/.bashrc
[ -f ~/.rninja-env ] && source ~/.rninja-env
```

Protect the file:

```bash
chmod 600 ~/.rninja-env
```

#### Credential Manager (Recommended)

```bash
# Store in system keyring
secret-tool store --label="rninja cache token" service rninja key cache_token

# Retrieve
export RNINJA_CACHE_TOKEN=$(secret-tool lookup service rninja key cache_token)
```

#### CI Secrets

Use your CI platform's secret management:

```yaml
# GitHub Actions
env:
  RNINJA_CACHE_TOKEN: ${{ secrets.CACHE_TOKEN }}

# GitLab CI
variables:
  RNINJA_CACHE_TOKEN: $CACHE_TOKEN  # From CI variables
```

## Token Strategies

### Single Token (Simple)

One token for everyone:

```bash
# Server
--tokens "team-shared-token"

# All clients
export RNINJA_CACHE_TOKEN=team-shared-token
```

**Pros:** Simple
**Cons:** Can't revoke individual access

### Per-Team Tokens

Different tokens per team:

```bash
# Server
--tokens "team-a-token,team-b-token,ci-token"

# Team A clients
export RNINJA_CACHE_TOKEN=team-a-token

# Team B clients
export RNINJA_CACHE_TOKEN=team-b-token

# CI systems
export RNINJA_CACHE_TOKEN=ci-token
```

**Pros:** Can revoke per-team
**Cons:** More tokens to manage

### Read/Write Separation

Different tokens for different access levels:

```bash
# Server
--tokens "write-token,read-token"

# CI (read/write)
export RNINJA_CACHE_TOKEN=write-token
export RNINJA_CACHE_PUSH_POLICY=always

# Developers (read-only)
export RNINJA_CACHE_TOKEN=read-token
export RNINJA_CACHE_PUSH_POLICY=never
```

!!! note "Access Control"
    Currently, all valid tokens have the same permissions. Use push/pull policies on clients for access control.

## Token Rotation

### Rotation Process

1. **Add new token** to server:
   ```bash
   --tokens "old-token,new-token"
   ```

2. **Update clients** to use new token:
   ```bash
   export RNINJA_CACHE_TOKEN=new-token
   ```

3. **Remove old token** after all clients updated:
   ```bash
   --tokens "new-token"
   ```

### Automated Rotation Script

```bash
#!/bin/bash
# rotate_token.sh

# Generate new token
NEW_TOKEN=$(openssl rand -hex 32)

# Update server config
sed -i "s/RNINJA_SERVER_TOKENS=.*/RNINJA_SERVER_TOKENS=old-token,$NEW_TOKEN/" /etc/rninja/env

# Restart server
systemctl restart rninja-cached

echo "New token: $NEW_TOKEN"
echo "Update clients, then remove old-token from server config"
```

## Security Hardening

### Network Security

Combine authentication with network controls:

```bash
# Firewall: Only allow internal network
sudo ufw allow from 10.0.0.0/8 to any port 9999
```

### TLS Encryption

Use a reverse proxy for TLS:

```nginx
server {
    listen 443 ssl;
    server_name cache.example.com;

    ssl_certificate /etc/ssl/certs/cache.crt;
    ssl_certificate_key /etc/ssl/private/cache.key;

    location / {
        proxy_pass http://127.0.0.1:9999;
    }
}
```

Clients use HTTPS endpoint (via proxy):

```bash
# Note: Direct TLS not yet supported
# Use reverse proxy for encryption
```

### Audit Logging

Monitor authentication attempts:

```bash
# Server logs show authentication events
journalctl -u rninja-cached | grep -i auth
```

## Troubleshooting

### Authentication Failed

```bash
# Verify token is set
echo $RNINJA_CACHE_TOKEN | wc -c  # Check length

# Verify token matches server
# On server: check RNINJA_SERVER_TOKENS
# On client: check RNINJA_CACHE_TOKEN
```

### Token Not Being Sent

```bash
# Check environment
env | grep RNINJA_CACHE

# Ensure mode uses remote
export RNINJA_CACHE_MODE=auto
```

### Server Rejecting Valid Token

```bash
# Check for whitespace in token
echo "'$RNINJA_CACHE_TOKEN'" | cat -A

# Regenerate token without special characters
openssl rand -hex 32
```

## Example Configurations

### Small Team

```bash
# Server
RNINJA_SERVER_TOKENS=team-token-2024

# All members
RNINJA_CACHE_TOKEN=team-token-2024
```

### Enterprise

```bash
# Server
RNINJA_SERVER_TOKENS=eng-token,qa-token,ci-token,readonly-token

# Engineering
RNINJA_CACHE_TOKEN=eng-token

# QA
RNINJA_CACHE_TOKEN=qa-token

# CI/CD
RNINJA_CACHE_TOKEN=ci-token
RNINJA_CACHE_PUSH_POLICY=always

# External contractors (read-only)
RNINJA_CACHE_TOKEN=readonly-token
RNINJA_CACHE_PUSH_POLICY=never
```
