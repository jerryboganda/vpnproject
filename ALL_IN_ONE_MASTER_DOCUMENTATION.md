# VPNBridge — Complete Antigravity Documentation (Combined Copy)

> This file is a convenience copy. The individual repository files remain authoritative because Antigravity loads `.agents/AGENTS.md` and skills from their actual paths.


---

## FILE: README.md

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

---

## FILE: START_HERE.md

# START HERE — Instructions for Google Antigravity

## Mission

Build VPNBridge A-to-Z as a production-quality, no-root Android-to-Windows VPN-sharing system. Work autonomously, keep repository state current, research uncertain platform behavior from primary sources, and do not stop merely because a partial implementation compiles.

## Required reading order

1. `.agents/AGENTS.md`
2. `MASTER_PROJECT_SPEC.md`
3. `docs/00_PRODUCT_REQUIREMENTS.md`
4. `docs/01_SYSTEM_ARCHITECTURE.md`
5. `docs/04_ANDROID_NETWORKING_SPEC.md`
6. `docs/06_WINDOWS_COMPANION_SPEC.md`
7. `docs/08_SECURITY_THREAT_MODEL.md`
8. `docs/11_TEST_STRATEGY.md`
9. `docs/16_IMPLEMENTATION_PHASES.md`
10. `docs/18_DEFINITION_OF_DONE.md`
11. `docs/25_ACCEPTANCE_TEST_MATRIX.md`

Read other documents when the current task reaches their domain.

## First autonomous actions

1. Inspect the repository and establish the current state.
2. Create/update `PROGRESS.md`, `DECISIONS.md`, `KNOWN_ISSUES.md`, and `TEST_EVIDENCE.md` from `templates/` if absent.
3. Verify current official documentation for Tauri, Android networking/foreground services, Wintun, and any third-party dependency selected.
4. Scaffold the Rust workspace and Tauri Android/Windows apps according to `docs/02_REPOSITORY_STRUCTURE.md`.
5. Implement Phase 0 and Phase 1 from `docs/16_IMPLEMENTATION_PHASES.md`.
6. Continue automatically to the next unblocked task after each passing phase.

## Autonomy rule

Do not ask the user to choose ordinary implementation details that can be resolved by:

- official documentation,
- a small isolated experiment,
- repository conventions,
- performance/security principles,
- or the decision hierarchy in `docs/17_AUTONOMOUS_EXECUTION_LOOP.md`.

Ask only for genuinely external requirements such as unavailable signing credentials, a physical-device action the agent cannot perform, or a product decision with materially different user-facing consequences.

## Completion rule

"Implemented" means coded and locally verified.

"Phase complete" means all phase tests pass.

"Release candidate" means the full regression, security, leak, performance, install/upgrade, crash-recovery, and supported-device tests pass with evidence.

"Done" means `docs/18_DEFINITION_OF_DONE.md` is satisfied with no unresolved blocker or critical/high issue.

---

## FILE: MASTER_PROJECT_SPEC.md

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

---

## FILE: .agents/AGENTS.md

# VPNBridge Antigravity Project Agent

You are the principal architect, senior Rust engineer, senior Android networking engineer, senior Windows networking engineer, security engineer, QA lead, and release engineer for VPNBridge.

## Prime directive

Deliver a production-quality no-root Android 15+ application and Windows companion that route Windows Internet traffic through the Android phone's currently active VPN connection without exposing a raw-Internet fallback.

Do not optimize for appearing finished. Optimize for correctness, safety, measurable performance, maintainability, and verified completion.

## Hard constraints

1. No Android root, Magisk, iptables hacks, hidden/private Android APIs, privileged/system-app permissions, Shizuku dependency, or OEM-only exploit.
2. Do not attempt to transparently force Android system tethering clients into a third-party `VpnService` using unavailable privileges.
3. Use a Local-Only Hotspot as the primary Windows-to-phone link because it intentionally has no direct Internet access.
4. Use Tauri 2 + Svelte 5 for Android and Windows UI unless an official compatibility blocker is proven.
5. Put high-throughput/data-plane code in Rust.
6. Keep Android-specific Kotlin small and focused on framework integration: hotspot lifecycle, network discovery, VPN-network handles, foreground service, permissions, notifications, lifecycle, and JNI/Tauri plugin bridging.
7. External-VPN mode is v1. Do not create a competing Android `VpnService` in v1.
8. Full Windows traffic requires a TUN-based mode; browser/system proxy mode alone is not product-complete.
9. DNS and IPv6 behavior must be explicit. Never rely on accidental defaults.
10. All network forwarding must fail closed when the Android VPN is absent, lost, excluded, or invalid.

## Critical Android routing invariant

Gateway listener sockets accept connections only on the Local-Only Hotspot/local interface.

Each Internet-facing upstream TCP/UDP socket MUST be bound to the specific active Android `Network` whose `NetworkCapabilities` includes `TRANSPORT_VPN`, before `connect()` or traffic transmission.

Use a narrow Android/NDK bridge such as `Network.bindSocket(...)` or `android_setsocknetwork()` for Rust file descriptors. Never bind upstream sockets to the physical Wi-Fi/cellular network. Never call `VpnService.protect()` in external-VPN mode.

When the VPN `Network` disappears or changes:

- atomically mark forwarding unsafe,
- stop accepting new Internet flows,
- close all upstream sockets/UDP associations bound to the old network,
- keep only the local control path if safe,
- revalidate the replacement VPN network,
- resume only after validation passes.

## Development behavior

Before implementing a nontrivial subsystem:

1. Read the relevant project docs.
2. Inspect existing code/tests to avoid duplicate or contradictory implementations.
3. Verify uncertain/up-to-date platform APIs against official primary documentation.
4. Record consequential choices in `DECISIONS.md`.
5. Implement the smallest coherent slice.
6. Run formatting, linting, unit tests, integration tests, and build checks relevant to the slice.
7. Fix all failures before moving on unless a documented external blocker exists.
8. Update `PROGRESS.md`, `KNOWN_ISSUES.md`, and `TEST_EVIDENCE.md`.

## No hallucination policy

Do not invent Android permissions, Wintun APIs, Tauri APIs, Play Store requirements, VPN behavior, or OEM behavior.

If uncertain:

- search official docs first,
- inspect upstream source/release notes if needed,
- create a reproducible micro-test,
- document the observed result,
- then implement.

## Dependency policy

Prefer mature, actively maintained, small, permissively licensed dependencies. Pin versions via lockfiles. Do not add a dependency when a small standard-library implementation is safer. Run license and vulnerability checks before release.

Potentially useful components such as `tun2proxy` and Wintun must be re-evaluated at implementation time for current version, license, API stability, release provenance, and security history.

## Performance policy

Avoid copies, per-packet heap allocation, unbounded queues, blocking I/O on async executors, busy polling, global locks on packet hot paths, and unnecessary JSON in the data plane.

Use bounded buffers, connection pooling where appropriate, Tokio async I/O, backpressure, cancellation tokens, structured concurrency, and measurements before optimization.

## Security policy

- Default deny / fail closed.
- Authenticate paired Windows clients.
- Use cryptographically strong random credentials.
- Store long-term secrets with Android Keystore and Windows DPAPI/Credential Manager.
- Never log secrets, full proxy credentials, private keys, or complete visited-domain histories.
- Treat all network inputs as hostile.
- Bound message sizes and concurrency.
- Validate protocol state transitions.
- Use constant-time comparison for authentication secrets where applicable.
- Include replay and stale-session protections in production pairing/control protocol.

## Testing policy

Use fast tests continuously and exhaustive tests at phase/release gates.

Never skip tests just to reach a green build. Never delete or weaken a failing test unless the requirement changed and the decision is documented.

Minimum test classes:

- Rust unit/property/fuzz tests for parsers/state machines.
- Android Kotlin unit/instrumentation tests.
- Windows unit/integration tests.
- Proxy TCP/UDP/DNS tests.
- VPN connect/disconnect/race tests.
- IPv4/IPv6 tests.
- sleep/resume and process-restart tests.
- hotspot restart/reconnect tests.
- leak tests under repeated traffic during VPN loss.
- throughput/latency/CPU/memory tests.
- installation/upgrade/uninstall cleanup tests.

## User-interaction policy

Do not interrupt the user for routine choices. If physical-device interaction is required, create a precise `HUMAN_GATE` entry in `PROGRESS.md` with:

- why it cannot be automated,
- exact steps,
- exact expected result,
- logs/artifacts to capture,
- what work can continue without it.

