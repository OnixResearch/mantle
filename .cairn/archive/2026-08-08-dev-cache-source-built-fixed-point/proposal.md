# Change: Dev-cache foundation for source-built fixed-point iteration

## Why

The source-built fixed-point proof reconstructs expensive deterministic provider and store state. Development runs need bounded reuse without weakening the promoted cold proof.

The implemented foundation now provides receipt-validated provider adoption, persistent content-addressed store reuse, and an unchanged-source fast-fail decision. Fresh-directory multi-stage resume and full runtime confirmation require separate work and evidence.

## What Changes

- Add an opt-in dev-only provider-output cache keyed by verified source and policy identities.
- Add a persistent content-addressed native store and state keyed by the plan digest.
- Add a fast-fail baseline that reports an unchanged prior fixed-point result without creating a new proof.
- Keep every cache path dev-labeled and prevent promoted receipts or release-alias updates.
- Move fresh-directory multi-stage resume and cold-to-cached runtime confirmation to `add-dev-cache-cross-run-resume`.

## Non-Goals

- Satisfying a promoted fixed-point proof from cached outputs.
- Claiming fresh-directory multi-stage resume.
- Claiming a completed cold-to-cached-to-adopt runtime cycle.
- Weakening strict hermeticity, source authority, protected execution, or fallback policy.
- Proving compiler correctness or general reproducibility.

## Impact

- **Affected spec:** `source-built-fixed-point-improved-iteration`
- **Implemented code:** `src/source_built_fixed_point_dev_cache.rs`, `src/source_built_fixed_point_shell.rs`, and `src/main.rs`
- **Compatibility:** the promoted cold path remains separate and cache-disabled
- **Successor:** `add-dev-cache-cross-run-resume`
