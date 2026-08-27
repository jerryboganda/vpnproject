# 00 — Product Requirements

## Goal

Provide a no-root Android 15+ application that exposes the Android phone's VPN-protected Internet path to a Windows computer connected over a local Wi-Fi link.

## Primary user story

1. User connects an existing VPN on Android.
2. User opens VPNBridge and taps **Share VPN**.
3. VPNBridge verifies that its own traffic is covered by an active VPN network.
4. VPNBridge starts a private Local-Only Hotspot and local authenticated gateway.
5. Windows connects to that hotspot and the VPNBridge desktop app connects to the phone.
6. In Full Tunnel mode, all eligible Windows IPv4/IPv6, TCP/UDP, and DNS traffic is routed through the phone gateway and the phone's VPN.
7. If VPN protection disappears, Internet forwarding stops rather than falling back to raw connectivity.

## Functional requirements

### Android

- Detect whether VPNBridge itself has an active `TRANSPORT_VPN` default path.
- Start/stop Local-Only Hotspot using supported public APIs.
- Show SSID/credential or QR pairing information.
- Run a foreground gateway service while sharing.
- Accept only authenticated clients.
- Support TCP CONNECT and UDP forwarding.
- Supply protected DNS behavior.
- Bind Internet-facing sockets to the validated VPN `Network`.
- Detect VPN loss/replacement and fail closed.
- Recover without requiring app restart when the VPN safely returns.
- Expose status, throughput, session, and actionable error states.

### Windows

- Discover or manually connect to the phone gateway.
- Pair securely.
- Provide Proxy Mode.
- Provide Full Tunnel Mode through Wintun/TUN-to-SOCKS.
- Route DNS through the protected path.
- Protect IPv6 or disable it for the protected session until fully supported.
- Install/remove routes safely.
- Implement a kill switch that prevents raw fallback while protection is enabled.
- Restore the user's original network state on intentional disconnect/uninstall.
- Recover after sleep/resume and transient phone reconnect.

## Non-functional requirements

- No root or privileged Android APIs.
- Fail closed.
- Minimal battery and CPU overhead.
- High throughput with bounded memory.
- Clear, simple UI.
- No ads/analytics requirement in core design.
- No cloud account required for local sharing.
- Deterministic logs and diagnostics without secrets.
- Reproducible builds and dependency pinning.

## Supported baseline

- Primary Android runtime: Android 15 / API 35.
- Primary Windows runtime: Windows 11 x64.
- Secondary: Windows 10 only if dependencies remain supported and tests pass.
- Cellular upstream + Android VPN is the first required upstream configuration.
- Wi-Fi upstream + Local-Only Hotspot is supported only on devices whose hardware/OEM permits concurrent STA+AP behavior; detect and document unsupported cases.

## Out of scope for v1

- Android root workflows.
- iOS/macOS/Linux client apps.
- Sharing to arbitrary hotspot clients without the Windows companion as the production full-tunnel method.
- Running VPNBridge's own Android `VpnService` at the same time as another VPN.
- VPN provider account management.
- Circumventing enterprise/device-owner policy restrictions.

## Success criteria

The release candidate must demonstrate on physical hardware that:

- Windows public egress matches the Android VPN path.
- DNS resolves through protected routing.
- full-tunnel TCP and UDP function.
- repeated VPN disconnects produce no successful raw fallback in the leak harness.
- reconnect recovers automatically.
- no critical/high security or correctness defects remain.