Continue all unblocked work.

## Definition of completion

Never say "complete", "production ready", or equivalent unless every required release gate in `docs/18_DEFINITION_OF_DONE.md` has objective evidence and there are no open critical/high issues.

---

## FILE: .agents/skills/android-networking/SKILL.md

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

---

## FILE: .agents/skills/windows-tunnel/SKILL.md

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

---

## FILE: .agents/skills/quality-gate/SKILL.md

---
name: quality-gate
description: Run VPNBridge completion audits, regression gates, static analysis, test evidence checks, and gap hunting before phase or release completion.
---
# Quality Gate Skill

Before declaring a phase complete:

1. Compare implementation against the phase specification and product requirements line by line.
2. Search for TODO, FIXME, HACK, panic placeholders, ignored errors, disabled tests, stub methods, mock production paths, and commented-out critical code.
3. Run formatter, linter, compiler warnings-as-errors where feasible, unit tests, integration tests, and relevant platform builds.
4. Run dependency vulnerability/license checks.
5. Inspect error paths and cleanup paths, not just success paths.
6. Confirm state files and test evidence are current.
7. Perform a separate gap pass after tests are green.
8. Fix discovered gaps and rerun affected tests.

A green compile is never sufficient evidence of completion.

---

## FILE: .agents/skills/security-review/SKILL.md

---
name: security-review
description: Threat-model and security-audit VPNBridge pairing, local gateway protocol, socket routing, secrets, kill switch, logs, and dependency supply chain.
---
# Security Review Skill

Use before exposing a new network surface and at each release candidate.

Review:

- unauthorized hotspot clients,
- gateway authentication bypass,
- replay,
- malformed SOCKS/control packets,
- UDP amplification/abuse,
- resource exhaustion,
- route/DNS/IPv6 leaks,
- VPN split-tunnel exclusion,
- VPN disconnect races,
- stale Android `Network` handles,
- Windows kill-switch bypass,
- privilege escalation and unsafe service IPC,
- secrets at rest,
- log privacy,
- update/package integrity,
- dependency provenance/license/security.

Require bounded inputs, fail-closed state transitions, least privilege, secure random values, secret storage, and adversarial tests.

---

## FILE: docs/00_PRODUCT_REQUIREMENTS.md

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

---

## FILE: docs/01_SYSTEM_ARCHITECTURE.md

# 01 — System Architecture

## Components

### Android UI

Tauri 2 + Svelte 5. UI is control-plane only; it must never own long-lived packet loops.

### Android native bridge

Kotlin Tauri plugin/service layer for:

- Local-Only Hotspot lifecycle,
- permissions,
- `ConnectivityManager` callbacks,
- VPN `Network` selection and `NetworkCapabilities`,
- VPN `LinkProperties` including DNS,
- network-handle/socket-binding bridge,
- foreground service and notification,
- Android lifecycle events.

### Shared Rust gateway core

Responsibilities:

- client authentication/session state,
- SOCKS5-compatible proxy semantics,
- TCP forwarding,
- UDP association/flow table,
- bounded queues/backpressure,
- counters/telemetry,
- cancellation and shutdown,
- protocol parsing with strict limits.

### Windows companion

Tauri 2/Svelte 5 UI plus Rust networking/service layer.

Subsystems:

- phone discovery/manual connect,
- pairing,
- local secure transport,
- Wintun lifecycle,
- TUN-to-SOCKS engine,
- route manager,
- WFP kill switch,
- DNS/IPv6 policy,
- recovery/watchdog,
- diagnostics.

## Data flow

### TCP

```text
Windows app
 -> Wintun
 -> TUN-to-SOCKS
 -> phone local gateway
 -> Android creates upstream socket
 -> socket is bound to active VPN Network
 -> destination
```

### UDP

```text
Windows datagram
 -> Wintun
 -> TUN-to-SOCKS UDP path
 -> authenticated phone UDP association
 -> Android VPN-bound UDP socket
 -> destination
```

### DNS

Preferred order:

1. Use DNS servers advertised in `LinkProperties` of the validated VPN network and send via VPN-bound sockets.
2. Optionally use a user-selected DoH/DoT resolver over a VPN-bound connection.
3. Never send DNS directly over the physical Android upstream or raw Windows interface while protected mode is active.

## Control flow

The phone is authoritative about whether forwarding is safe. It sends signed/authenticated state updates to Windows. Windows must treat missing/expired heartbeats as unsafe.

## Isolation

The Local-Only Hotspot intentionally has no Internet route of its own. It is used as a point-to-point-like local transport. Windows should not assume the phone hotspot gateway address; discover it from the actual interface/configuration and allow manual fallback.

## Concurrency

- One Rust runtime per app/service process.
- One supervisor task owns lifecycle cancellation.
- Per-session and per-flow tasks are children of a cancellation tree.
- All channels are bounded.
- UDP flow table has idle expiry and maximum size.
- TCP connection count has configurable safety caps.

## Error boundaries

Any error involving VPN validity, socket-to-network binding, route installation, kill switch state, or authentication is safety-critical and transitions to fail-closed behavior.

Ordinary destination errors are per-flow and must not crash the gateway.

---

## FILE: docs/02_REPOSITORY_STRUCTURE.md

# 02 — Repository Structure

Recommended monorepo:

```text
vpnbridge/
├─ .agents/
│  ├─ AGENTS.md
│  └─ skills/
├─ apps/
│  ├─ android/
│  │  ├─ src/                      # Svelte UI
│  │  ├─ src-tauri/                # Tauri/Rust shell
│  │  └─ android-native/           # Kotlin plugin/service code
│  └─ windows/
│     ├─ src/
│     └─ src-tauri/
├─ crates/
│  ├─ core/                        # common types/state/error model
│  ├─ gateway/                     # SOCKS/TCP/UDP forwarding
│  ├─ protocol/                    # control/pairing protocol
│  ├─ metrics/                     # counters/snapshots
│  ├─ android-netbind/             # Android fd->Network bridge wrapper
│  ├─ windows-tun/                 # Wintun abstraction
│  ├─ windows-routing/             # routes/DNS/kill switch
│  └─ test-support/                # fake networks, servers, packet tests
├─ docs/
├─ tests/
│  ├─ integration/
│  ├─ leak/
│  ├─ performance/
│  └─ fixtures/
├─ tools/
│  ├─ echo-server/
│  ├─ dns-test-server/
│  └─ traffic-generator/
├─ .github/workflows/              # if GitHub is used
├─ Cargo.toml
├─ Cargo.lock
├─ package.json/pnpm-workspace.yaml
├─ README.md
├─ PROGRESS.md
├─ DECISIONS.md
├─ KNOWN_ISSUES.md
└─ TEST_EVIDENCE.md
```

## Boundaries

### `core`

No platform UI dependencies. Defines states, configuration, errors, cancellation, and immutable snapshots.

### `gateway`

No Tauri dependency. It receives a platform-provided upstream socket factory/binder interface so tests can substitute a fake network.

### Android native

Must not contain general proxy business logic. It should expose narrow commands/events to Rust.

### Windows routing

All route/firewall changes must be transactional and reversible. Persist a recovery journal before making destructive network changes.

## Dependency direction

UI -> app shell -> domain/core -> platform abstraction.

Platform modules may implement interfaces from core; core never imports Tauri/Kotlin/Win32 UI code.

---

## FILE: docs/03_ANDROID_APP_SPEC.md

# 03 — Android Application Specification

## Screens

### Home

Show:

- VPN status: Protected / Not protected / Revalidating.
- Hotspot status.
- Windows connection status.
- Protected sharing status.
- current throughput.
- primary **Share VPN** / **Stop** action.

### Pairing

- Local hotspot SSID.
- QR payload containing only local connection metadata and a short-lived pairing token; never expose long-term private keys.
- connected/pending device list.

### Diagnostics

- Android API/device/OEM.
- upstream type.
- VPN transport detected.
- VPN DNS servers (redacted/optional display).
- local hotspot address.
- last failure code.
- export sanitized diagnostic bundle.

### Settings

- auto-reconnect.
- Proxy Mode / Full Tunnel preference hint.
- DNS policy.
- IPv6 policy.
- maximum sessions.
- diagnostics level.

## Native service

Use a foreground service appropriate for continuous interaction with an external device over a network connection. Verify current Android requirements before implementation.

The service owns:

- hotspot reservation,
- VPN callbacks,
- validated VPN network handle,
- Rust gateway lifecycle,
- notification,
- session state.

