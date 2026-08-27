---
name: windows-tunnel
description: Implement and audit VPNBridge Windows Wintun, TUN-to-SOCKS, routing, DNS, privilege, recovery, and kill-switch behavior.
---
# Windows Tunnel Skill

Use for Windows companion networking and routing tasks.

## Preferred architecture

- Tauri 2/Svelte UI.
- Rust service/data plane.
- Wintun for Layer-3 TUN.
- Evaluate current `tun2proxy` as the preferred Rust TUN-to-SOCKS component before writing a custom userspace stack.
- Keep dependency behind a local abstraction so it can be replaced.
- Use an authenticated phone gateway endpoint.
- Add Windows Filtering Platform (WFP) or an equivalently robust fail-closed mechanism for the production kill switch.

## Routing invariant

The physical hotspot interface may communicate with the phone gateway/control endpoint and local network essentials only. General Internet traffic must use the VPNBridge TUN path while protection is enabled.

## Failure behavior

If phone gateway, VPN-protected state, TUN engine, or control heartbeat fails:

1. stop new tunneled sessions,
2. preserve the kill switch,
3. do not silently restore the raw default route,
4. show a clear protected/disconnected state,
5. recover automatically only after the phone reports a validated VPN path.

## Tests

Test TCP, UDP, DNS, IPv4, IPv6, browser QUIC, sleep/resume, Wi-Fi reconnect, process crash, service restart, route restoration, uninstall cleanup, and repeated Android VPN toggling.
