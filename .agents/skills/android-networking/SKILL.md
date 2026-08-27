---
name: android-networking
description: Implement and audit VPNBridge Android hotspot, VPN-network binding, foreground service, DNS, lifecycle, and fail-closed networking.
---
# Android Networking Skill

Use this skill whenever work touches Android connectivity, Local-Only Hotspot, VPN detection, socket binding, permissions, foreground execution, or network lifecycle.

## Workflow

1. Read `docs/04_ANDROID_NETWORKING_SPEC.md` and `docs/07_DNS_IPV6_LEAK_PROTECTION.md`.
2. Confirm APIs against current `developer.android.com` documentation.
3. Keep Local-Only Hotspot control in Kotlin/Android framework code.
4. Discover the active default network for VPNBridge and require `NetworkCapabilities.TRANSPORT_VPN`.
5. Obtain a stable network handle/bridge usable by Rust.
6. For every upstream socket, bind that socket to the validated VPN `Network` before connection/use.
7. Bind local listener sockets only to the hotspot-local IP/interface; do not process-bind to the hotspot.
8. Handle VPN `onLost`, replacement, route changes, DNS changes, and hotspot teardown as explicit state-machine events.
9. Forward only while state is `SAFE_FORWARDING`.
10. Test VPN loss under continuous TCP, UDP, DNS, and QUIC-like traffic.

## Forbidden shortcuts

- No `VpnService.protect()` for gateway upstream sockets in external-VPN mode.
- No binding upstream sockets to cellular/Wi-Fi physical networks.
- No system tethering or privileged tethering API assumptions.
- No hidden APIs or reflection hacks.
- No continuing forwarding after VPN network loss.

## Required evidence

Record API references, instrumentation logs, successful protected egress, and VPN-drop leak-test results in `TEST_EVIDENCE.md`.
