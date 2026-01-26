---
title: IPC Protocol
description: rninja daemon IPC protocol reference
tags:
  - reference
  - api
---

# IPC Protocol

rninja daemon inter-process communication protocol.

## Overview

The daemon uses Unix domain sockets (Linux/macOS) or named pipes (Windows) for IPC with the CLI.

## Transport

### Unix Domain Socket

```
/tmp/rninja-{user}-{hash}.sock
```

Or custom path via `RNINJA_DAEMON_SOCKET`.

### Windows Named Pipe

```
\\.\pipe\rninja-{user}-{hash}
```

## Message Format

Messages use MessagePack serialization over a length-prefixed framing.

### Frame Format

```
+--------+----------------+
| Length | MessagePack    |
| 4 bytes| payload        |
| (BE)   | (variable)     |
+--------+----------------+
```

## Request Types

### Build Request

```javascript
{
  "type": "build",
  "id": "uuid",
  "working_dir": "/path/to/project",
  "build_file": "build.ninja",
  "targets": ["target1", "target2"],
  "jobs": 8,
  "keep_going": 1,
  "verbose": false,
  "explain": false,
  "cache_mode": "auto"
}
```

### Status Request

```javascript
{
  "type": "status",
  "id": "uuid"
}
```

### Cancel Request

```javascript
{
  "type": "cancel",
  "id": "uuid",
  "build_id": "build-uuid"
}
```

### Shutdown Request

```javascript
{
  "type": "shutdown",
  "id": "uuid",
  "graceful": true
}
```

## Response Types

### Build Started

```javascript
{
  "type": "build_started",
  "id": "uuid",
  "build_id": "build-uuid"
}
```

### Build Progress

```javascript
{
  "type": "progress",
  "id": "uuid",
  "build_id": "build-uuid",
  "completed": 50,
  "total": 100,
  "current_target": "src/foo.o",
  "running": ["src/bar.o", "src/baz.o"]
}
```

### Build Output

```javascript
{
  "type": "output",
  "id": "uuid",
  "build_id": "build-uuid",
  "stream": "stdout",  // or "stderr"
  "data": "base64-encoded-output"
}
```

### Build Complete

```javascript
{
  "type": "build_complete",
  "id": "uuid",
  "build_id": "build-uuid",
  "success": true,
  "exit_code": 0,
  "stats": {
    "total_targets": 100,
    "built": 50,
    "cached": 45,
    "failed": 0,
    "duration_ms": 5432
  }
}
```

### Status Response

```javascript
{
  "type": "status_response",
  "id": "uuid",
  "daemon_version": "0.1.0",
  "uptime_seconds": 3600,
  "active_builds": 2,
  "total_builds": 150,
  "memory_bytes": 52428800,
  "cache_stats": {
    "hits": 1234,
    "misses": 56
  }
}
```

### Error Response

```javascript
{
  "type": "error",
  "id": "uuid",
  "code": "build_failed",
  "message": "Build failed with 3 errors",
  "details": {
    "failed_targets": ["src/foo.o", "src/bar.o"]
  }
}
```

## Error Codes

| Code | Description |
|------|-------------|
| `invalid_request` | Malformed request |
| `build_failed` | Build completed with failures |
| `build_cancelled` | Build was cancelled |
| `not_found` | Build or target not found |
| `busy` | Daemon is at capacity |
| `internal_error` | Internal daemon error |

## Streaming Protocol

Build output is streamed as multiple `output` messages:

```
Client                          Daemon
  |                               |
  |-- build request ------------->|
  |<-- build_started -------------|
  |<-- progress (10/100) ---------|
  |<-- output (stdout) -----------|
  |<-- progress (20/100) ---------|
  |<-- output (stderr) -----------|
  |      ...                      |
  |<-- build_complete ------------|
  |                               |
```

## Connection Lifecycle

### Connect

1. Client connects to socket
2. Client sends request
3. Daemon processes and streams responses
4. Client receives `build_complete` or `error`
5. Connection can be reused or closed

### Heartbeat

Long-running connections use heartbeat:

```javascript
// Client sends periodically
{ "type": "ping", "id": "uuid" }

// Daemon responds
{ "type": "pong", "id": "uuid" }
```

### Disconnect

Client can disconnect anytime. Active builds continue in background unless cancelled.

## Concurrency

- Multiple clients can connect simultaneously
- Each build gets unique `build_id`
- Progress updates broadcast to all interested clients

## Example: Python Client

```python
import socket
import msgpack
import struct

def connect_daemon():
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.connect("/tmp/rninja-user-abc123.sock")
    return sock

def send_request(sock, request):
    data = msgpack.packb(request)
    sock.sendall(struct.pack(">I", len(data)) + data)

def recv_response(sock):
    length_data = sock.recv(4)
    length = struct.unpack(">I", length_data)[0]
    data = sock.recv(length)
    return msgpack.unpackb(data)

# Build request
sock = connect_daemon()
send_request(sock, {
    "type": "build",
    "id": "123",
    "working_dir": "/project",
    "targets": ["all"],
    "jobs": 8
})

# Receive responses
while True:
    response = recv_response(sock)
    if response["type"] == "build_complete":
        break
    elif response["type"] == "progress":
        print(f"Progress: {response['completed']}/{response['total']}")
    elif response["type"] == "output":
        print(response["data"])
```

## Versioning

Protocol version in status response:

```javascript
{
  "type": "status_response",
  "protocol_version": 1,
  ...
}
```

Clients should check version compatibility.
