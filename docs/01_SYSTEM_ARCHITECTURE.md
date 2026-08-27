# 01 — System Architecture

## Components

### Android UI

Tauri 2 + Svelte 5. UI is control-plane only; it must never own long-lived packet loops.

### Android native bridge

Kotlin Tauri plugin/service layer for:

- Local-Only Hotspot lifecycle,
- permissions,
- `ConnectivityManager` callbacks,
- VPN `Network` selection and `NetworkCapabilities`,
- VPN `LinkProperties` including DNS,
- network-handle/socket-binding bridge,
- foreground service and notification,
- Android lifecycle events.

### Shared Rust gateway core

Responsibilities:

- client authentication/session state,
- SOCKS5-compatible proxy semantics,
- TCP forwarding,
- UDP association/flow table,
- bounded queues/backpressure,
- counters/telemetry,
- cancellation and shutdown,
- protocol parsing with strict limits.

### Windows companion

Tauri 2/Svelte 5 UI plus Rust networking/service layer.

Subsystems:

- phone discovery/manual connect,
- pairing,
- local secure transport,
- Wintun lifecycle,
- TUN-to-SOCKS engine,
- route manager,
- WFP kill switch,
- DNS/IPv6 policy,
- recovery/watchdog,
- diagnostics.

## Data flow

### TCP

```text
Windows app
 -> Wintun
 -> TUN-to-SOCKS
 -> phone local gateway
 -> Android creates upstream socket
 -> socket is bound to active VPN Network
 -> destination
```

### UDP

```text
Windows datagram
 -> Wintun
 -> TUN-to-SOCKS UDP path
 -> authenticated phone UDP association
 -> Android VPN-bound UDP socket
 -> destination
```

### DNS

Preferred order:

1. Use DNS servers advertised in `LinkProperties` of the validated VPN network and send via VPN-bound sockets.
2. Optionally use a user-selected DoH/DoT resolver over a VPN-bound connection.
3. Never send DNS directly over the physical Android upstream or raw Windows interface while protected mode is active.

## Control flow

The phone is authoritative about whether forwarding is safe. It sends signed/authenticated state updates to Windows. Windows must treat missing/expired heartbeats as unsafe.

## Isolation

The Local-Only Hotspot intentionally has no Internet route of its own. It is used as a point-to-point-like local transport. Windows should not assume the phone hotspot gateway address; discover it from the actual interface/configuration and allow manual fallback.

## Concurrency

- One Rust runtime per app/service process.
- One supervisor task owns lifecycle cancellation.
- Per-session and per-flow tasks are children of a cancellation tree.
- All channels are bounded.
- UDP flow table has idle expiry and maximum size.
- TCP connection count has configurable safety caps.

## Error boundaries

Any error involving VPN validity, socket-to-network binding, route installation, kill switch state, or authentication is safety-critical and transitions to fail-closed behavior.

Ordinary destination errors are per-flow and must not crash the gateway.
