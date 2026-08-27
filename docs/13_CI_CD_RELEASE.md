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
