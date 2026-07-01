# Current Blocker — Offline source bundle manifest

Date: 2026-07-01

## Question

Can `offline-source-bundle-manifest` be honestly drained after the pure/source-state, build-root planning, local export materialization, local offline-preflight, remote source-state handoff, local VCS revision-check, and CLI failure-coverage slices?

## Inspected evidence

- `cairn/changes/offline-source-bundle-manifest/tasks.md` still requires:
  - language-neutral package-manager mirror adapters, including Cargo and at least one non-Cargo fixture;
  - bootstrap source archive, provider manifest, toolchain/source-root, and proof-input fixtures;
  - a final decision on whether pure fixture coverage is sufficient for package/provider/toolchain/proof inputs or whether real adapter shell support is required before archive.
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
- The current implementation still does not gather real package-manager/provider/toolchain/proof fixtures beyond pure fixture records.

## Decision

Still blocked as a complete source/input transport only if real package/provider/toolchain adapter shells are required before archive. The current implementation proves explicit local-record bundles, metadata-only build-root source discovery, local file/check-out payload export with VCS revision checks, no-network imported-source handoff for non-local material, local imported-source preflight, remote upload from imported store-path source records, and CLI fail-closed diagnostics. It does not implement real adapter acquisition shells beyond pure fixture records.

## Owner

Mantle source/input transport owner / next implementation pass.

## Next action

Implement in dependency order:

1. decide whether pure fixture records satisfy the package-manager/provider/toolchain/proof source-kind scope for this change;
2. if not, broaden package-manager mirror adapter coverage beyond pure local fixtures with real Cargo/non-Cargo shell support;
3. if yes, run full gates and mark the now-proven tasks complete with evidence references.
