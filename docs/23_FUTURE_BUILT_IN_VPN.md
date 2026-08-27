# 23 — Future Built-In VPN Mode

This is a post-v1 roadmap item.

## Goal

Allow VPNBridge itself to establish the Android VPN rather than requiring a separate VPN app.

## Constraint

Android permits only one active VPN service per user. Built-In VPN mode and External VPN mode are mutually exclusive.

## Potential protocols

Evaluate maintained userspace implementations of:

- WireGuard,
- OpenVPN only if footprint/complexity is acceptable,
- modern proxy/tunnel engines if product scope expands.

## Architectural reuse

The local hotspot, Windows client, pairing, TUN, kill switch, DNS/leak protection, observability, and most UI remain reusable. Replace the external-VPN socket binding layer with an internal VPN engine/route ownership model.

## Do not implement early

Do not allow this roadmap to delay v1 feasibility and stability.
