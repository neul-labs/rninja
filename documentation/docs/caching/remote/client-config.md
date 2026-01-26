---
title: Remote Cache Client Configuration
description: Configuring rninja clients for remote caching
tags:
  - caching
  - remote
  - configuration
---

# Remote Cache Client Configuration

Configure rninja clients to use a remote cache server.

## Basic Configuration

### Environment Variables

```bash
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.example.com:9999
export RNINJA_CACHE_TOKEN=your-auth-token
export RNINJA_CACHE_MODE=auto
```

### Configuration File

```toml title="~/.config/rninja/config.toml"
[cache]
enabled = true
mode = "auto"

# Remote settings come from environment variables
# for security (don't put tokens in files)
```

## Connection Settings

### Server Address

```bash
# TCP connection
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.internal:9999

# With explicit port
export RNINJA_CACHE_REMOTE_SERVER=tcp://192.168.1.100:9999

# hostname
export RNINJA_CACHE_REMOTE_SERVER=tcp://rninja-cache.svc.cluster.local:9999
```

### Timeouts

```bash
# Connection timeout (seconds)
export RNINJA_CACHE_CONNECT_TIMEOUT=5

# Request timeout (seconds)
export RNINJA_CACHE_REQUEST_TIMEOUT=30
```

### Concurrency

```bash
# Maximum concurrent remote operations
export RNINJA_CACHE_MAX_CONCURRENT=4
```

## Cache Policies

### Push Policy

Control when artifacts are uploaded:

```bash
# Never push (read-only client)
export RNINJA_CACHE_PUSH_POLICY=never

# Push successful builds only (default)
export RNINJA_CACHE_PUSH_POLICY=on_success

# Always push
export RNINJA_CACHE_PUSH_POLICY=always
```

### Pull Policy

Control when artifacts are downloaded:

```bash
# Always check remote first (default)
export RNINJA_CACHE_PULL_POLICY=always

# Only check remote on local miss
export RNINJA_CACHE_PULL_POLICY=on_miss

# Never pull (push-only mode)
export RNINJA_CACHE_PULL_POLICY=never
```

## Configuration by Environment

### Development Machine

```bash
# ~/.bashrc

# Remote cache server
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.internal:9999
export RNINJA_CACHE_TOKEN=$TEAM_CACHE_TOKEN

# Use auto mode - remote with local fallback
export RNINJA_CACHE_MODE=auto

# Standard timeouts
export RNINJA_CACHE_CONNECT_TIMEOUT=5
export RNINJA_CACHE_REQUEST_TIMEOUT=30
```

### CI Runner

```bash
# CI environment

export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.internal:9999
export RNINJA_CACHE_TOKEN=$CI_CACHE_TOKEN
export RNINJA_CACHE_MODE=auto

# Push all successful builds
export RNINJA_CACHE_PUSH_POLICY=always

# Higher concurrency for faster CI
export RNINJA_CACHE_MAX_CONCURRENT=8
```

### Read-Only Client

For machines that shouldn't modify the cache:

```bash
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.internal:9999
export RNINJA_CACHE_TOKEN=$READ_ONLY_TOKEN
export RNINJA_CACHE_MODE=auto

# Never push
export RNINJA_CACHE_PUSH_POLICY=never
export RNINJA_CACHE_PULL_POLICY=always
```

### Offline Fallback

For laptops that may be offline:

```bash
export RNINJA_CACHE_REMOTE_SERVER=tcp://cache.internal:9999
export RNINJA_CACHE_TOKEN=$TEAM_TOKEN
export RNINJA_CACHE_MODE=auto  # Falls back to local

# Short timeouts for quick fallback
export RNINJA_CACHE_CONNECT_TIMEOUT=2
```

## Configuration Files

### Global Configuration

```toml title="~/.config/rninja/config.toml"
[cache]
enabled = true
mode = "auto"

# Local cache settings (used as fallback)
max_size = 5368709120  # 5GB
```

### Project Configuration

```toml title=".rninjarc"
[cache]
# Project can override mode
mode = "auto"
```

## CI Platform Examples

### GitHub Actions

```yaml title=".github/workflows/build.yml"
env:
  RNINJA_CACHE_REMOTE_SERVER: tcp://cache.example.com:9999
  RNINJA_CACHE_TOKEN: ${{ secrets.CACHE_TOKEN }}
  RNINJA_CACHE_MODE: auto
  RNINJA_CACHE_PUSH_POLICY: always

steps:
  - uses: actions/checkout@v4
  - name: Build
    run: rninja
```

### GitLab CI

```yaml title=".gitlab-ci.yml"
variables:
  RNINJA_CACHE_REMOTE_SERVER: tcp://cache.internal:9999
  RNINJA_CACHE_TOKEN: $CACHE_TOKEN
  RNINJA_CACHE_MODE: auto

build:
  script:
    - rninja
```

### Jenkins

```groovy title="Jenkinsfile"
pipeline {
    environment {
        RNINJA_CACHE_REMOTE_SERVER = 'tcp://cache.internal:9999'
        RNINJA_CACHE_TOKEN = credentials('rninja-cache-token')
        RNINJA_CACHE_MODE = 'auto'
    }
    stages {
        stage('Build') {
            steps {
                sh 'rninja'
            }
        }
    }
}
```

## Testing Configuration

### Verify Connection

```bash
# Check environment
env | grep RNINJA_CACHE

# Test with a build
rninja

# Check stats
rninja -t cache-stats
```

### Debug Connection Issues

```bash
# Test network connectivity
nc -zv cache.example.com 9999

# Verbose build
RUST_LOG=debug rninja
```

## Security Best Practices

### Token Management

```bash
# Don't put tokens in scripts
# BAD:
export RNINJA_CACHE_TOKEN=secret-token

# GOOD: Use environment from secure source
export RNINJA_CACHE_TOKEN=$(cat ~/.cache_token)
# Or
export RNINJA_CACHE_TOKEN=$TEAM_CACHE_TOKEN  # Set by admin
```

### Don't Commit Tokens

```bash
# .gitignore
.env
.cache_token
```

### Use CI Secrets

```yaml
# GitHub Actions
env:
  RNINJA_CACHE_TOKEN: ${{ secrets.CACHE_TOKEN }}
```

## Troubleshooting

### Connection Failed

```bash
# Check server address
echo $RNINJA_CACHE_REMOTE_SERVER

# Test connectivity
nc -zv <host> <port>

# Check firewall
```

### Authentication Failed

```bash
# Verify token
echo $RNINJA_CACHE_TOKEN | head -c 10  # Show first 10 chars

# Check with server admin for valid token
```

### Slow Performance

```bash
# Increase timeouts
export RNINJA_CACHE_REQUEST_TIMEOUT=60

# Check network latency
ping cache.example.com

# Consider local-first policy
export RNINJA_CACHE_PULL_POLICY=on_miss
```

### Fallback Not Working

```bash
# Ensure mode is 'auto'
export RNINJA_CACHE_MODE=auto

# Reduce connect timeout for faster fallback
export RNINJA_CACHE_CONNECT_TIMEOUT=2
```
