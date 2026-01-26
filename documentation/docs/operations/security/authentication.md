---
title: Authentication
description: Configuring authentication for rninja
tags:
  - operations
  - security
---

# Authentication

Securing access to rninja cache.

## Token-Based Auth

### Server Configuration

```bash
rninja-cached --tokens "token1,token2,token3"
```

Or environment:

```bash
export RNINJA_SERVER_TOKENS=token1,token2
```

### Client Configuration

```bash
export RNINJA_CACHE_TOKEN=token1
```

## Token Best Practices

### Generate Strong Tokens

```bash
openssl rand -hex 32
```

### Token Organization

| Token | Use |
|-------|-----|
| `team-dev-xxx` | Development team |
| `ci-prod-xxx` | CI pipelines |
| `readonly-xxx` | Read-only access |

### Token Rotation

1. Add new token to server
2. Update clients to use new token
3. Remove old token

```bash
# Server: add new token
--tokens "old-token,new-token"

# Update clients
export RNINJA_CACHE_TOKEN=new-token

# Remove old token
--tokens "new-token"
```

## Secret Management

### CI Secrets

```yaml
# GitHub Actions
env:
  RNINJA_CACHE_TOKEN: ${{ secrets.CACHE_TOKEN }}
```

### Environment Files

```bash
# Secure file
chmod 600 ~/.rninja-token
echo "export RNINJA_CACHE_TOKEN=xxx" > ~/.rninja-token
```
