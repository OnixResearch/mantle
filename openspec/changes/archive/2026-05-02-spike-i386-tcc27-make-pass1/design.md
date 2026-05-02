# Design: i386 TinyCC 0.9.27 to Make 3.82 pass1 spike

## Decision

Keep the proof as `bootstrap/spike-i386-tcc27-make-pass1.ncl`, a sibling diagnostic derivation. It imports the already-proven `spike-i386-tinycc26-cross-smoke.ncl` and does not alter production `bootstrap/make-tcc.ncl`.

## Rationale

This isolates i386 evidence from the amd64 Mes runtime route and allows fast iteration on exact handoff blockers. If the proof fails, the change remains useful by narrowing whether the missing piece is runtime layout, TinyCC 0.9.27 self-build, or GNU Make compile/link/runtime.

## Validation

The derivation must log each step, preserve rc/stdout/stderr snippets, and require a positive `make --version` plus trivial Makefile smoke before claiming pass1 success.
