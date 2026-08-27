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
