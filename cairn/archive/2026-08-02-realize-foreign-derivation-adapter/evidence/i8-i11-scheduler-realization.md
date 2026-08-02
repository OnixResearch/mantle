# I8-I11 scheduler and local realization

Task-ID: I8, I9, I10, I11
Covers: r[foreign_derivation_import.execution_profile], r[foreign_derivation_import.realization_adapter]
Date: 2026-08-02

## Implementation

- Build-request creation requires an explicit `ExecutionProfile`.
- Existing callers pass the native compatibility profile.
- Foreign units retain verified profiles in the ordinary `DerivationRegistry`.
- `crunch_pipeline::build_registered_derivations` uses the normal scheduler, worker, and store.
- Worker results include completed dependency outcomes without changing root failure accounting.
- `mantle foreign-import realize` accepts explicit plan, receipt, source, profile, state, store, root, and cache controls.
- The command supports `--offline`, `--substitute`, `--no-substitute`, and `--remote`.
- This version rejects `--remote` before key or store state creation.

## End-to-end evidence

The integration fixture has two native units.
The child writes `child`; the parent copies the exact child target path.
The test confirms these facts:

- both units build through the ordinary worker;
- the selected parent output contains `child`;
- the first receipt classifies both units as `built`;
- a second run classifies both units as `already-present`;
- only selected root outputs become retained roots;
- a failing child builder produces `partial-realization` evidence;
- a Guix profile does not provide `/bin/sh`, so a shell-dependent fixture fails;
- a fixed-output mismatch records its selected source-state attempt;
- remote preflight creates no receipt and no state directory;
- a stale receipt cannot drive `store pull --closure`.

## Existing scheduler evidence

Focused worker and pipeline tests cover sibling failure isolation, dependency failure propagation, cancellation, job bounds, and root outcome accounting.
Fetch-service tests cover substitution-independent fixed-output validation and candidate fallback.

## Claim boundary

Local realization records scheduler and store observations.
It does not prove evaluator parity, package correctness, provenance, reproducibility, or release eligibility.
