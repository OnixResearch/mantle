## Context

Crunch currently has broad live-bootstrap OpenSpec changes that group many upstream parts together. Upstream `parts.rst` treats `automake 1.10.3` as a named part, while Crunch represents it in `bootstrap/automake-1.10.3.ncl`.

## Goals / Non-Goals

**Goals:**
- Make `automake 1.10.3` independently schedulable, debuggable, and archivable.
- Keep evidence local to `bootstrap/automake-1.10.3.ncl` and its declared output contract.
- Preserve dependency ordering through predecessor part outputs.

**Non-Goals:**
- Completing downstream parts in this change.
- Reworking unrelated bootstrap files.
- Changing the overall bootstrap trust claim without evidence from all required parts.

## Decisions

### 1. One OpenSpec per upstream part/Crunch derivation

**Choice:** Track `bootstrap/automake-1.10.3.ncl` in `live-part-automake-1-10-3`.

**Rationale:** This matches `parts.rst` granularity and prevents a single failure from making the whole live-bootstrap queue opaque.

**Alternative:** Keep grouped changes only. Rejected because grouped tasks already hid blocker ownership for `tinycc-mes.ncl`.

**Implementation:** Update only `bootstrap/automake-1.10.3.ncl` and part-local evidence unless a predecessor contract is wrong; if a predecessor changes, create or update that predecessor part change first.

## Risks / Trade-offs

**Many active changes** → Mitigate with the generated part index and strict dependency ordering.

**Duplicate evidence across umbrella changes** → Treat umbrella changes as roll-up status only; source/build/smoke evidence lives in the part change.

## Validation Plan

1. Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/automake-1.10.3.ncl`.
2. Run `/tmp/crunch-build/debug/crunch build bootstrap/automake-1.10.3.ncl` with the documented bootstrap build environment.
3. Smoke-test the output contract described by `bootstrap/automake-1.10.3.ncl`.
4. Record host-leakage scan results for paths, env usage, and undeclared tools.
