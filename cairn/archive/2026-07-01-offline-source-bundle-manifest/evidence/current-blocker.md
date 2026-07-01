# Current Blocker — Offline source bundle manifest

Date: 2026-07-01

## Question

Can `offline-source-bundle-manifest` be honestly drained after the pure/source-state, build-root planning, local export materialization, local offline-preflight, remote source-state handoff, local VCS revision-check, CLI failure-coverage, and adapter readiness-class slices?

## Inspected evidence

- `cairn/changes/offline-source-bundle-manifest/tasks.md` still has unchecked boxes, but the implementation now has evidence for:
  - language-neutral package-manager mirror fixtures, including Cargo and npm/non-Cargo metadata;
  - bootstrap source archive, provider manifest, toolchain/source-root, and proof-input fixture records;
  - generic unsupported and untrusted adapter readiness classes without Cargo-specific requirements.
- `src/source_bundle.rs` now provides useful first slices: source-bundle manifest model, local declared source specs, BLAKE3 refs, safe path/symlink rejection, source-state import/list/verify, pin files, CLI wiring, build-root evaluation that derives metadata-only records for fixed fetchers, git fetchers, and declared store-path inputs without running builds or network fetches, export-time materialization for local `file://` fixed fetcher payloads plus local checkout VCS snapshots with `.git/` excluded, and `mantle-source-offline-preflight-v1` over imported source state for local build planning.
- The 2026-07-01 hardening slice tightened parsed-bundle validation for root/order/store-prefix/non-claim drift, file payload self-consistency, symlink metadata, byte counts, path case collisions, and adapter metadata completeness; it also added pure non-Cargo package-mirror coverage plus CLI plan/list no-mutate and import/verify/tamper tests.
- The later 2026-07-01 fixture slice added pure Cargo and npm package-mirror fixtures plus bootstrap archive, provider manifest, toolchain/source-root, and proof-input source record coverage.
- The imported-source handoff slice added a no-network export path that reuses already-imported matching materialized source records for remote fixed URL/VCS metadata, while preserving fail-closed behavior when source state is absent or metadata mismatches.
- The VCS revision guard slice rejects git/VCS snapshot records without an explicit `rev` fact and keeps that revision in the metadata matched against imported source state.
- The local offline-preflight slice adds `mantle source bundle preflight --build-root ...` plus `mantle build --offline-source-preflight`, and classifies ready, missing, stale, unsupported, network-required, and unpinned source readiness before normal plan/build execution.
- The remote source-state handoff slice threads source state into remote input-upload preparation, preserves local-store preference, falls back only for missing source paths, and proves an absent client store path can be uploaded from imported source-bundle records and materialized remotely.
- The local VCS revision-check slice verifies `file://` checkout snapshots against `.git/HEAD`, loose refs, and packed refs before materializing payload bytes, and rejects mismatched requested revisions.
- The CLI failure-coverage slice proves stale imported records, unpinned imported records, and remote network-required preflight all surface as JSON diagnostics before build execution.
- The non-local acquisition policy is now explicit in behavior and evidence: source-bundle export/preflight never fetch remote payloads implicitly; matching imported source state is the no-network handoff path.
- The adapter readiness-class slice adds generic `unsupported` and `untrusted` classes through adapter metadata, with pure tests proving both classes fail closed without requiring Cargo-specific fields.

## Decision

Ready for final gates and task reconciliation under the scoped interpretation that this change defines the source-bundle format, generic adapter contract, source-state handoff, and representative fixture coverage rather than real ecosystem adapter acquisition shells. Real package-manager/provider/toolchain shell adapters can be follow-up changes built on this generic contract.

## Owner

Mantle source/input transport owner / next implementation pass.

## Next action

Implement in dependency order:

1. run full focused validation plus Cairn proposal/design/tasks gates;
2. mark proven tasks complete with evidence references;
3. archive the change if gates pass after task reconciliation.
