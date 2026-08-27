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
