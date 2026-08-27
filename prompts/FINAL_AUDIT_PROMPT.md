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
