# Current Blocker — Offline source bundle manifest

Date: 2026-06-30

## Question

Can `offline-source-bundle-manifest` be honestly drained after the pure/source-state, build-root planning, local export materialization, and local offline-preflight slices?

## Inspected evidence

- `cairn/changes/offline-source-bundle-manifest/tasks.md` still requires:
  - source bundle plan/export over actual selected build-root source/input closures beyond local `file://` payloads;
  - VCS snapshot materialization for non-local or revision-checked checkouts;
  - language-neutral package-manager mirror adapters, including Cargo and at least one non-Cargo fixture;
  - bootstrap source archive, provider manifest, toolchain/source-root, and proof-input fixtures;
  - remote-builder input preparation from imported source state;
  - CLI tests proving no-mutate plan behavior, no-network import/list/verify behavior, and broader fail-closed stale/missing source preflight fixtures.
- `src/source_bundle.rs` now provides useful first slices: source-bundle manifest model, local declared source specs, BLAKE3 refs, safe path/symlink rejection, source-state import/list/verify, pin files, CLI wiring, build-root evaluation that derives metadata-only records for fixed fetchers, git fetchers, and declared store-path inputs without running builds or network fetches, export-time materialization for local `file://` fixed fetcher payloads plus local checkout VCS snapshots with `.git/` excluded, and `mantle-source-offline-preflight-v1` over imported source state for local build planning.
- The local offline-preflight slice adds `mantle source bundle preflight --build-root ...` plus `mantle build --offline-source-preflight`, and classifies ready, missing, stale, unsupported, network-required, and unpinned source readiness before normal plan/build execution.
- That slice still does not acquire non-local payloads, verify VCS revisions while snapshotting, gather package-manager/provider/toolchain/proof fixtures, or prepare remote-builder input uploads from imported source state.

## Decision

Still blocked as a complete source/input transport. The current implementation proves explicit local-record bundles, metadata-only build-root source discovery, local file/check-out payload export, and a first local imported-source preflight path, but not non-local payload acquisition, adapter fixtures, remote-builder input preparation, or complete offline source readiness. Marking the change drained now would overclaim offline build readiness.

## Owner

Mantle source/input transport owner / next implementation pass.

## Next action

Implement in dependency order:

1. add package-manager mirror adapter records, including Cargo plus one non-Cargo fixture;
2. add bootstrap/provider/toolchain/proof input fixtures;
3. define the explicit non-local payload acquisition/import handoff for remote fetcher URLs and revision-checked VCS snapshots;
4. integrate imported source state into remote-builder input preparation before remote dispatch;
5. add CLI and fixture tests for no-network import/list/verify and missing/stale/unsupported/untrusted preflight failures across representative source kinds.
