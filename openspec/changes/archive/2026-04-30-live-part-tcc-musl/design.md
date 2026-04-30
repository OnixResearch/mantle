## Context

Crunch currently has broad live-bootstrap OpenSpec changes that group many upstream parts together. Upstream `parts.rst` treats `musl 1.1.24 and musl_target` as a named part, while Crunch represents it in `bootstrap/tcc-musl.ncl`.

## Goals / Non-Goals

**Goals:**
- Make `tcc linked to musl` independently schedulable, debuggable, and archivable.
- Keep evidence local to `bootstrap/tcc-musl.ncl` and its declared output contract.
- Preserve dependency ordering through predecessor part outputs.

**Non-Goals:**
- Completing downstream parts in this change.
- Reworking unrelated bootstrap files.
- Changing the overall bootstrap trust claim without evidence from all required parts.

## Decisions

### 1. One OpenSpec per upstream part/Crunch derivation

**Choice:** Track `bootstrap/tcc-musl.ncl` in `live-part-tcc-musl`.

**Rationale:** This matches `parts.rst` granularity and prevents a single failure from making the whole live-bootstrap queue opaque.

**Alternative:** Keep grouped changes only. Rejected because grouped tasks already hid blocker ownership for `tinycc-mes.ncl`.

**Implementation:** Update only `bootstrap/tcc-musl.ncl` and part-local evidence unless a predecessor contract is wrong; if a predecessor changes, create or update that predecessor part change first.

## Risks / Trade-offs

**Many active changes** → Mitigate with the generated part index and strict dependency ordering.

**Duplicate evidence across umbrella changes** → Treat umbrella changes as roll-up status only; source/build/smoke evidence lives in the part change.

## Validation Plan

1. Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tcc-musl.ncl`.
2. Run `/tmp/crunch-build/debug/crunch build bootstrap/tcc-musl.ncl` with the documented bootstrap build environment.
3. Smoke-test the output contract described by `bootstrap/tcc-musl.ncl`.
4. Record host-leakage scan results for paths, env usage, and undeclared tools.
