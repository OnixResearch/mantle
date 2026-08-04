# Change: Dev cache and snapshot for the source-built fixed-point proof

## Why

The `prove-source-built-mantle-fixed-point` change builds Mantle from an authenticated source-root closure. Its full proof starts from empty transition, provider, and output authority every run and reconstructs the entire StageX transition, full-source native provider, host tools, Rust provider, and both Mantle stages. In practice this is a multi-hour run that frequently dies mid-StageX-transition, and none of the deterministic provider outputs are reused across attempts.

The design for that change already commits to a split: **development runs may use receipt-validated caches, but cached provider outputs cannot satisfy the promoted proof.** That dev-cache seam is currently missing. As a result every iteration pays the full cold-build cost, and a run killed after an hour loses all of that hour.

This change adds the sanctioned dev-cache and snapshot/snapshot-resume layer so iteration on the source-built fixed-point proof is fast, while the promoted proof stays cold and fresh.

## What Changes

- Add an opt-in dev-only provider-output cache keyed by a verified source-authority/plan digest. On a matching, receipt-validated hit, a dev run adopts the cached StageX and native provider instead of reconstructing them.
- Add a snapshot/resume seam so a dev run killed mid-pipeline restarts from the last completed stage instead of from an empty authority.
- Add a persistent content-addressed store snapshot (native store + state) keyed by plan digest so unchanged store paths import as cheap hits, not rebuilds.
- Add a fast-fail baseline that compares the current source profile against the last published fixed-point binary and reports unchanged results without a full run.
- Keep the promoted proof cold: the fresh-empty-authority path and its evidence rules are unchanged.
- Expose these behind an explicit dev-only flag so no production or promoted-proof path uses a cache.

## Non-Goals

- Satisfying the promoted fixed-point proof from cached provider outputs.
- Caching fetched seeds, lineages, or source bundles that must remain authenticated fresh inputs.
- Changing the v2 receipt schema, parity receipts, or evidence rules.
- Weakening strict hermeticity, no-fetch, no-Cargo, protected-execution, or no-fallback policy for promoted runs.
- Proving compiler correctness or arbitrary reproducibility from a cache hit.

## Impact

- **Affected spec:** `source-built-fixed-point-improved-iteration`
- **Planned code:** `src/source_built_fixed_point_shell.rs`, `src/source_built_fixed_point.rs`, `src/main.rs` (dev cache flag wiring)
- **Planned fixtures:** positive and negative cache hit/miss, stale-receipt, resume, and fast-fail fixtures
- **Documentation:** source-built fixed-point iteration guide
- **Compatibility:** cached provider outputs never enter promoted proof or release aliases
- **Current effect:** lifecycle planning only; this change does not yet alter proof or cache behavior