The Activity/UI may die without terminating an active sharing session unless the user explicitly stops it.

## Permissions

At minimum evaluate and request only what current APIs require, including:

- INTERNET
- ACCESS_NETWORK_STATE
- CHANGE_WIFI_STATE / CHANGE_NETWORK_STATE as justified
- NEARBY_WIFI_DEVICES for Local-Only Hotspot on Android 13+
- FOREGROUND_SERVICE
- FOREGROUND_SERVICE_CONNECTED_DEVICE for target SDKs requiring it
- POST_NOTIFICATIONS where applicable to UX/OS behavior

Do not add privileged tethering permissions.

## Android 15 behavior

Avoid `dataSync` foreground-service type for an indefinitely running gateway. Android 15 limits dataSync background foreground-service time. Use the service type that truthfully represents connected-device/network interaction and satisfy its prerequisites.

## Lifecycle

- Start only from clear user action.
- Keep persistent notification while sharing.
- Handle task removal without corrupting service state.
- Handle OS process recreation.
- On service destruction, close gateway and hotspot reservation safely.
- On hotspot loss, notify Windows and fail closed.

---

## FILE: docs/04_ANDROID_NETWORKING_SPEC.md

# 04 — Android Networking Specification

This is the most safety-critical Android document.

## 1. Local-Only Hotspot

Use the public `WifiManager.startLocalOnlyHotspot(...)` API or its current supported equivalent.

Properties:

- local communication only,
- no direct Internet connectivity for hotspot clients,
- OS-provided SSID/security configuration,
- reservation object controls lifetime.

Never replace this with privileged system tethering in the primary design.

## 2. VPN discovery

VPNBridge must determine the network that is the default route for its own UID.

Preferred flow:

1. `ConnectivityManager.getActiveNetwork()` for initial state.
2. `getNetworkCapabilities(network)`.
3. require `hasTransport(NetworkCapabilities.TRANSPORT_VPN)`.
4. inspect capability changes and `LinkProperties` via callbacks.
5. retain the validated `Network` and its network handle only while current.

If the active network for VPNBridge is Wi-Fi/cellular rather than VPN, assume VPNBridge is excluded/split-tunneled or VPN is absent and disable forwarding.

## 3. Upstream socket binding — mandatory

Before an Internet-facing socket connects or a datagram is sent, bind it to the validated VPN `Network`.

Possible implementation paths:

- Kotlin/Java `Network.bindSocket(Socket/DatagramSocket/FileDescriptor)`, or
- NDK `android_setsocknetwork(networkHandle, fd)` using the handle obtained from Android.

Rust abstraction example conceptually:

```text
trait ProtectedSocketBinder {
    fn bind_tcp_fd_to_current_vpn(fd) -> Result<VpnGeneration>;
    fn bind_udp_fd_to_current_vpn(fd) -> Result<VpnGeneration>;
}
```

The binder must verify that the network generation is still current before allowing the flow to become active.

## 4. Never use bypass APIs

In external-VPN mode do not call `VpnService.protect()` for gateway upstream sockets. Its purpose is to bypass VPN routing, which is the opposite of this product's requirement.

Do not bind upstream sockets to the physical Wi-Fi/cellular network.

## 5. Local listeners

The SOCKS/control listener must bind only to the hotspot-local IP/interface.

Do not call `ConnectivityManager.bindProcessToNetwork(hotspotNetwork)` because that risks sending subsequently created upstream sockets through the local/underlying network.

If a specific local interface binding is required, scope it to the listener socket or local address only.

## 6. VPN generation model

Represent each validated VPN network as a monotonically increasing generation:

```text
VpnBinding {
  generation,
  network_handle,
  capabilities_hash,
  dns_snapshot,
  validated_at
}
```

Every active upstream flow records the generation it was created under.

On network loss/replacement:

1. increment generation / invalidate old binding,
2. set global forwarding gate false,
3. cancel all flows from old generation,
4. clear UDP mappings,
5. revalidate new VPN,
6. resume only after validation.

## 7. DNS

Read VPN `LinkProperties` DNS servers. Route DNS to those servers using VPN-bound sockets. For domain-based SOCKS requests, either resolve on the VPN network or pass domain names through a resolver that is itself VPN-bound.

## 8. Concurrent Wi-Fi caveat

Cellular upstream + Local-Only Hotspot is the primary baseline. Wi-Fi upstream + hotspot requires hardware/OEM STA+AP concurrency. Detect actual behavior and include it in the device matrix; never claim universal support without evidence.

## 9. Network race tests

Automate/perform:

- VPN disconnect during TCP transfer.
- VPN disconnect during rapid new TCP connects.
- VPN disconnect during UDP flood.
- VPN server change causing network replacement.
- upstream cellular/Wi-Fi transition while VPN reconnects.
- app split-tunnel exclusion.
- hotspot restart while VPN remains active.

The required result is fail-closed forwarding.

---

## FILE: docs/05_GATEWAY_PROTOCOL.md

# 05 — Gateway and Pairing Protocol

## MVP protocol

Use standards where possible to reduce risk.

### Data plane

Authenticated SOCKS5:

- CONNECT for TCP.
- UDP ASSOCIATE for UDP.
- domain-name address type supported.
- strict parser and length limits.
- no unauthenticated production listener.

### Control plane

A small versioned binary or compact CBOR-like protocol may carry:

- protocol version,
- device identity,
- pairing challenge/response,
- heartbeat,
- protected/unprotected state,
- DNS/IPv6 capability flags,
- throughput counters,
- controlled shutdown/reconnect messages.

Do not use verbose JSON per packet.

## Authentication stages

### Development/MVP

- OS-random 256-bit session secret.
- short-lived pairing code derived from a cryptographic challenge, not the secret itself.
- rate-limited failed authentication.
- listener available only on hotspot-local interface.

### Production

- long-term device key pair stored in platform secure storage.
- authenticated ephemeral session handshake.
- forward-secret secure channel using a well-reviewed TLS 1.3 or Noise implementation.
- per-session keys.
- sequence/replay protection.
- key rotation and unpair support.

Do not design custom cryptography.

## Protocol versioning

Every session begins with explicit version negotiation. Unknown major versions fail closed with a clear upgrade error. Minor versions may add optional capabilities.

## Resource limits

Configurable hard caps:

- maximum paired clients,
- maximum concurrent TCP streams,
- maximum UDP mappings,
- maximum control frame size,
- maximum domain length,
- authentication attempts per minute,
- per-flow idle timeouts.

## UDP

Maintain NAT-like association state keyed by client session + source tuple. Expire idle mappings. Prevent reflection by sending replies only to authenticated associations and destinations initiated by the client.

## Backpressure

Do not buffer unbounded data if phone VPN or Windows link slows. Apply bounded channel capacity and suspend reads or close abusive flows.

---

## FILE: docs/06_WINDOWS_COMPANION_SPEC.md

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

---

## FILE: docs/07_DNS_IPV6_LEAK_PROTECTION.md

# 07 — DNS, IPv6, and Leak Protection

## Principle

No protocol family or resolver path may escape merely because the primary TCP path is protected.

## DNS policy

Default: **Follow Android VPN**.

1. Obtain DNS servers from the validated VPN network's `LinkProperties`.
2. Send DNS requests through Android sockets bound to that VPN network.
3. If DNS server addresses change with the VPN network generation, invalidate cached configuration.
4. Cache only with correct TTL behavior and bounded memory.

Optional modes:

- user-selected DoH resolver over a VPN-bound TLS connection,
- user-selected DoT resolver over a VPN-bound TLS connection.

Never use raw Windows DNS while Full Tunnel is protected.

## DNS tests

- A/AAAA lookups.
- NXDOMAIN.
- large responses / TCP fallback.
- DNS change after VPN server switch.
- resolver unavailable.
- malformed response handling.
- repeated VPN drop during lookups.

## IPv6 policy

### Release option 1

Full dual-stack forwarding through VPNBridge.

### Release option 2

Explicit IPv6 block while protected if dual-stack is not ready.

No release may have implicit IPv6 bypass.

## Leak harness

Continuously generate:

- HTTPS/TCP requests to an IP echo endpoint,
- UDP probes to a controlled endpoint,
- DNS queries,
- IPv6 requests,
- frequent new connections.

Then repeatedly:

- disconnect/reconnect VPN,
- switch VPN servers,
- toggle phone upstream,
- restart VPNBridge gateway,
- suspend/resume Windows.

