# Architectural & Engineering Decisions (ADRs)

## ADR-001: Fail-Closed Generational Sockets
- **Context:** If the Android VPN tunnel disconnects or reconnects, lingering TCP/UDP sockets must not silently route over cellular or raw Wi-Fi egress.
- **Decision:** Every VPN validation increments a monotonic generation counter (`u64`) and mints a `CancellationToken`. Sockets receive a `VpnBindingReceipt(generation)`. Any drop or generation bump immediately cancels the active generation token, causing active Tokio stream pumps to break synchronously.
- **Outcome:** Zero unmanaged packet leaks. Verified via `vpn_lifecycle_race_test`.

## ADR-002: Pure-Rust Cryptographic Dependencies
- **Context:** C-based cryptography libraries like `ring` require external MSVC/GCC C compiler tools and Perl during build, introducing host platform link dependencies.
- **Decision:** Use pure-Rust `sha2` (v0.10), `hmac` (v0.12), and `subtle` (v2.6) for HMAC-SHA256 challenge-response proofs and constant-time token comparison.
- **Outcome:** Clean, standalone compilation across Windows, Android, and CI without external C toolchain compilation issues.

## ADR-003: LLVM-MinGW Toolchain Integration on Windows Host
- **Context:** The Windows development environment uses `x86_64-pc-windows-gnu` which requires `dlltool` and `as` when building import libraries.
- **Decision:** Provisioned `MartinStorsjo.LLVM-MinGW.UCRT` via `winget` and configured `.cargo/config.toml` with `rust-lld`.
- **Outcome:** Workspace builds, tests, and clippy passes with zero warnings.

## ADR-004: WFP Kill Switch and Persistent Recovery Journal on Windows
- **Context:** If the Windows companion app crashes or is forcibly terminated while routes and firewall rules are mutated, the user could be left without network access or with exposed routes.
- **Decision:** State is snapshotted into `RecoveryJournal` on disk before any routing change. On startup/shutdown, the journal is checked and network state is automatically restored.
- **Outcome:** Crash-safe full tunnel routing.

## ADR-005: Svelte 5 Runes for Frontend Architecture
- **Context:** Frontend requires real-time reactive telemetry (RX/TX counters, connection state, throughput) with minimal bundle size and maximum runtime speed.
- **Decision:** Use Svelte 5 with `$state` runes and direct Tauri 2 invoke APIs.
- **Outcome:** Clean, modern, lightweight UI for both Android and Windows companion apps.
