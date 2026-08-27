# 05 — Gateway and Pairing Protocol

## MVP protocol

Use standards where possible to reduce risk.

### Data plane

Authenticated SOCKS5:

- CONNECT for TCP.
- UDP ASSOCIATE for UDP.
- domain-name address type supported.
- strict parser and length limits.
- no unauthenticated production listener.

### Control plane

A small versioned binary or compact CBOR-like protocol may carry:

- protocol version,
- device identity,
- pairing challenge/response,
- heartbeat,
- protected/unprotected state,
- DNS/IPv6 capability flags,
- throughput counters,
- controlled shutdown/reconnect messages.

Do not use verbose JSON per packet.

## Authentication stages

### Development/MVP

- OS-random 256-bit session secret.
- short-lived pairing code derived from a cryptographic challenge, not the secret itself.
- rate-limited failed authentication.
- listener available only on hotspot-local interface.

### Production

- long-term device key pair stored in platform secure storage.
- authenticated ephemeral session handshake.
- forward-secret secure channel using a well-reviewed TLS 1.3 or Noise implementation.
- per-session keys.
- sequence/replay protection.
- key rotation and unpair support.

Do not design custom cryptography.

## Protocol versioning

Every session begins with explicit version negotiation. Unknown major versions fail closed with a clear upgrade error. Minor versions may add optional capabilities.

## Resource limits

Configurable hard caps:

- maximum paired clients,
- maximum concurrent TCP streams,
- maximum UDP mappings,
- maximum control frame size,
- maximum domain length,
- authentication attempts per minute,
- per-flow idle timeouts.

## UDP

Maintain NAT-like association state keyed by client session + source tuple. Expire idle mappings. Prevent reflection by sending replies only to authenticated associations and destinations initiated by the client.

## Backpressure

Do not buffer unbounded data if phone VPN or Windows link slows. Apply bounded channel capacity and suspend reads or close abusive flows.
