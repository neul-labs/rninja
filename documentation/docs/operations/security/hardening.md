---
title: Hardening
description: Hardening rninja deployments
tags:
  - operations
  - security
---

# Hardening

Security hardening for rninja deployments.

## Run as Non-Root

```bash
# Create user
useradd -r -s /bin/false rninja

# Set permissions
chown -R rninja:rninja /var/lib/rninja-cache
```

## Systemd Hardening

```ini
[Service]
User=rninja
Group=rninja

# Security restrictions
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX
RestrictNamespaces=true
RestrictRealtime=true
RestrictSUIDSGID=true
MemoryDenyWriteExecute=true
LockPersonality=true

ReadWritePaths=/var/lib/rninja-cache
```

## Network Restrictions

### Firewall

```bash
# Allow only internal network
ufw allow from 10.0.0.0/8 to any port 9999
```

### Listen Address

```bash
# Listen only on internal interface
rninja-cached --listen tcp://10.0.0.1:9999
```

## File Permissions

```bash
# Cache directory
chmod 750 /var/lib/rninja-cache

# Socket
chmod 700 /tmp/rninja-daemon.sock
```

## Checklist

- [ ] Run as non-root
- [ ] Apply systemd restrictions
- [ ] Configure firewall
- [ ] Restrict listen address
- [ ] Set proper file permissions
- [ ] Enable audit logging
