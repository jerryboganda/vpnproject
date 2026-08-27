# 14 — Observability and Logging

## Goals

Logs must make race conditions and routing failures diagnosable without becoming a privacy liability.

## Structured events

Use stable event names/codes, for example:

- `ANDROID_VPN_FOUND`
- `ANDROID_VPN_INVALID`
- `ANDROID_VPN_LOST`
- `HOTSPOT_STARTED`
- `HOTSPOT_STOPPED`
- `UPSTREAM_SOCKET_BOUND`
- `UPSTREAM_BIND_FAILED`
- `CLIENT_PAIRED`
- `CLIENT_AUTH_FAILED`
- `TCP_FLOW_OPEN/CLOSE`
- `UDP_ASSOC_OPEN/CLOSE`
- `WINDOWS_TUN_UP/DOWN`
- `WINDOWS_KILLSWITCH_ON/OFF`
- `ROUTE_TRANSACTION_ROLLBACK`

## Required fields

- monotonic timestamp,
- session ID (random, non-secret),
- VPN generation,
- component,
- event code,
- severity,
- error code,
- bounded diagnostic metadata.

## Privacy

Do not log by default:

- passwords/tokens/private keys,
- full browsing history,
- full DNS query history,
- packet payloads,
- Authorization headers.

Domain/IP logging for debugging must be opt-in, time-limited, and clearly marked.

## Diagnostics bundle

Export a sanitized bundle containing:

- app versions,
- OS/device info,
- current config excluding secrets,
- route state summaries,
- VPN/hotspot state transitions,
- recent error events,
- test/self-check results.

## Metrics

Counters/gauges:

- active TCP flows,
- active UDP mappings,
- bytes up/down,
- rejected auth attempts,
- bind failures,
- VPN generation changes,
- reconnects,
- queue depth/high-water marks,
- dropped packets/datagrams,
- memory/CPU samples in diagnostics mode.