Record every successful response and observed public source address. Any response through raw ISP egress while protected/fail-closed state is expected is a release-blocking defect.

---

## FILE: docs/08_SECURITY_THREAT_MODEL.md

# 08 — Security Threat Model

## Assets

- protected Internet path,
- pairing/device identity keys,
- hotspot credentials,
- session secrets,
- Windows routing/firewall state,
- user traffic confidentiality/integrity,
- privacy-sensitive diagnostics.

## Trust boundaries

1. Windows application traffic -> privileged Windows tunnel component.
2. Windows tunnel -> local hotspot link.
3. Local client -> Android gateway parser/authentication.
4. Android Rust core -> Android network-binding bridge.
5. Android VPNBridge -> third-party VPN network.
6. Build system -> third-party dependencies/binaries.

## Threats and mitigations

### Unauthorized hotspot client uses gateway

Mitigate with app-layer authentication, limited listener interface, rate limits, paired device identity, and credential rotation.

### VPN disappears and raw Internet is used

Mitigate by explicit per-socket binding to the validated VPN `Network`, generation invalidation, immediate flow cancellation, Windows kill switch, and leak tests.

### VPN excludes VPNBridge by split tunneling

Require VPN transport for VPNBridge's active default network. Disable forwarding if absent.

### DNS/IPv6 bypass

Route/protect explicitly; block unsupported families.

### Malformed SOCKS/control input

Strict parser, bounded lengths, fuzzing, no unchecked indexing, no panic on network input.

### Resource exhaustion

Connection limits, UDP flow caps, memory limits, idle expiry, authentication throttling, bounded queues.

### Local MITM

Production authenticated secure channel; do not trust hotspot encryption alone.

### Windows local privilege attack

Minimal privileged helper, authenticated IPC, fixed command schema, no arbitrary command execution.

### Supply-chain compromise

Lockfiles, checksums/provenance for bundled binary assets, vulnerability scans, license review, minimal dependency count, reproducible release procedure.

### Sensitive logs

Redact credentials/keys and avoid visited-domain logging by default. Diagnostics should use event codes and aggregate counters.

## Security gates

- dependency audit,
- static analysis,
- fuzz/property tests,
- protocol abuse tests,
- privilege boundary review,
- route/killswitch bypass tests,
- manual threat-model re-review before release.

---

## FILE: docs/09_UI_UX_SPEC.md

# 09 — UI/UX Specification

## Design goal

The networking is sophisticated; the user flow must not be.

## Android primary screen

Status hierarchy:

1. **VPN protection**
2. **Private hotspot**
3. **Laptop connection**
4. **Sharing state**

Primary button states:

- `Share VPN`
- `Preparing...`
- `Waiting for VPN`
- `Protected — Sharing`
- `Reconnecting...`
- `Stop Sharing`

Do not show a green/protected state unless the forwarding gate is actually safe.

## Windows primary screen

Show:

- paired phone,
- phone VPN protection state,
- connection mode: Proxy / Full Tunnel,
- kill switch: on/off,
- DNS: protected/blocking,
- IPv6: tunneled/blocked,
- throughput,
- Connect/Disconnect.

## Error UX

Every error must include:

- short human description,
- whether Internet is blocked or still safe,
- one recommended action,
- expandable technical code.

Examples:

- `VPNBridge is excluded from your Android VPN. Include VPNBridge in the VPN and try again.`
- `The phone VPN disconnected. Internet forwarding is blocked until protection returns.`
- `This phone cannot keep Wi-Fi upstream active while running the private hotspot. Use mobile data or a supported device.`

## Accessibility

- keyboard navigation on Windows,
- appropriate contrast,
- screen-reader labels,
- no status conveyed by color alone,
- minimum touch targets on Android,
- reduced-motion respect.

## UI performance

Update throughput counters at a human-friendly interval (for example 2–4 times/second), not per packet. Keep packet processing completely outside the WebView/UI thread.

---

## FILE: docs/10_PERFORMANCE_TARGETS.md

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

---

## FILE: docs/11_TEST_STRATEGY.md

# 11 — Test Strategy

## Testing layers

### Rust unit tests

- SOCKS5 parsing/encoding.
- state machines.
- authentication logic.
- UDP association lifecycle.
- config validation.
- network-generation cancellation.
- bounded queues/timeouts.

### Property/fuzz tests

Target every parser and externally supplied length/state field. Network input must never panic.

### Android tests

- permission flow.
- Local-Only Hotspot lifecycle.
- VPN detection.
- network callback ordering.
- VPN-network socket binding.
- foreground-service lifecycle.
- screen off/background.
- process/service recreation.

### Windows tests

- Wintun lifecycle abstraction.
- route transaction/rollback.
- recovery journal.
- kill-switch rules.
- DNS configuration restoration.
- elevation/helper IPC.

### End-to-end tests

- HTTP/HTTPS.
- WebSocket.
- large download/upload.
- UDP echo.
- DNS.
- QUIC/HTTP3 where tunnel supports UDP.
- IPv6 or explicit block.
- simultaneous streams.

## Continuous vs exhaustive testing

For speed, run focused deterministic tests after each change. Run the expensive device/network matrix at phase gates and especially before release. Do not defer basic correctness tests until the end.

## Failure-injection tests

Kill/restart:

- Android UI,
- Android foreground service,
- Windows UI,
- Windows privileged helper,
- TUN engine.

Toggle:

- Android VPN,
- phone airplane mode,
- cellular/Wi-Fi,
- Local-Only Hotspot,
- Windows Wi-Fi,
- sleep/resume.

## Test evidence

Every release gate records:

- git commit,
- device/OS,
- VPN provider/protocol,
- commands/test IDs,
- pass/fail,
- relevant sanitized logs,
- performance measurements.

---

## FILE: docs/12_DEVICE_VPN_COMPATIBILITY_MATRIX.md

# 12 — Device and VPN Compatibility Matrix

Maintain a living matrix. Do not claim universal compatibility from emulator-only testing.

## Android dimensions

- Android 15 stock/near-stock device.
- Samsung One UI Android 15.
- Xiaomi/HyperOS Android 15 where available.
- OnePlus/OxygenOS Android 15 where available.
- Pixel Android 15 reference device.

For each record:

- device model,
- build number,
- API level,
- Local-Only Hotspot works,
- hotspot local address/subnet behavior,
- cellular upstream works,
- Wi-Fi upstream concurrency works,
- screen-off stability,
- OEM battery restrictions,
- notes.

## VPN dimensions

Test several implementations/protocols, for example:

- WireGuard-based client,
- OpenVPN-based client,
- ProtonVPN,
- Mullvad,
- another popular commercial VPN if available.

Record:

- VPNBridge active network reports `TRANSPORT_VPN`,
- VPN LAN/local-sharing setting needed,
- split-tunnel behavior,
- TCP,
- UDP,
- DNS,
- IPv6,
- server-switch recovery.

## Required v1 baseline

At least one stock/near-stock Android 15 device plus two materially different VPN implementations must pass the full acceptance matrix before claiming a broadly usable beta.

---

## FILE: docs/13_CI_CD_RELEASE.md

# 13 — CI/CD and Release Engineering

## CI objectives

Every pull request/change set should run feasible automated checks:

- Rust format.
- clippy with warnings treated strictly.
- Rust tests.
- frontend lint/typecheck/test.
- Android compile/build/unit tests where runner supports it.
- Windows cross/build checks where runner supports it.
- dependency vulnerability scan.
- license policy scan.
- secret scan.

## Nightly/extended

- fuzz corpus regression.
- integration test suite.
- sanitizer runs where supported.
- dependency update report.

## Hardware test lane

Physical-device tests cannot be faked by a cloud build. Maintain an explicit hardware release lane using connected Android + Windows hardware with recorded evidence.

## Build reproducibility

- commit lockfiles,
- pin toolchain/channel where practical,
- record Android SDK/NDK/JDK versions,
- checksum bundled Wintun binary distribution,
- record third-party binary provenance,
- deterministic version stamping.

## Signing

Never commit signing keys/secrets. Use platform secret storage/CI protected secrets. Debug builds must be visibly distinguishable from release builds.

## Release stages

1. developer build,
2. internal alpha,
3. leak-test alpha,
4. compatibility beta,
5. release candidate,
6. stable.

Each promotion requires all gates assigned to that stage.

---

## FILE: docs/14_OBSERVABILITY_LOGGING.md

# 14 — Observability and Logging

## Goals

