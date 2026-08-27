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
