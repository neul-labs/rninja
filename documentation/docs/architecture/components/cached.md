---
title: rninja-cached Architecture
description: Remote cache server - protocol, storage, configuration
tags:
  - architecture
---

# rninja-cached Architecture

`rninja-cached` is the **remote** cache server. It is optional: rninja
works fine with just the per-machine local cache. `rninja-cached` exists
to share the local-cache hit rate across an entire team or CI fleet.

The binary lives at
[`src/bin/rninja-cached.rs`](https://github.com/neul-labs/rninja/blob/main/src/bin/rninja-cached.rs)
and the server implementation lives under
[`src/server/`](https://github.com/neul-labs/rninja/tree/main/src/server).

## Module layout

| Module                | Purpose                                                  |
|-----------------------|----------------------------------------------------------|
| `server/mod.rs`       | Lifecycle (`run_server`) and module wiring               |
| `server/config.rs`    | `ServerConfig`, `AuthConfig`, TOML + env loading         |
| `server/handler.rs`   | Request dispatch (Get / Put / Delete / HasEntry / Stats) |
| `server/auth.rs`      | Bearer-token authentication                              |

## Configuration

`ServerConfig` (in `server/config.rs`) is loaded from a TOML file
(`--config`) or environment variables, with CLI flags overriding both.

| Field              | Default                          | Env var                      |
|--------------------|----------------------------------|------------------------------|
| `listen_addr`      | `tcp://0.0.0.0:9999`             | `RNINJA_SERVER_LISTEN`       |
| `storage_dir`      | XDG-aware, fallback `/var/cache/rninja-cached` | `RNINJA_SERVER_STORAGE` |
| `max_storage_size` | unlimited                        | `RNINJA_SERVER_MAX_SIZE`     |
| `auth.tokens`      | empty                            | `RNINJA_SERVER_TOKENS`       |
| `auth.require_auth`| `true`                           | -                            |
| `workers`          | `0` (auto)                       | -                            |
| `max_connections`  | `100`                            | -                            |
| `entry_ttl_secs`   | none                             | `RNINJA_SERVER_ENTRY_TTL`    |

`RNINJA_SERVER_MAX_SIZE` and `--max-size` accept `K`/`M`/`G` suffixes
(see `parse_size` in `server/config.rs`).

The server **refuses to start** if `auth.require_auth` is true and no
tokens are configured. This is a deliberate safety check from
[`src/bin/rninja-cached.rs`](https://github.com/neul-labs/rninja/blob/main/src/bin/rninja-cached.rs)
and not user-bypassable without explicitly disabling auth in the config
file.

## Protocol

NNG (`nng-rs`) with the Request/Reply pattern over TCP. The wire encoding
is MessagePack (`rmp-serde`). See
[Architecture > Network Protocol](../network-protocol.md) for the
request/response schema.

There is **no HTTP/REST surface** in the current codebase - all
communication is NNG/MessagePack for throughput. If you need scripted
interaction, drive the protocol directly via NNG client libraries.

## Storage

Content-addressed, with metadata in `sled` and blobs on disk:

```
$storage_dir/
├── index/      # sled database: key -> blob path, size, mtime
└── blobs/      # artifact files, named by their content hash
```

Keys are 64-character BLAKE3 hex digests. See
[Architecture > Cache > Hashing](../cache/hashing.md) for how keys are
computed on the client side.

## Authentication

Bearer tokens, configured statically via `--tokens`, `RNINJA_SERVER_TOKENS`,
or a TOML config. Every request must present a valid token unless
`auth.require_auth = false` (not recommended outside a private network).

Per [`memory`](https://github.com/neul-labs) preference, secrets are
**environment variables only** - there is no keyring or encrypted secrets
file in `rninja-cached`. Token rotation is "edit env, restart server".

## Capacity

The defaults are conservative; the hot path (NNG `Req`/`Rep` with
content-addressed lookups in `sled`) scales well on modern hardware. For
tuning guidance see
[Caching > Remote > Performance Tuning](../../caching/remote/tuning.md).

## Failure modes

- **Token misconfiguration**: explicit refuse-to-start (see above).
- **Storage full**: requests start failing with explicit errors once
  `max_storage_size` is reached; pair with
  [GC](../../operations/maintenance/garbage-collection.md).
- **Connection exhaustion**: capped at `max_connections`; excess clients
  see connection-refused.

## Related

- [Caching > Remote > Architecture](../../caching/remote/architecture.md) -
  client/server topology.
- [Caching > Remote > Deployment](../../caching/remote/deployment.md) -
  operational deployment guide.
- [Operations > Security > Authentication](../../operations/security/authentication.md) -
  token management.
