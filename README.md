# VPNBridge — Antigravity Autonomous Development Pack

VPNBridge is a no-root Android 15+ gateway that shares an Android phone's **VPN-protected** Internet connection to a Windows laptop over a private local Wi-Fi link.

The product deliberately does **not** depend on privileged Android tethering/NAT manipulation. The primary architecture is:

`Windows -> Local-Only Hotspot -> Android VPNBridge Gateway -> existing Android VPN -> Internet`

A Windows companion creates a full-device tunnel using Wintun/TUN-to-SOCKS technology and adds fail-closed routing/leak protection.

## Start here

1. Read `START_HERE.md`.
2. Antigravity must load `.agents/AGENTS.md` as its persistent project rules.
3. Read `MASTER_PROJECT_SPEC.md` before changing code.
4. Execute `docs/16_IMPLEMENTATION_PHASES.md` in order.
5. Maintain the state files defined in `templates/`.
6. Never mark the project complete until every gate in `docs/18_DEFINITION_OF_DONE.md` and `docs/25_ACCEPTANCE_TEST_MATRIX.md` passes with evidence.

## Non-negotiable product constraints

- No Android root.
- Primary target device: Android 15 / API 35.
- Tauri 2 + Svelte 5 UI.
- Rust for networking/core logic.
- Minimal Kotlin only where Android framework APIs require it.
- Windows companion: Tauri 2 + Rust.
- Existing third-party Android VPN is supported in v1.
- No raw-Internet fallback when VPN protection is lost.
- DNS and IPv6 must be explicitly protected.
- No completion claims without tests and evidence.

## Recommended first milestone

Build the smallest end-to-end proof:

1. Detect an active Android VPN applicable to VPNBridge.
2. Create a Local-Only Hotspot.
3. Run an authenticated SOCKS5 gateway on the hotspot-local address.
4. Bind each gateway upstream socket explicitly to the active Android VPN `Network` before connecting.
5. Connect Windows through the proxy.
6. Verify the laptop public IP is the VPN egress IP.
7. Disconnect the Android VPN and verify no raw Internet traffic succeeds.

Only after this proof is stable should the full Windows Wintun mode be completed.

## Important expectation

The documentation is designed to minimize defects and gaps, but no engineering process can truthfully guarantee literal zero bugs. The project therefore uses strict fail-closed behavior, automated tests, repeated audits, hardware validation, and objective release gates.
