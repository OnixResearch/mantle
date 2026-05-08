## Context

Crunch currently has broad live-bootstrap OpenSpec changes that group many upstream parts together. Upstream `steps/manifest` and `steps/mpc-1.2.1/` identify this implemented part as `mpc-1.2.1`, while upstream `parts.rst` currently labels the narrative heading `mpc 3.2.1`; Crunch represents the implemented part in `bootstrap/mpc-1.2.1.ncl`.

## Goals / Non-Goals

**Goals:**
- Make `mpc 1.2.1` independently schedulable, debuggable, and archivable.
- Keep evidence local to `bootstrap/mpc-1.2.1.ncl` and its declared output contract.
- Preserve dependency ordering through predecessor part outputs.

**Non-Goals:**
- Completing downstream parts in this change.
- Reworking unrelated bootstrap files.
- Changing the overall bootstrap trust claim without evidence from all required parts.

## Decisions

### 1. One OpenSpec per upstream part/Crunch derivation

**Choice:** Track `bootstrap/mpc-1.2.1.ncl` in `live-part-mpc-1-2-1`.

**Rationale:** This matches `parts.rst` granularity and prevents a single failure from making the whole live-bootstrap queue opaque.

**Alternative:** Keep grouped changes only. Rejected because grouped tasks already hid blocker ownership for `tinycc-mes.ncl`.

**Implementation:** Update only `bootstrap/mpc-1.2.1.ncl` and part-local evidence unless a predecessor contract is wrong; if a predecessor changes, create or update that predecessor part change first.

## Risks / Trade-offs

**Many active changes** → Mitigate with the generated part index and strict dependency ordering.

**Duplicate evidence across umbrella changes** → Treat umbrella changes as roll-up status only; source/build/smoke evidence lives in the part change.

## Validation Plan

1. Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/mpc-1.2.1.ncl`.
2. Run `/tmp/crunch-build/debug/crunch build bootstrap/mpc-1.2.1.ncl` with the documented bootstrap build environment when declared prerequisite providers exist; otherwise record the fail-closed prerequisite blocker and do not substitute host or legacy compilers.
3. Smoke-test the output contract described by `bootstrap/mpc-1.2.1.ncl` when an output exists.
4. Record host-leakage scan results for paths, env usage, and undeclared tools when a transcript exists; otherwise preserve the no-host-fallback gate.
