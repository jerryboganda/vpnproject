---
name: security-review
description: Threat-model and security-audit VPNBridge pairing, local gateway protocol, socket routing, secrets, kill switch, logs, and dependency supply chain.
---
# Security Review Skill

Use before exposing a new network surface and at each release candidate.

Review:

- unauthorized hotspot clients,
- gateway authentication bypass,
- replay,
- malformed SOCKS/control packets,
- UDP amplification/abuse,
- resource exhaustion,
- route/DNS/IPv6 leaks,
- VPN split-tunnel exclusion,
- VPN disconnect races,
- stale Android `Network` handles,
- Windows kill-switch bypass,
- privilege escalation and unsafe service IPC,
- secrets at rest,
- log privacy,
- update/package integrity,
- dependency provenance/license/security.

Require bounded inputs, fail-closed state transitions, least privilege, secure random values, secret storage, and adversarial tests.
