# 10 — Performance Targets and Benchmarking

## Philosophy

Measure incremental VPNBridge overhead under identical upstream/VPN conditions. Raw absolute throughput varies by phone, radio, VPN provider, server distance, and Windows hardware.

## Initial targets

These are engineering targets, not marketing guarantees.

- Local forwarding latency overhead: p50 <= 5 ms and p95 <= 15 ms on a stable local hotspot under non-saturated conditions.
- Throughput: target >= 90% of a comparable direct protected proxy baseline when phone/VPN/radio are not the bottleneck.
- No sustained busy-loop CPU at idle.
- No unbounded memory growth during 8-hour soak.
- Gateway memory remains stable under connection churn.
- Reconnect should occur automatically without user action once a valid VPN network returns, subject to OS timing.

## Hot-path rules

- bounded reusable buffers,
- avoid per-packet JSON/serialization,
- avoid unnecessary Vec reallocations,
- batch where APIs support it,
- no blocking DNS/file work on Tokio worker threads,
- sharded/concurrent UDP map if contention appears,
- use atomics for simple counters,
- keep logging sampled/off the packet hot path.

## Benchmarks

Create repeatable tests for:

- TCP single stream.
- TCP 8/32 parallel streams.
- small request/response latency.
- UDP throughput and packet loss.
- DNS latency.
- 1k/10k connection churn.
- 8-hour soak.
- VPN server switch.
- screen-on vs screen-off Android behavior.

Capture:

- Mbps,
- p50/p95/p99 latency,
- CPU,
- RSS,
- battery delta where feasible,
- packet loss,
- reconnect time,
- errors per million operations.

Optimize only after obtaining a baseline profile.
