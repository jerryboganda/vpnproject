# 16 — Implementation Phases

Do not skip phases. A later phase may be prepared in parallel only when it does not hide a blocker in an earlier phase.

## Phase 0 — Feasibility laboratory

Deliver isolated experiments, not UI polish.

- Android app detects active VPN for its own UID.
- Local-Only Hotspot starts/stops.
- Laptop can reach an Android-local TCP server.
- Android Rust/Kotlin can create an outbound socket and bind it to the detected VPN network.
- Prove socket stops working when that VPN network disappears.
- Record OEM/device behavior.

**Gate:** all central assumptions have physical-device evidence.

## Phase 1 — Protected TCP proxy MVP

- authenticated SOCKS5 TCP CONNECT.
- hotspot-local listener.
- upstream per-socket VPN binding.
- VPN generation state machine.
- TCP cancellation on VPN loss.
- minimal Android Tauri status UI.
- Windows manual SOCKS test.

**Gate:** Windows HTTPS traffic exits through Android VPN; repeated VPN toggles never produce observed raw fallback.

## Phase 2 — UDP and DNS

- SOCKS5 UDP ASSOCIATE.
- bounded UDP mapping table.
- VPN-bound UDP sockets.
- VPN DNS discovery.
- DNS-through-protected-path.
- DNS/UDP leak tests.

**Gate:** UDP and DNS pass with VPN loss injection.

## Phase 3 — Pairing and production local security

- device pairing.
- secure storage.
- production authenticated secure transport.
- unpair/rekey.
- abuse/rate-limit handling.

**Gate:** unauthorized client tests fail safely; protocol fuzzing is green.

## Phase 4 — Windows companion Proxy Mode

- Tauri Windows app.
- phone discovery/manual entry.
- pairing UX.
- protected-state heartbeat.
- diagnostics.

**Gate:** robust reconnect across hotspot restart and Windows sleep/resume in Proxy Mode.

## Phase 5 — Full Tunnel / Wintun

- evaluate/integrate current `tun2proxy` or equivalent.
- Wintun lifecycle.
- TUN routes.
- TCP/UDP/DNS full-device traffic.
- explicit IPv6 tunnel or block.

**Gate:** browsers and representative non-proxy-aware apps work through the phone VPN.

## Phase 6 — Windows fail-closed kill switch

- minimal privileged helper.
- WFP policy.
- route recovery journal.
- crash/reboot recovery.
- uninstall cleanup.

**Gate:** killing TUN/client/phone/VPN cannot expose raw Internet while protection is expected.

## Phase 7 — Product UX and resilience

- polished Android/Windows UI.
- actionable errors.
- foreground/background lifecycle.
- OEM battery guidance only where needed.
- auto-reconnect.
- sanitized diagnostics export.

**Gate:** complete user workflows pass without developer tools.

## Phase 8 — Performance and soak

- profile CPU/memory/copies/locks.
- optimize measured bottlenecks.
- high-concurrency tests.
- 8-hour soak.
- battery observations.

**Gate:** performance targets are met or variances are documented and accepted.

## Phase 9 — Security and gap audit

Run independent audits using `FINAL_AUDIT_PROMPT.md` and `BUG_HUNT_PROMPT.md`.

**Gate:** no unresolved critical/high issues, no critical TODO/stub paths.

## Phase 10 — Release candidate

- full acceptance matrix.
- supported device/VPN matrix.
- clean installation.
- upgrade.
- uninstall/route restoration.
- signed artifacts.
- release notes and known limitations.

**Gate:** `18_DEFINITION_OF_DONE.md` satisfied.