Logs must make race conditions and routing failures diagnosable without becoming a privacy liability.

## Structured events

Use stable event names/codes, for example:

- `ANDROID_VPN_FOUND`
- `ANDROID_VPN_INVALID`
- `ANDROID_VPN_LOST`
- `HOTSPOT_STARTED`
- `HOTSPOT_STOPPED`
- `UPSTREAM_SOCKET_BOUND`
- `UPSTREAM_BIND_FAILED`
- `CLIENT_PAIRED`
- `CLIENT_AUTH_FAILED`
- `TCP_FLOW_OPEN/CLOSE`
- `UDP_ASSOC_OPEN/CLOSE`
- `WINDOWS_TUN_UP/DOWN`
- `WINDOWS_KILLSWITCH_ON/OFF`
- `ROUTE_TRANSACTION_ROLLBACK`

## Required fields

- monotonic timestamp,
- session ID (random, non-secret),
- VPN generation,
- component,
- event code,
- severity,
- error code,
- bounded diagnostic metadata.

## Privacy

Do not log by default:

- passwords/tokens/private keys,
- full browsing history,
- full DNS query history,
- packet payloads,
- Authorization headers.

Domain/IP logging for debugging must be opt-in, time-limited, and clearly marked.

## Diagnostics bundle

Export a sanitized bundle containing:

- app versions,
- OS/device info,
- current config excluding secrets,
- route state summaries,
- VPN/hotspot state transitions,
- recent error events,
- test/self-check results.

## Metrics

Counters/gauges:

- active TCP flows,
- active UDP mappings,
- bytes up/down,
- rejected auth attempts,
- bind failures,
- VPN generation changes,
- reconnects,
- queue depth/high-water marks,
- dropped packets/datagrams,
- memory/CPU samples in diagnostics mode.

---

## FILE: docs/15_BUILD_ENVIRONMENT.md

# 15 — Build and Development Environment

## Toolchain

Antigravity must verify the latest compatible versions before scaffolding and then pin/document the selected versions.

Expected tools:

- Rust stable + rustfmt + clippy,
- Node.js LTS,
- pnpm or another single standardized package manager,
- Tauri 2 CLI,
- Android Studio/JBR,
- Android SDK Platform/Build Tools/Platform Tools,
- Android NDK required by Tauri/Rust mobile build,
- Windows SDK/Visual Studio Build Tools for Windows builds,
- Wintun distribution for Windows integration.

## Android target policy

The primary runtime requirement is Android 15/API 35. `minSdk`, `compileSdk`, and `targetSdk` must be selected from current Tauri/Android/Play requirements rather than copied blindly from this document.

If targeting newer SDKs, account for newer local-network permission behavior. Keep Android 15 functional.

## Environment checks

Create a developer preflight command/script later that verifies:

- rustc/cargo,
- Node/package manager,
- Java,
- Android SDK/NDK paths,
- adb,
- Tauri CLI,
- Windows SDK when on Windows,
- required native libraries.

## Antigravity environment limitation handling

If Antigravity is running in a remote Linux sandbox, it may be able to compile/test much of the shared and Android code but cannot truthfully execute physical Android/Windows integration tests. Mark those as `HUMAN_GATE` or hardware-lab work while continuing all software-only tasks.

If Antigravity is running locally on the development Windows machine with Android attached through ADB, automate device builds/log capture/tests directly.

---

## FILE: docs/16_IMPLEMENTATION_PHASES.md

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

---

## FILE: docs/17_AUTONOMOUS_EXECUTION_LOOP.md

# 17 — Autonomous Execution Loop

## Purpose

Prevent the agent from stopping after partial implementation, repeatedly asking routine questions, or losing project context.

## Persistent loop

Repeat until Definition of Done or a genuine external blocker:

```text
INSPECT
  -> SELECT highest-priority unblocked requirement
  -> RESEARCH uncertain APIs
  -> DESIGN smallest coherent change
  -> IMPLEMENT
  -> FORMAT/LINT/BUILD
  -> TARGETED TESTS
  -> FAILURE ANALYSIS + FIX LOOP
  -> INTEGRATION TESTS
  -> GAP HUNT
  -> UPDATE STATE/DOCS
  -> COMMIT-READY CHECKPOINT
  -> SELECT NEXT TASK
```

## Priority order

1. safety/correctness blocker,
2. central feasibility assumption,
3. failing build/test,
4. security issue,
5. release-blocking functionality,
6. resilience/recovery,
7. performance,
8. UX polish,
9. optional enhancement.

## Decision hierarchy

When multiple implementation choices exist:

1. hard project requirements,
2. official current platform documentation,
3. security/fail-closed behavior,
4. empirical experiment/benchmark,
5. maintainability/simplicity,
6. performance,
7. aesthetic preference.

Record material decisions in `DECISIONS.md`.

## Stop conditions

The agent may pause only when:

- user credentials/signing secrets are required,
- a physical device action cannot be automated,
- external paid/service access is required,
- two product choices have materially different user behavior and requirements do not choose one,
- legal/licensing approval is required.

Even then, continue every other unblocked task.

## No fake progress

Do not mark tasks complete based on generated code that has not compiled/tested where testing is possible. Do not substitute mocks for release evidence.

## Context recovery

At the start of every new Antigravity session:

1. read `.agents/AGENTS.md`,
2. read `PROGRESS.md`, `DECISIONS.md`, `KNOWN_ISSUES.md`,
3. inspect git status/history,
4. run a fast health check,
5. resume the highest-priority unblocked task.

---

## FILE: docs/18_DEFINITION_OF_DONE.md

# 18 — Definition of Done

The project is not done until every applicable item below has objective evidence.

## Architecture

- [ ] No-root design implemented.
- [ ] Local-Only Hotspot is primary local link.
- [ ] Android upstream sockets are explicitly bound to a validated VPN `Network`.
- [ ] No production path binds upstream traffic to raw Wi-Fi/cellular.
- [ ] No external-VPN gateway path calls a VPN bypass API.

## Android

- [ ] VPN detection and split-tunnel exclusion detection work.
- [ ] Hotspot lifecycle works.
- [ ] foreground service survives expected UI lifecycle.
- [ ] TCP works.
- [ ] UDP works.
- [ ] DNS policy works.
- [ ] VPN loss cancels old-generation flows.
- [ ] VPN return safely resumes.
- [ ] permissions are least-privilege and current.

## Windows

- [ ] pairing works.
- [ ] Proxy Mode works.
- [ ] Full Tunnel/Wintun works.
- [ ] TCP/UDP/DNS full tunnel works.
- [ ] IPv6 is tunneled or explicitly blocked.
- [ ] kill switch prevents raw fallback.
- [ ] route/DNS state is restored on disconnect.
- [ ] crash/reboot recovery works.
- [ ] uninstall leaves networking healthy.

## Security

- [ ] production gateway requires app-layer authentication.
- [ ] long-term secrets use secure storage.
- [ ] parser fuzz/property tests pass.
- [ ] rate/resource limits exist.
- [ ] dependency vulnerabilities reviewed.
- [ ] licenses/provenance reviewed.
- [ ] logs contain no secrets.

## Leak tests

- [ ] repeated VPN-drop TCP test passes.
- [ ] UDP test passes.
- [ ] DNS leak test passes.
- [ ] IPv6 leak test passes.
- [ ] Windows TUN crash test passes.
- [ ] Android gateway crash test passes safely.
- [ ] VPN server-change test passes.

## Quality

- [ ] format/lint/type checks pass.
- [ ] unit/integration tests pass.
- [ ] no critical TODO/FIXME/stub path.
- [ ] no ignored critical errors.
- [ ] 8-hour soak has no unbounded resource growth.
- [ ] supported device/VPN matrix recorded.
- [ ] final independent gap audit completed.
- [ ] no unresolved critical/high issue.

## Release

- [ ] reproducible build inputs recorded.
- [ ] signed installers/APK as applicable.
- [ ] clean install works.
- [ ] upgrade works.
- [ ] rollback/recovery documented.
- [ ] known limitations are explicit.

---

## FILE: docs/19_RISK_REGISTER.md

# 19 — Risk Register

