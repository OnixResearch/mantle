# ADR 0119: Isolate evaluator fault fixtures from the release CLI

## Status

Proposed. The native evaluator target passes, including its release guard.
The full package remains blocked by nine other integration targets.
See `evidence/package-test-fixtures-2026-09-05.md` for the scoped observations.

## Context

ADR 0074 keeps evaluator process effects outside the pure budget core.
The existing evaluator fault hooks compile only with debug assertions.
The Nix package runs release integration tests, where those hooks are absent.
Five fixture tests therefore received ordinary success instead of their requested faults.

The release guards are intentional. Test maintenance must not expose fault
injection in the installed CLI or remove negative coverage.

## Decision

Keep production evaluator code and its release guards unchanged.
Build a separate debug fixture executable from the same source and pinned
toolchain under the package check's temporary directory.
Provide its absolute path through `MANTLE_TEST_EVALUATOR_BINARY` to test code only.
Do not install this executable.

Only fault-injection integration cases select the fixture executable.
Ordinary integration cases continue to select the normal release binary.
A release-only control requires that binary to ignore panic, cancellation,
and forced reap-failure fixture variables.
Missing explicit fixture input fails release tests without a fallback or skip.

Package checks and nextest share the preparation script. Standalone release
checks require the explicit setup in `docs/package-checks.md`.

## Alternatives Considered

### Enable fault hooks in the production release

Rejected. Test convenience does not justify a new release attack surface.

### Skip fixture tests in release package checks

Rejected. This removes the intended fault, timeout, cancellation, and teardown coverage.

### Change every package test to the debug profile

Rejected. Ordinary integration cases must retain their release-binary coverage.

### Build a separate fixture executable

Selected. This retains the existing debug-only seam and the real release boundary.

## Consequences

- Package checks compile an additional debug executable and take longer.
- Fault observations describe the debug fixture, not every optimized release failure mode.
- Ordinary release tests and the new release guard remain separate observations.
- No public CLI, core policy, accepted requirement, signature policy, or runtime authority changes.
- Test success does not prove evaluator correctness or downstream admission.
