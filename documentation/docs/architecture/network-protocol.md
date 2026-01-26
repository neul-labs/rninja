---
title: Network Protocol
description: rninja network communication
tags:
  - architecture
---

# Network Protocol

Client-server communication protocol.

## Transport

Uses NNG (nanomsg next-gen):

- Request/Reply pattern
- TCP transport
- High performance

## Message Format

MessagePack serialization:

```
Request:
{
    "type": "get" | "put",
    "key": [32 bytes],
    "token": "auth-token",
    "data": [bytes]
}

Response:
{
    "status": "ok" | "not_found" | "error",
    "data": [bytes],
    "message": "error message"
}
```

## Operations

### GET (Lookup)

```
Client → Server: GET(key, token)
Server → Client: OK(data) | NOT_FOUND
```

### PUT (Store)

```
Client → Server: PUT(key, data, token)
Server → Client: OK | ERROR
```

## Performance

| Metric | Typical |
|--------|---------|
| Lookup latency | 5-50ms |
| Store latency | 10-100ms |
| Throughput | ~1000 req/s |
