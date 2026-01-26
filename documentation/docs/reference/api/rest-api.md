---
title: REST API
description: rninja-cached REST API reference
tags:
  - reference
  - api
---

# REST API

rninja-cached server REST API reference.

!!! note "Optional Feature"
    REST API is an optional interface. The primary protocol is NNG/MessagePack for performance.

## Base URL

```
https://cache.example.com:9877/api/v1
```

## Authentication

All requests require authentication via bearer token:

```http
Authorization: Bearer <token>
```

Or via query parameter:

```
?token=<token>
```

## Endpoints

### Health Check

```http
GET /health
```

Response:

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 86400
}
```

### Cache Operations

#### Get Entry

```http
GET /cache/{key}
```

Parameters:

| Name | Type | Description |
|------|------|-------------|
| `key` | string | Cache key (64 hex chars) |

Response:

- `200 OK`: Entry found, body contains artifact
- `404 Not Found`: Entry not in cache
- `401 Unauthorized`: Invalid token

Headers:

```http
Content-Type: application/octet-stream
X-Cache-Created: 1609459200
X-Cache-Size: 12345
```

#### Put Entry

```http
PUT /cache/{key}
```

Parameters:

| Name | Type | Description |
|------|------|-------------|
| `key` | string | Cache key (64 hex chars) |

Request body: artifact data (binary)

Headers:

```http
Content-Type: application/octet-stream
Content-Length: 12345
```

Response:

- `201 Created`: Entry stored
- `409 Conflict`: Entry already exists
- `413 Payload Too Large`: Entry exceeds size limit
- `401 Unauthorized`: Invalid token

#### Delete Entry

```http
DELETE /cache/{key}
```

Response:

- `204 No Content`: Entry deleted
- `404 Not Found`: Entry not found
- `401 Unauthorized`: Invalid token

#### Check Entry

```http
HEAD /cache/{key}
```

Response:

- `200 OK`: Entry exists
- `404 Not Found`: Entry not found

Headers on success:

```http
X-Cache-Created: 1609459200
X-Cache-Size: 12345
```

### Statistics

#### Get Stats

```http
GET /stats
```

Response:

```json
{
  "total_entries": 12345,
  "total_size": 5368709120,
  "hits": 98765,
  "misses": 4321,
  "hit_rate": 0.958,
  "gets_per_second": 150.5,
  "puts_per_second": 25.3,
  "uptime_seconds": 86400
}
```

#### Get Detailed Stats

```http
GET /stats/detailed
```

Response:

```json
{
  "cache": {
    "entries": 12345,
    "size_bytes": 5368709120,
    "max_size_bytes": 107374182400
  },
  "operations": {
    "gets": 103086,
    "puts": 12345,
    "deletes": 123,
    "hits": 98765,
    "misses": 4321
  },
  "performance": {
    "avg_get_latency_ms": 5.2,
    "avg_put_latency_ms": 12.8,
    "p99_get_latency_ms": 25.0,
    "p99_put_latency_ms": 50.0
  },
  "connections": {
    "active": 42,
    "total": 98765
  }
}
```

### Administration

#### Trigger Garbage Collection

```http
POST /admin/gc
```

Request:

```json
{
  "max_age_seconds": 604800,
  "target_size_bytes": 53687091200
}
```

Response:

```json
{
  "entries_removed": 1234,
  "bytes_freed": 1073741824,
  "duration_ms": 5432
}
```

#### Clear Cache

```http
POST /admin/clear
```

!!! danger "Destructive"
    This removes all cached entries.

Response:

```json
{
  "entries_removed": 12345,
  "bytes_freed": 5368709120
}
```

#### Get Configuration

```http
GET /admin/config
```

Response:

```json
{
  "storage": {
    "backend": "filesystem",
    "path": "/var/cache/rninja",
    "max_size": "100G"
  },
  "auth": {
    "mode": "token"
  },
  "server": {
    "bind": "0.0.0.0:9877",
    "workers": 8
  }
}
```

## Error Responses

All errors return JSON:

```json
{
  "error": "not_found",
  "message": "Cache entry not found",
  "details": {
    "key": "abc123..."
  }
}
```

Error codes:

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `not_found` | 404 | Entry not found |
| `unauthorized` | 401 | Invalid or missing token |
| `forbidden` | 403 | Token lacks permission |
| `bad_request` | 400 | Invalid request |
| `conflict` | 409 | Entry already exists |
| `too_large` | 413 | Payload too large |
| `internal_error` | 500 | Server error |

## Rate Limiting

Responses include rate limit headers:

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1609459260
```

## Examples

### cURL

```bash
# Get entry
curl -H "Authorization: Bearer $TOKEN" \
  https://cache.example.com:9877/api/v1/cache/abc123...

# Put entry
curl -X PUT \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/octet-stream" \
  --data-binary @artifact.bin \
  https://cache.example.com:9877/api/v1/cache/abc123...

# Get stats
curl -H "Authorization: Bearer $TOKEN" \
  https://cache.example.com:9877/api/v1/stats
```

### Python

```python
import requests

BASE_URL = "https://cache.example.com:9877/api/v1"
TOKEN = "your-token"

headers = {"Authorization": f"Bearer {TOKEN}"}

# Get entry
response = requests.get(f"{BASE_URL}/cache/{key}", headers=headers)
if response.status_code == 200:
    artifact = response.content

# Put entry
response = requests.put(
    f"{BASE_URL}/cache/{key}",
    headers={**headers, "Content-Type": "application/octet-stream"},
    data=artifact_data
)

# Get stats
response = requests.get(f"{BASE_URL}/stats", headers=headers)
stats = response.json()
```

## OpenAPI Specification

Full OpenAPI 3.0 spec available at:

```
GET /openapi.json
GET /openapi.yaml
```