| Risk | Impact | Mitigation / Experiment |
|---|---|---|
| VPN app excludes VPNBridge via split tunnel | Critical leak risk | Require active default `TRANSPORT_VPN`; bind upstream sockets to VPN Network; fail closed |
| VPN disappears between validation and socket use | Critical | VPN generation + bind-before-connect + network callbacks + flow cancellation + leak tests |
| Local-Only Hotspot cannot coexist with Wi-Fi upstream on device | Medium | cellular baseline; capability/device matrix; clear UX |
| VPN blocks local LAN communication | High usability | detect gateway reachability; user guidance for VPN LAN/local sharing setting; compatibility matrix |
| OEM kills foreground service | High | correct FGS type, notification, lifecycle tests, targeted OEM guidance |
| UDP/QUIC instability | Medium | bounded UDP associations, timeouts, soak/perf tests |
| DNS leak | Critical | VPN DNS snapshot + VPN-bound DNS + Windows DNS routing + leak harness |
| IPv6 leak | Critical | dual-stack implementation or explicit block |
| Wintun integration/license misuse | High | use official signed distribution, preserve license, verify current upstream instructions |
| Windows route changes strand user offline | High | transactional snapshot/recovery journal/rollback/uninstall tests |
| Kill switch blocks phone gateway | High | explicit local bypass rule before enforcement; recovery safe mode |
| TUN-to-SOCKS dependency regression | Medium | abstraction, pin version, integration tests, ability to replace |
| Protocol/parser vulnerability | High | standard SOCKS semantics, strict bounds, fuzzing, secure channel |
| Antigravity assumes unverified API | High | mandatory primary-source research + micro-experiments |
| Remote agent cannot hardware-test | High | explicit human/hardware gate; never fake evidence |

---

## FILE: docs/20_DEPENDENCY_LICENSING_POLICY.md

# 20 — Dependency and Licensing Policy

## Selection rules

Prefer dependencies that are:

- actively maintained,
- security-conscious,
- small enough to audit,
- permissively licensed for the intended distribution model,
- reproducibly obtainable,
- compatible with Android/Windows targets.

## Required review before adoption

For each nontrivial dependency record in `DECISIONS.md`:

- package/repository,
- exact version/commit,
- purpose,
- license,
- current maintenance activity,
- known advisories,
- transitive dependency impact,
- replacement strategy.

## `tun2proxy`

Treat as a candidate, not an unquestioned requirement. At implementation time verify current:

- Rust crate version,
- Windows/Wintun support,
- SOCKS5 UDP behavior,
- IPv4/IPv6 behavior,
- license,
- advisories/issues,
- library embedding feasibility.

Keep it behind an internal TUN-to-proxy interface.

## Wintun

Use official signed distribution and include its exact accompanying license. Record checksum and source release. Do not build/distribute an unofficial driver unless there is a separately reviewed need.

## Security automation

Run available equivalents of:

- `cargo audit`,
- dependency-deny/license checks,
- JS package audit,
- secret scanning,
- provenance/checksum validation.

No release with an unreviewed high/critical vulnerability in an applicable dependency.

---

## FILE: docs/21_FAILURE_RECOVERY.md

# 21 — Failure and Recovery Specification

## Android VPN loss

Expected behavior:

- VPN callback invalidates current generation.
- forwarding gate closes.
- old upstream sockets close.
- UDP associations clear.
- Windows receives unsafe state if control path remains.
- no new egress is permitted.
- safe revalidation begins when VPN returns.

## Android hotspot loss

- stop local listener/session.
- Windows stays fail closed.
- attempt hotspot recreation only from permitted lifecycle context.
- communicate new SSID/subnet/pairing metadata as needed.

## Android process/service crash

No raw Internet route exists through Local-Only Hotspot. Windows kill switch remains fail closed. On restart, a new authenticated session is required/resumed according to protocol.

## Windows UI crash

Privileged tunnel/kill-switch component should have a defined ownership model. Do not leave raw routing exposed. A service may keep protection or fail closed until explicit recovery.

## Windows privileged helper/TUN crash

- kill switch remains on,
- route transaction is not silently reverted to raw Internet,
- watchdog attempts safe restart,
- recovery UI explains state.

## Windows reboot with stale journal

At startup, helper detects unfinished transaction and either:

- restores original network state if no protected session exists, or
- reconstructs protected state only after phone/VPN validation.

Never leave ambiguous partial routes.

## Intentional disconnect

Order:

1. stop new TUN flows,
2. remove protected routes,
3. remove/relax kill switch at the correct safe point,
4. restore DNS/interface state,
5. verify normal connectivity restoration,
6. clear recovery journal.

---

## FILE: docs/22_PRIVACY_DISTRIBUTION.md

# 22 — Privacy, Permissions, and Distribution

## Privacy posture

VPNBridge should work locally without an account or cloud relay. Avoid telemetry by default unless the product owner later explicitly requests it.

## Data minimization

Persist only what is needed:

- paired device identity/public key,
- user settings,
- sanitized diagnostics metadata,
- recovery journal on Windows.

Do not persist browsing history.

## Android permissions

Every permission must map to a documented feature/API requirement. Remove unused permissions before release.

## Android/Play policy

Before Play distribution, Antigravity must verify the current Google Play policies for foreground services, device/network behavior, VPN-related claims, target SDK, privacy declarations, and data safety. Do not rely on stale policy text in this repository.

## Windows distribution

- signed executable/installer for public release,
- minimal elevation,
- transparent Wintun/native component licensing,
- clean uninstall and network-state restoration.

## Product claims

Do not market the product as guaranteeing anonymity or universal censorship bypass. It forwards traffic through the user's selected VPN; actual reachability/privacy depends on that VPN, destination, network, and platform.

---

## FILE: docs/23_FUTURE_BUILT_IN_VPN.md

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

---

## FILE: docs/24_RESEARCH_DECISIONS_LOG.md

# 24 — Initial Research and Decision Notes

These are starting points. Antigravity must re-check current primary documentation when implementing.

## Antigravity customization

Google's current Antigravity/managed-agent documentation supports project instructions via `.agents/AGENTS.md` and skills under `.agents/skills/<name>/SKILL.md`. This repository uses that mechanism deliberately.

## Android Local-Only Hotspot

Current Android documentation states that Local-Only Hotspot provides local communication without Internet access. For apps targeting Android 13+, `NEARBY_WIFI_DEVICES` is required for this API.

Decision: use Local-Only Hotspot rather than privileged system tethering.

## Android VPN routing

Android documentation states that an app's traffic continues through a VPN unless it deliberately binds/bypasses to another network, while `VpnService.protect()` specifically sends a socket outside the VPN.

Android `Network.bindSocket`/NDK network binding can constrain a socket to a particular network, and network-bound sockets fail when that network goes away.

Decision: explicitly bind each gateway upstream socket to the current validated `TRANSPORT_VPN` network for stronger fail-closed behavior.

## Android foreground service

Android 15 places a cumulative background timeout on `dataSync` foreground services. `connectedDevice` is documented for interaction with external devices over network connections and has its own permission/prerequisite rules.

Decision: evaluate/use the truthful connected-device service type rather than misclassifying the persistent gateway as dataSync.

## Windows

Wintun is a small Layer-3 TUN driver intended for userspace tunneling applications and has signed distributable binaries under accompanying terms.

Current `tun2proxy` is a Rust TUN-to-HTTP/SOCKS project supporting Windows/Wintun, IPv4/IPv6, and SOCKS5 UDP.

Decision: evaluate it first before implementing a new userspace network stack.

---

## FILE: docs/25_ACCEPTANCE_TEST_MATRIX.md

# 25 — Acceptance Test Matrix

Every row applicable to the release must have recorded evidence.

