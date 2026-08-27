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
