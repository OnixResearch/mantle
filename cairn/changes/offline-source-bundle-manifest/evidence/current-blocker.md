# Current Blocker — Offline source bundle manifest

Date: 2026-06-30

## Question

Can `offline-source-bundle-manifest` be honestly drained after the pure/source-state, build-root planning, and local export materialization slices?

## Inspected evidence

- `cairn/changes/offline-source-bundle-manifest/tasks.md` still requires:
  - source bundle plan/export over actual selected build-root source/input closures beyond local `file://` payloads;
  - VCS snapshot materialization for non-local or revision-checked checkouts;
  - language-neutral package-manager mirror adapters, including Cargo and at least one non-Cargo fixture;
  - bootstrap source archive, provider manifest, toolchain/source-root, and proof-input fixtures;
  - offline build preflight integration before sandbox execution;
  - CLI tests proving no-mutate plan behavior, no-network import/list/verify behavior, and fail-closed stale/missing source preflight.
- `src/source_bundle.rs` now provides useful first slices: source-bundle manifest model, local declared source specs, BLAKE3 refs, safe path/symlink rejection, source-state import/list/verify, pin files, CLI wiring, build-root evaluation that derives metadata-only records for fixed fetchers, git fetchers, and declared store-path inputs without running builds or network fetches, and export-time materialization for local `file://` fixed fetcher payloads plus local checkout VCS snapshots with `.git/` excluded.
- That slice still does not acquire non-local payloads, verify VCS revisions while snapshotting, gather package-manager/provider/toolchain/proof fixtures, or enforce source-state preflight before builds.

## Decision

Still blocked as a complete source/input transport. The current implementation proves explicit local-record bundles, metadata-only build-root source discovery, and local file/check-out payload export, but not non-local payload acquisition, adapter fixtures, or offline preflight behavior. Marking the change drained now would overclaim offline build readiness.

## Owner

Mantle source/input transport owner / next implementation pass.

## Next action

Implement in dependency order:

1. add package-manager mirror adapter records, including Cargo plus one non-Cargo fixture;
2. add bootstrap/provider/toolchain/proof input fixtures;
3. define the explicit non-local payload acquisition/import handoff for remote fetcher URLs and revision-checked VCS snapshots;
4. integrate imported source state into local/remote offline preflight before sandbox or remote dispatch;
5. add CLI and fixture tests for no-network import/list/verify and missing/stale preflight failures.
