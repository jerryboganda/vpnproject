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
