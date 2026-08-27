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
