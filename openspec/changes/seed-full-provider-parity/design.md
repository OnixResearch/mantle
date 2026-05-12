## Context

The parity report introduced in `4fa66bda` intentionally fails closed, but `seed-full` is currently a single partial row on both Guix and StageX axes. The actual evidence requirements differ: Guix needs a source-root normalized provider contract, while StageX needs audited lineage proof and must not inherit the Guix row.

## Goals / Non-Goals

**Goals:**
- Preserve `seed-full` as the Guix/source-root provider row.
- Add an explicit StageX seed provider row that remains blocked without lineage proof.
- Add a deterministic `bootstrap/seed-full.ncl` contract check for normalized metadata and absence of legacy raw provider evidence.
- Verify JSON and fail-closed `--require stagex` behavior.

**Non-Goals:**
- Do not claim complete live-bootstrap, Guix, or StageX parity.
- Do not build the entire bootstrap chain.
- Do not promote additional GCC 4.0 libgcc members.

## Decisions

### 1. Split the ambiguous row instead of weakening StageX gating

**Choice:** Keep row ID `seed-full` for Guix and add `seed-full.stagex-lineage` for the StageX provider proof.
**Rationale:** Existing consumers can still find `seed-full`, while StageX claims remain independently blocked.
**Alternative:** Mark the existing shared row complete after contract validation. Rejected because it would imply StageX seed-provider evidence exists.

**Implementation:** Adjust `parity_stage_specs()` axes and add a new StageX-only `StageSpec` without a derivation until lineage evidence exists.

### 2. Validate seed-full contract from derivation text

**Choice:** Add a narrow inspector for `bootstrap/seed-full.ncl` that requires `provider.json`, `full-source-v1`, retained tools, reduction metadata, target triple, and no legacy `raw` provider block or musl.cc URL/hash evidence.
**Rationale:** This is deterministic, cheap, and matches the current report's static derivation inspection model.
**Alternative:** Require a full Crunch build before row status can move. Rejected for this drain step because current parity-report rows are static gap-map evidence and the full chain still has known blockers.

## Risks / Trade-offs

**Static evidence is not runtime proof** → The row text must say source-root provider contract evidence, not full provider build success. Build/proof rows remain separate blockers.

**JSON row ID changes for StageX** → Add tests for `seed-full` and `seed-full.stagex-lineage` so consumers see the split clearly.

## Validation Plan

- `cargo test -p crunch --bin crunch bootstrap_parity`
- `cargo test -p crunch --test bootstrap_parity_cli`
- `cargo run -q -p crunch -- --json bootstrap parity-report`
- `cargo run -q -p crunch -- bootstrap parity-report --require stagex` must fail while reporting the StageX blocker.
- `openspec validate --all --strict`
