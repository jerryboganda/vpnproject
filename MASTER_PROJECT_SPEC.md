# VPNBridge Master Project Specification

## 1. Product definition

VPNBridge allows a Windows laptop to use the Internet path of an Android phone **through the VPN already connected on that phone**, without rooting Android.

The Android device acts as an application-layer secure gateway rather than modifying Android's privileged tethering/NAT tables.

## 2. Primary architecture

```text
Internet
   ^
   | encrypted VPN tunnel managed by third-party Android VPN app
   |
Android VPN Network (TRANSPORT_VPN)
   ^
   | every VPNBridge upstream socket explicitly bound here
   |
Rust gateway / SOCKS5 + control plane
   ^
   | hotspot-local listener only
   |
Android Local-Only Hotspot
   ^
   | local Wi-Fi, no direct Internet route
   |
Windows hotspot interface
   ^
   | gateway/control connection bypass only
   |
Wintun + TUN-to-SOCKS + kill switch
   ^
   |
All Windows applications
```

## 3. Why this design

A normal third-party Android app cannot depend on privileged system tethering controls or root-level packet/NAT manipulation. Local-Only Hotspot intentionally provides local communication without Internet access, so Windows cannot simply fall back to raw phone tethering through that network. VPNBridge then provides Internet access only through its own application-layer gateway.

## 4. Android data-plane invariant

The Android service has two distinct networking roles:

### Local ingress

- Listen only on the Local-Only Hotspot address/interface.
- Accept authenticated Windows sessions.
- Do not bind the whole process to the local hotspot network.

### VPN egress

- Discover a VPN `Network` applicable to VPNBridge.
- Require `TRANSPORT_VPN` and required capabilities.
- Bind each upstream socket to that VPN network before connection/data transmission.
- Resolve gateway-originated DNS via the same VPN network or an explicitly VPN-bound encrypted resolver.
- On loss of the VPN network, close all egress flows and stop forwarding.

This separation is the core no-root technique.

## 5. Modes

### Mode A — Proxy mode

Windows uses the authenticated SOCKS5 gateway directly. Useful for initial proof, debugging, and selected applications.

### Mode B — Full-tunnel mode

Windows uses Wintun plus TUN-to-SOCKS and kill-switch logic. This is the required production mode for whole-computer traffic.

### Future Mode C — Built-in VPN

VPNBridge itself may later implement a VPN protocol. This is not v1 because Android only permits one active VPN service per user and it would conflict with the external VPN mode.

## 6. Technology stack

### Shared

- Rust stable
- Tokio
- Serde only for control/configuration data, not packet hot path
- tracing with redaction
- cargo workspace

### Android

- Tauri 2
- Svelte 5 + TypeScript
- minimal Kotlin Tauri plugin/native layer
- Android foreground service type appropriate to connected-device/network interaction
- Local-Only Hotspot
- ConnectivityManager/NetworkCapabilities
- NDK or Kotlin socket-network binding bridge

### Windows

- Tauri 2
- Svelte 5 + TypeScript
- Rust networking/service layer
- Wintun
- current Rust TUN-to-SOCKS implementation such as `tun2proxy`, after dependency review
- WFP-based or equivalently robust kill switch
- DPAPI/Credential Manager for secrets

## 7. Product state machine

Android top-level states:

```text
STOPPED
  -> PREPARING_HOTSPOT
  -> WAITING_FOR_VPN
  -> VALIDATING_VPN
  -> READY_LOCAL
  -> SAFE_FORWARDING
  -> VPN_LOST_FAIL_CLOSED
  -> REVALIDATING
  -> SAFE_FORWARDING
```

Any invalid transition goes to a safe non-forwarding state.

Windows states:

```text
DISCONNECTED
  -> DISCOVERING_PHONE
  -> PAIRING/CONNECTING
  -> PROXY_READY
  -> INSTALLING_TUN_ROUTE
  -> PROTECTED
  -> DEGRADED_FAIL_CLOSED
  -> RECONNECTING
  -> PROTECTED
```

## 8. Security model

Production sessions must authenticate the paired Windows device. The hotspot password is not sufficient as the sole authentication factor.

Use a production secure channel or authenticated application protocol. A recommended progression is:

1. MVP: cryptographically random per-session SOCKS credential + WPA2/WPA3 Local-Only Hotspot.
2. Production: mutual device identity and TLS 1.3/Noise-style secure channel, with credential rotation and replay protection.

## 9. Leak protection

A release cannot ship if any test shows successful raw egress when VPN protection is supposed to be active or immediately after the product has observed VPN loss.

Protect:

- TCP
- UDP
- DNS
- IPv4
- IPv6
- reconnect paths
- Windows route fallback
- Android VPN replacement/split tunnel

## 10. Performance model

The local hop must add minimal overhead. Benchmark against the same Android VPN/upstream conditions.

Targets are defined in `docs/10_PERFORMANCE_TARGETS.md`; optimize based on measurements, not guesses.

## 11. Autonomy model

Antigravity maintains living repository state, automatically selects the next unblocked task, verifies current APIs, creates experiments for ambiguity, tests each coherent slice, performs independent gap hunts, and only pauses for unavoidable human/physical-device gates.

## 12. Release philosophy

A release is evidence-driven. It must survive:

- VPN drops and changes,
- Android service lifecycle pressure,
- hotspot loss/recreation,
- Windows sleep/resume,
- app/service crashes,
- DNS/IPv6 tests,
- repeated connection churn,
- malicious/malformed local input,
- install/upgrade/uninstall,
- supported VPN/provider/device combinations.

See `docs/18_DEFINITION_OF_DONE.md` and `docs/25_ACCEPTANCE_TEST_MATRIX.md`.
