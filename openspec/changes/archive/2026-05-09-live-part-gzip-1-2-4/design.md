## Context

Crunch currently has broad live-bootstrap OpenSpec changes that group many upstream parts together. Upstream `parts.rst` labels this point as `gzip 1.2.5`, but the implemented step directory is `steps/gzip-1.2.4` and the source pin is `gzip-1.2.4.tar.gz`. Crunch represents the implemented step in `bootstrap/gzip-tcc.ncl`.

## Goals / Non-Goals

**Goals:**
- Make the implemented `gzip 1.2.4` step independently schedulable, debuggable, and archivable.
- Keep evidence local to `bootstrap/gzip-tcc.ncl` and its declared output contract.
- Preserve dependency ordering through predecessor part outputs.

**Non-Goals:**
- Completing downstream parts in this change.
- Reworking unrelated bootstrap files.
- Changing the overall bootstrap trust claim without evidence from all required parts.

## Decisions

### 1. One OpenSpec per upstream part/Crunch derivation

**Choice:** Track `bootstrap/gzip-tcc.ncl` in `live-part-gzip-1-2-4`, not the stale generated `live-part-gzip-1-2-5` name.

**Rationale:** This matches the implemented `steps/gzip-1.2.4` source directory while documenting the `parts.rst` heading mismatch, and prevents a single failure from making the whole live-bootstrap queue opaque.

**Alternative:** Keep grouped changes only. Rejected because grouped tasks already hid blocker ownership for `tinycc-mes.ncl`.

**Implementation:** Update only `bootstrap/gzip-tcc.ncl` and part-local evidence unless a predecessor contract is wrong; if a predecessor changes, create or update that predecessor part change first.

## Risks / Trade-offs

**Many active changes** → Mitigate with the generated part index and strict dependency ordering.

**Duplicate evidence across umbrella changes** → Treat umbrella changes as roll-up status only; source/build/smoke evidence lives in the part change.

## Validation Plan

1. Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/gzip-tcc.ncl`.
2. Run `/tmp/crunch-build/debug/crunch build bootstrap/gzip-tcc.ncl` with the documented bootstrap build environment.
3. Smoke-test the output contract described by `bootstrap/gzip-tcc.ncl`.
4. Record host-leakage scan results for paths, env usage, and undeclared tools.
