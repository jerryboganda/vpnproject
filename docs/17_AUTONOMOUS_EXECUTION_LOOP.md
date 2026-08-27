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
