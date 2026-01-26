---
title: Security Overview
description: Security considerations for rninja
tags:
  - operations
  - security
---

# Security Overview

Security best practices for rninja deployments.

## Threat Model

| Threat | Mitigation |
|--------|------------|
| Unauthorized cache access | Token authentication |
| Data in transit | TLS via reverse proxy |
| Cache poisoning | Content-addressed hashing |
| Credential exposure | Secret management |

## Key Security Areas

### Authentication

- Token-based access control
- Per-team/user tokens
- Regular token rotation

[Details](authentication.md)

### Transport Security

- TLS termination
- Network segmentation
- Firewall rules

[Details](tls.md)

### Hardening

- Non-root execution
- Minimal permissions
- systemd restrictions

[Details](hardening.md)

## Security Checklist

- [ ] Strong tokens (32+ chars)
- [ ] TLS for external access
- [ ] Firewall restricts access
- [ ] Run as non-root user
- [ ] Tokens in secret management
- [ ] Regular token rotation
- [ ] Monitor access logs