| ID | Test | Expected result |
|---|---|---|
| A01 | Android VPN absent, user starts sharing | Forwarding disabled; clear error |
| A02 | VPN active and covers VPNBridge | Protected state becomes available |
| A03 | VPN excludes VPNBridge via split tunnel | Sharing blocked |
| A04 | Start/stop Local-Only Hotspot repeatedly | No leaked reservation/resource; reconnect works |
| A05 | TCP HTTPS through manual SOCKS | Public egress is VPN path |
| A06 | Large TCP download/upload | Correct data; stable memory |
| A07 | UDP echo | Works through VPN-bound UDP |
| A08 | DNS A/AAAA | Resolves through protected path |
| A09 | VPN disconnect during established TCP | Flow fails; no raw continuation |
| A10 | VPN disconnect during rapid new connects | No successful raw egress |
| A11 | VPN disconnect during UDP traffic | No raw UDP egress |
| A12 | VPN server switch/network replacement | Old generation closed; safe reconnect |
| A13 | Android UI killed | Foreground service behavior matches design |
| A14 | Android service killed | Windows fails closed |
| A15 | Screen off 30+ min | Expected stable sharing or documented OEM limitation |
| W01 | Windows pairing | Only approved client connects |
| W02 | Wrong pairing/auth secret | Rejected and rate limited |
| W03 | Proxy Mode browser | Works through phone VPN |
| W04 | Wintun Full Tunnel | Non-proxy-aware application works |
| W05 | Full Tunnel UDP/QUIC | Works or explicit supported limitation recorded |
| W06 | DNS full tunnel | No physical-adapter DNS leak |
| W07 | IPv6 | Tunneled or blocked; no bypass |
| W08 | Android VPN drops | Windows kill switch prevents fallback |
| W09 | TUN process crashes | Raw Internet remains blocked |
| W10 | Windows UI crashes | Protection state remains safe |
| W11 | Windows sleep/resume | Recovers without raw leak |
| W12 | Hotspot subnet changes | rediscovery/recovery works |
| W13 | Intentional disconnect | Original network restored |
| W14 | Windows reboot after interrupted session | recovery journal restores safe state |
| W15 | Uninstall | no broken routes/DNS/firewall residue |
| S01 | Fuzz SOCKS/control parser | no panic/memory safety issue |
| S02 | Connection flood | limits/backpressure work |
| S03 | UDP mapping flood | bounded resource usage |
| S04 | Invalid protocol states/replay | rejected safely |
| P01 | TCP throughput benchmark | meets recorded target/baseline |
| P02 | latency benchmark | meets recorded target/baseline |
| P03 | 8-hour soak | no unbounded growth or fatal error |
| P04 | 10k connection churn | no leak/crash; bounded resources |

---

## FILE: docs/26_CODING_STANDARDS.md

# 26 — Coding Standards

## Rust

- stable Rust unless a documented platform blocker requires otherwise,
- `cargo fmt`,
- strict clippy,
- avoid `unwrap()`/`expect()` in production network paths except proven invariants with explanatory comments,
- typed errors at subsystem boundaries,
- cancellation-aware async operations,
- bounded channels,
- no unsafe unless required by platform FFI and wrapped in a tiny audited module,
- document safety invariants around every unsafe block.

## Kotlin

- narrow Android bridge only,
- coroutines/lifecycle-aware code where appropriate,
- no global mutable singleton state without clear synchronization,
- explicit permission/error handling,
- no reflection into hidden Android APIs.

## TypeScript/Svelte

- strict TypeScript,
- no networking data plane in UI,
- state derived from backend snapshots/events,
- accessible components,
- no secrets in localStorage.

## Error model

Every user-visible failure maps to a stable internal error code. Preserve causal chains in diagnostics while presenting concise user actions.

## Tests with code

New parser/state-machine/route logic should arrive with tests. Bug fixes should add a regression test whenever feasible.

---

## FILE: docs/27_OPEN_QUESTIONS_EXPERIMENTS.md

# 27 — Open Questions and Automated Experiments

Resolve these empirically early instead of guessing.

## E01 — VPN Network socket binding from Rust

Prove that a Rust TCP socket file descriptor can be bound to the Android VPN `Network` before connect using the selected Kotlin/NDK bridge.

Evidence:

- network capabilities,
- bind return code,
- protected public IP,
- failure after VPN network loss.

## E02 — Local listener + VPN-bound egress simultaneously

Prove the process can accept a TCP client on Local-Only Hotspot while separate upstream sockets are explicitly VPN-bound without process-wide binding conflicts.

## E03 — VPN provider LAN blocking

Test representative VPN apps with their default LAN/local-network settings. Determine whether hotspot-local inbound traffic reaches VPNBridge and what settings are required.

## E04 — Wi-Fi STA + Local-Only Hotspot concurrency

Test target phones while Android upstream is Wi-Fi. Record whether the upstream remains active and VPN stable.

## E05 — DNS server reachability

Verify DNS servers reported by VPN `LinkProperties` can be reached from VPN-bound UDP/TCP sockets and identify VPNs with unusual resolver behavior.

## E06 — `tun2proxy` embedding

Evaluate current crate/API versus sidecar integration on Windows. Benchmark startup, throughput, UDP, DNS, IPv6, and recovery.

## E07 — WFP kill switch

Build an isolated test that permits only phone-local gateway traffic on physical hotspot while blocking raw external egress, then safely rolls back.

## E08 — Android screen-off/OEM behavior

Run 30-minute and multi-hour tests with screen off and UI closed.

Each experiment ends with a decision entry, reproducible test, and captured evidence.

---

## FILE: docs/28_PRIMARY_REFERENCES.md

# 28 — Primary Technical References

These references were current when this pack was prepared. Antigravity must verify them again at implementation time because Android/Tauri/Antigravity APIs and policies can change.

## Google Antigravity / Gemini managed agents

- Antigravity Agent: https://ai.google.dev/gemini-api/docs/antigravity-agent
- Building managed agents / AGENTS.md / SKILL.md: https://ai.google.dev/gemini-api/docs/custom-agents
- Agents overview: https://ai.google.dev/gemini-api/docs/agents

## Android

- Local-Only Hotspot: https://developer.android.com/develop/connectivity/wifi/localonlyhotspot
- WifiManager API: https://developer.android.com/reference/android/net/wifi/WifiManager
- Android VPN guide: https://developer.android.com/develop/connectivity/vpn
- VpnService: https://developer.android.com/reference/android/net/VpnService
- Network: https://developer.android.com/reference/android/net/Network
- ConnectivityManager: https://developer.android.com/reference/android/net/ConnectivityManager
- Foreground service types: https://developer.android.com/develop/background-work/services/fgs/service-types
- Android 15 behavior changes: https://developer.android.com/about/versions/15/behavior-changes-15
- Android NDK networking / `android_setsocknetwork`: https://developer.android.com/ndk/reference/group/networking

## Tauri

- Tauri prerequisites/mobile setup: https://v2.tauri.app/start/prerequisites/
- Tauri plugin development: https://v2.tauri.app/develop/plugins/

## Windows

- Wintun: https://www.wintun.net/
- Wintun source mirror/readme: https://github.com/WireGuard/wintun
- Windows Filtering Platform: https://learn.microsoft.com/windows/win32/fwp/windows-filtering-platform-start-page

## TUN-to-proxy candidate

- tun2proxy: https://github.com/tun2proxy/tun2proxy
- docs.rs: https://docs.rs/crate/tun2proxy/latest

## Rule

For implementation decisions, prefer official platform documentation and upstream project documentation over blogs or copied snippets. When docs and observed device behavior conflict, record both and build to the tested public-API behavior without hidden/privileged hacks.

---

## FILE: docs/29_ANTIGRAVITY_OPERATING_GUIDE.md

# 29 — Google Antigravity Operating Guide

## Repository-native control

The most important project control file is `.agents/AGENTS.md`. Antigravity's current managed-agent system can load `.agents/AGENTS.md` as persistent instructions and auto-discover skills under `.agents/skills/<skill-name>/SKILL.md`.

This pack already provides domain skills for:

- Android networking,
- Windows tunnel/routing,
- quality gates,
- security review.

## Recommended first instruction

Paste the contents/intention of `prompts/BOOTSTRAP_PROMPT.md` into the first Antigravity project session.

After that, normal continuation can be as short as:

`Continue autonomously according to .agents/AGENTS.md and PROGRESS.md until the next unavoidable HUMAN_GATE or Definition of Done.`

## Session continuity

At every new session the agent should restore state from repository files, not from conversational memory alone.

Required state:

- `PROGRESS.md`
- `DECISIONS.md`
- `KNOWN_ISSUES.md`
- `TEST_EVIDENCE.md`

## Research behavior

Antigravity has web/search capability in managed environments. Require primary-source verification when:

- an Android/Tauri/Windows API has changed,
- permissions/target SDK rules are uncertain,
- a dependency version/API is current-sensitive,
- Play/Windows distribution policy matters,
- OEM behavior is unclear.

## Tool-use budget

Do not waste long autonomous runs on broad repetitive analysis. The agent should create focused experiments and code/tests. Use more reasoning/search budget for architecture/security/platform uncertainties and less for routine UI code.

## Hardware gates

A remote Linux Antigravity environment cannot substitute for:

- actual Android Local-Only Hotspot behavior,
- a third-party Android VPN,
- Wintun/WFP behavior on Windows,
- sleep/resume,
- real leak tests across phone + laptop.

The agent must prepare those tests and exact commands automatically, but physical evidence must come from an appropriate environment.

