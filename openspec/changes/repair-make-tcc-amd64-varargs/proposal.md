## Why

`live-part-make-3-82` found that `bootstrap/make-tcc.ncl` is not a local evidence-only part on amd64. The archived Mes-linked `tinycc 0.9.27` handoff hangs on any C compilation, while compile-capable `tinycc 0.9.26` can build GNU make objects but the resulting binary segfaults on a simple Makefile. The crash lands in make's varargs-heavy string construction path, matching the Mes/tcc amd64 varargs limitation already seen in the tinycc work.

## What Changes

- Repair the amd64 bootstrap boundary for the first `make 3.82` pass.
- Decide and implement the smallest honest fix that still produces GNU Make 3.82: patch GNU make varargs/string construction or adjust the predecessor compiler boundary.
- Preserve source-pin, build, smoke, and host-leakage evidence for the repaired `bootstrap/make-tcc.ncl`.

## Capabilities

### Modified Capabilities
- `bootstrap`: live-bootstrap part `make 3.82` must produce a make executable that can run a simple Makefile on amd64, not only report a version string.

## Non-Goals

- Replacing GNU Make 3.82 with a non-GNU or version-only shim.
- Claiming completion from `bin/make --version` alone.
- Completing downstream musl/autotools parts.

## Impact

- **Files**: `bootstrap/make-tcc.ncl`, evidence under this change, and parent evidence in `live-part-make-3-82`.
- **APIs**: none.
- **Dependencies**: direct predecessor compiler selection for the first make pass may change.
- **Testing**: source-pin audit, `crunch build bootstrap/make-tcc.ncl`, version smoke, simple Makefile positive smoke, missing-target negative smoke, and host-leakage scan.

## Parent

Deferred from `openspec/changes/live-part-make-3-82` task I3/V2/V3/V4.
