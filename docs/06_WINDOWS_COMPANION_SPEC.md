# 06 — Windows Companion Specification

## Modes

### Proxy Mode

Expose/configure the phone SOCKS5 gateway for targeted applications. Primarily for development, troubleshooting, and fallback.

### Full Tunnel Mode

Required production mode:

```text
Windows apps -> Wintun -> TUN-to-SOCKS -> phone gateway -> Android VPN
```

## TUN engine

Before implementing a custom TCP/IP stack, evaluate the current `tun2proxy` Rust project because it supports Windows/Wintun, IPv4/IPv6, SOCKS5, UDP, and DNS-related behavior.

Integration preference:

1. library/API integration if stable and maintainable,
2. bundled verified sidecar behind an abstraction if library integration is impractical,
3. custom implementation only if measured requirements cannot be met.

Pin exact versions and preserve required license/provenance files.

## Wintun

Use official signed Wintun distribution and comply with its distribution license. Do not ship self-renamed/rebuilt artifacts contrary to upstream guidance.

Abstract adapter operations:

- install/load,
- create/open adapter,
- start session,
- read/write packets,
- teardown.

## Privilege model

Network adapter, route, DNS, and WFP changes may require elevation. Prefer a minimal privileged helper/service with authenticated local IPC rather than running the entire UI elevated.

## Route transaction

Before modifying routes:

1. snapshot relevant current routes/interface metrics/DNS,
2. write a recovery journal,
3. install explicit bypass route to the phone hotspot gateway/control endpoint,
4. install TUN routes,
5. activate kill-switch policy,
6. verify connectivity through the protected path,
7. commit transaction.

On intentional disconnect, restore the snapshot safely.

## Kill switch

Production should use WFP or another robust mechanism to block raw Internet fallback. Allowed traffic while protected should be narrowly scoped:

- phone local gateway/control endpoint over hotspot interface,
- DHCP/local network traffic strictly necessary to retain the hotspot connection,
- traffic through the VPNBridge TUN/protected stack,
- other explicit user-approved local exclusions if later supported.

A TUN crash must not silently expose the raw default route.

## DNS

Route DNS into TUN. Do not leave the physical adapter as the preferred resolver while protected.

## IPv6

If full IPv6 support is not proven, block IPv6 during protected sessions and report that state. Never allow IPv6 to bypass an IPv4-only tunnel.

## Recovery

Handle:

- Windows sleep/resume,
- Wi-Fi disconnect/reconnect,
- phone hotspot recreated with a new subnet,
- phone app process restart,
- TUN helper crash,
- privileged service restart,
- stale recovery journal after machine reboot.
