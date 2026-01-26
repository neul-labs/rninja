---
title: Docker Integration
description: Using rninja with Docker builds
tags:
  - ci-cd
  - docker
---

# Docker Integration

Using rninja in Docker-based builds.

## Basic Dockerfile

```dockerfile title="Dockerfile"
FROM rust:1.75 as builder

# Install rninja
RUN cargo install rninja

# Copy source
COPY . /app
WORKDIR /app

# Build
RUN rninja

# Runtime image
FROM debian:bookworm-slim
COPY --from=builder /app/build/myapp /usr/local/bin/
CMD ["myapp"]
```

## With Build Cache

```dockerfile title="Dockerfile"
FROM rust:1.75 as builder

# Install rninja
RUN cargo install rninja

# Cache mount for rninja cache
WORKDIR /app
COPY . .

RUN --mount=type=cache,target=/root/.cache/rninja \
    rninja

FROM debian:bookworm-slim
COPY --from=builder /app/build/myapp /usr/local/bin/
CMD ["myapp"]
```

## Docker Compose

```yaml title="docker-compose.yml"
version: '3.8'

services:
  build:
    build: .
    volumes:
      - rninja-cache:/root/.cache/rninja
    environment:
      - RNINJA_CACHE_ENABLED=1

volumes:
  rninja-cache:
```

## Multi-Stage CMake Build

```dockerfile title="Dockerfile"
FROM ubuntu:22.04 as builder

RUN apt-get update && apt-get install -y \
    cmake ninja-build build-essential curl

# Install Rust and rninja
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install rninja

WORKDIR /app
COPY . .

RUN cmake -G Ninja -B build -DCMAKE_BUILD_TYPE=Release
RUN rninja -C build

FROM debian:bookworm-slim
COPY --from=builder /app/build/bin/* /usr/local/bin/
```

## Remote Cache in Docker

```dockerfile title="Dockerfile"
FROM rust:1.75 as builder

ARG CACHE_SERVER
ARG CACHE_TOKEN

ENV RNINJA_CACHE_REMOTE_SERVER=$CACHE_SERVER
ENV RNINJA_CACHE_TOKEN=$CACHE_TOKEN
ENV RNINJA_CACHE_MODE=auto

RUN cargo install rninja

COPY . /app
WORKDIR /app
RUN rninja
```

Build with:

```bash
docker build \
  --build-arg CACHE_SERVER=tcp://cache:9999 \
  --build-arg CACHE_TOKEN=secret \
  -t myapp .
```

## BuildKit Cache

```dockerfile title="Dockerfile"
# syntax=docker/dockerfile:1.4

FROM rust:1.75 as builder

RUN cargo install rninja

WORKDIR /app
COPY . .

# Use BuildKit cache mount
RUN --mount=type=cache,target=/root/.cache/rninja \
    --mount=type=cache,target=/root/.cargo/registry \
    rninja
```

Build with BuildKit:

```bash
DOCKER_BUILDKIT=1 docker build -t myapp .
```