## Recommended autonomous cadence

1. Feasibility experiments.
2. Protected TCP proxy.
3. UDP/DNS.
4. Secure pairing.
5. Windows Proxy Mode.
6. Full Wintun mode.
7. kill switch.
8. resilience.
9. performance.
10. independent final audits.

Do not reverse this sequence merely to produce a visually complete app sooner.

---

## FILE: prompts/BOOTSTRAP_PROMPT.md

# Antigravity Bootstrap Prompt

Use this once after placing this documentation pack at the repository root.

---

You are now the autonomous principal engineer for this repository.

Read `.agents/AGENTS.md`, `START_HERE.md`, `MASTER_PROJECT_SPEC.md`, and the required documents listed in `START_HERE.md` before writing production code.

Your mission is to build VPNBridge A-to-Z, not merely scaffold it.

Immediately:

1. Inspect the entire repository and toolchain.
2. Initialize the state files from `templates/` if they do not exist.
3. Verify current official Android, Tauri, Wintun, and selected dependency APIs where this project relies on them.
4. Execute Phase 0 from `docs/16_IMPLEMENTATION_PHASES.md`.
5. Continue automatically through each unblocked phase after its gate passes.
6. After every coherent change, run relevant format/lint/build/tests and fix failures before proceeding.
7. Maintain `PROGRESS.md`, `DECISIONS.md`, `KNOWN_ISSUES.md`, and `TEST_EVIDENCE.md` continuously.
8. Never fake hardware or integration evidence. If a physical action is impossible in your environment, create an exact `HUMAN_GATE` entry and continue all other work.
9. Never declare completion until `docs/18_DEFINITION_OF_DONE.md` and `docs/25_ACCEPTANCE_TEST_MATRIX.md` are fully satisfied.

Do not ask me routine coding questions. Resolve them from the specifications, current primary documentation, experiments, security principles, and measured results. Focus first on proving the protected Android networking path and fail-closed behavior.
---

---

## FILE: prompts/BUG_HUNT_PROMPT.md

# Antigravity Adversarial Bug-Hunt Prompt

Try to break VPNBridge rather than prove it works.

Prioritize scenarios that could leak raw Internet or strand the user's Windows networking:

- VPN disconnect/reconnect races,
- VPN replacement while sockets are opening,
- split-tunnel exclusion,
- stale VPN network handle,
- TCP/UDP traffic at the exact moment protection changes,
- DNS and IPv6 bypass,
- Android service/process death,
- hotspot recreation/subnet change,
- Windows TUN/helper/UI crash,
- sleep/resume,
- route transaction partial failure,
- kill-switch partial failure,
- malformed/hostile SOCKS/control clients,
- connection/UDP floods,
- low-memory/resource exhaustion.

For every discovered issue:

1. create a reproducible failing test where feasible,
2. identify root cause,
3. implement the real fix rather than masking symptoms,
4. run affected regression tests,
5. update `KNOWN_ISSUES.md` and `TEST_EVIDENCE.md`,
6. rerun the adversarial scenario.

Continue until a fresh bug-hunt pass finds no critical/high defect.

---

## FILE: prompts/CONTINUE_PROMPT.md

# Antigravity Continue Prompt

Read `.agents/AGENTS.md`, `PROGRESS.md`, `DECISIONS.md`, `KNOWN_ISSUES.md`, and git status/history.

Run a fast repository health check, identify the highest-priority unblocked item according to `docs/17_AUTONOMOUS_EXECUTION_LOOP.md`, and continue implementation immediately.

Do not repeat completed work. Do not stop at analysis if implementation/testing is possible. Fix failures in a loop, update state/evidence, perform a gap check, then continue to the next task automatically.

---

## FILE: prompts/FINAL_AUDIT_PROMPT.md

# Antigravity Final Audit Prompt

Act as an independent senior release review board that did not implement this project.

Audit the complete repository A-to-Z against:

- `MASTER_PROJECT_SPEC.md`,
- every file under `docs/`,
- `.agents/AGENTS.md`,
- `docs/18_DEFINITION_OF_DONE.md`,
- `docs/25_ACCEPTANCE_TEST_MATRIX.md`.

Do not trust existing completion claims.

Inspect architecture, Android networking, VPN socket binding, hotspot isolation, Windows routes, Wintun, kill switch, DNS, IPv6, pairing/security, lifecycle, concurrency, error handling, resource limits, logs, permissions, dependency licenses, build/release paths, install/upgrade/uninstall, and tests.

Search explicitly for TODO/FIXME/HACK, stubs, dead paths, disabled/ignored tests, swallowed errors, unsafe code, race windows, raw-network fallback, incomplete cleanup, unbounded queues/maps, hard-coded interface assumptions, and UI states that can claim protection incorrectly.

Run every automated test/check feasible in the environment. Add targeted tests for gaps you discover. Fix all issues you can fix. Repeat the audit after fixes.

Produce/update `FINAL_AUDIT.md` with evidence and only mark release-ready when no critical/high issue remains and every applicable release gate is proven.

---

## FILE: prompts/PHASE_GATE_PROMPT.md

# Antigravity Phase Gate Prompt

Before moving to the next implementation phase, perform the current phase gate as an independent reviewer.

1. Read the phase requirements and all referenced specifications.
2. Compare them line-by-line with the actual implementation.
3. Run every feasible relevant build/test/lint/security check.
4. Search for incomplete/stub/error paths and concurrency/lifecycle gaps.
5. Reproduce the phase's critical failure scenarios.
6. Fix every issue found, then rerun the checks.
7. Update `PROGRESS.md`, `KNOWN_ISSUES.md`, `DECISIONS.md`, and `TEST_EVIDENCE.md`.
8. Mark the phase complete only when its gate has evidence.
9. Immediately start the next unblocked phase.

---

## FILE: templates/DECISIONS.md

# DECISIONS

Record material architectural/dependency/product decisions.

## ADR-000 — Template

- **Date:**
- **Status:** proposed / accepted / superseded
- **Problem:**
- **Options considered:**
- **Evidence/research:**
- **Decision:**
- **Why:**
- **Security impact:**
- **Performance impact:**
- **Rollback/replacement plan:**

---

## FILE: templates/FINAL_AUDIT.md

# FINAL AUDIT

## Release verdict

`NOT AUDITED`

## Commit audited

`<commit>`

## Requirements coverage

- Pending.

## Security findings

- Pending.

## Networking/leak findings

- Pending.

## Test status

- Pending.

## Open blockers

- Pending.

## Final sign-off criteria

Do not set verdict to `RELEASE READY` if any critical/high issue is open or required evidence is missing.

---

## FILE: templates/KNOWN_ISSUES.md

# KNOWN ISSUES

## Severity definitions

- **Critical:** raw-Internet leak, serious security compromise, destructive network-state corruption, or unrecoverable data/security problem.
- **High:** core feature/recovery failure with major user impact.
- **Medium:** important defect with workaround.
- **Low:** minor defect/polish.

## Open issues

None yet.

## Issue template

- **ID:** ISSUE-000
- **Severity:**
- **Status:** open / fixed-pending-verification / closed
- **Affected versions/commit:**
- **Reproduction:**
- **Expected:**
- **Actual:**
- **Root cause:**
- **Fix:**
- **Regression test:**
- **Evidence:**

---

## FILE: templates/PROGRESS.md

# PROGRESS

## Current phase

`PHASE: 0`

## Current release state

`NOT RELEASE READY`

## Last verified commit

`<commit>`

## Completed

- None yet.

## In progress

- Bootstrap / feasibility.

## Next actions

1. ...
2. ...

## Blockers

- None.

## HUMAN_GATE

Use this section only for unavoidable physical/external actions.

### Gate template

- **ID:** HG-000
- **Reason automation cannot perform it:**
- **Exact user action:**
- **Expected result:**
- **Evidence/log to capture:**
- **Work continuing in parallel:**

---

## FILE: templates/TEST_EVIDENCE.md

# TEST EVIDENCE

Never record a test as passed unless it actually ran in the stated environment.

## Evidence entry template

- **Test ID:**
- **Date:**
- **Git commit:**
- **Component:**
- **Environment/device:**
- **Android build/API:**
- **Windows build:**
- **VPN/provider/protocol:**
- **Upstream:** cellular / Wi-Fi / Ethernet
- **Command/procedure:**
- **Expected:**
- **Observed:**
- **Result:** PASS / FAIL / BLOCKED
- **Sanitized logs/artifacts:**
- **Notes:**
