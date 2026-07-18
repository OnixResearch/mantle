# Baseline evidence: hydrated fresh-clone fixed point

Date: 2026-07-17

## Repository state

The change began from synchronized `main` at `ff96b02e` with no active Cairn changes. Existing accepted evidence proves three-record fresh-clone hydration and, separately, prepared-checkout fixed-point equality. No current evidence binds a complete hydrated source closure to both proof stages with live acquisition forbidden.

## Baseline focused tests

Pueue task `107` ran before core implementation changes:

```text
source_bundle::tests: 55 passed; 0 failed
self_build::tests: 171 passed; 0 failed
self_hosting integration harness: 51 passed; 0 failed; 1 ignored
```

The ignored test is the expensive complete stage0 → stage1 → stage2 proof and was not executed by this baseline command.

## Checked seam

- `src/self_build.rs::self_build_pipeline_config` always sets `source_fetch_overrides: Vec::new()`.
- `crates/crunch-build::FetchBuildService` uses an override when one matches but otherwise performs ordinary URL/Git acquisition.
- `scripts/prove-self-hosting.sh` has no full-source-state argument or offline-fetch enforcement mode.
- `tests/self_hosting.rs::self_hosting_stage0_stage1_stage2` creates fresh stage state but does not copy authenticated source records into it.
- `fresh-clone-inputs` intentionally contains exactly three records and is not a complete evaluated bootstrap fetch closure.

## Mechanism screening

1. **Runtime source overrides:** active candidate; preserves derivation identities and can deny unmatched acquisition before I/O.
2. **Preseed source store outputs:** rejected because it can bypass fetch-policy evidence and risks admitting arbitrary non-source outputs.
3. **Rewrite derivation URLs:** rejected because source URL/path changes alter derivation fingerprints and weaken the fixed-point comparison.

The exact completion and non-claim contract is recorded in `design.md`.
