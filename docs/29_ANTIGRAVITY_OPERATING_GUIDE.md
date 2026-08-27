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
