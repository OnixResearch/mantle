## Why

`repair-make-tcc-amd64-varargs` completed source-level repair and source-pin/mirror preflight work, but the required `crunch build bootstrap/make-tcc.ncl` proof exceeded the local validation budget. The parent cannot honestly mark the build, smoke, and host-leakage checks as passing without a completed output.

## What Changes

- Run the repaired `bootstrap/make-tcc.ncl` build with a longer-lived runtime-validation budget.
- Preserve the full build transcript and output path.
- Smoke-test GNU Make 3.82 with version, positive Makefile, and missing-target negative checks.
- Scan the derivation, transcript, and output for host leakage.

## Capabilities

### Modified Capabilities
- `bootstrap`: the make 3.82 amd64 repair remains incomplete until this runtime-validation successor records fresh build, smoke, and leakage evidence.

## Non-Goals

- Adding new source-level repairs unless the long-running proof exposes a concrete failure.
- Replacing GNU Make 3.82 with a shim.
- Claiming downstream musl/autotools completion.

## Impact

- **Files**: evidence under this change; possibly `bootstrap/make-tcc.ncl` only if the long build exposes a concrete source-level bug.
- **APIs**: none.
- **Dependencies**: uses the repaired parent source state and local bubblewrap-enabled Crunch builder.
- **Testing**: long-running `crunch build`, version smoke, positive/negative Makefile smoke, host-leakage scan, and OpenSpec validation.

## Parent

Deferred from `openspec/changes/repair-make-tcc-amd64-varargs` tasks V2, V3, and V4.
