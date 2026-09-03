# Architecture search — 2026-09-03

## Question

Can Mantle reuse an existing OnixResearch component for backend-neutral source observations and monotonic ingest?

## Inspected evidence

- Mantle at `625b9445e02829b92489aca730298a99b3118356`:
  - `src/source_bundle.rs`;
  - `crates/crunch-build/src/trust_boundary_nominal.rs`;
  - `crates/crunch-release-core/src/manifest.rs`;
  - `crates/crunch-release-core/src/source_review.rs`;
  - `crates/mantlepkgs-core/src/updates.rs`;
  - the archived `extend-nominal-types-to-trust-boundaries` design and evidence.
- OnixOS at `7b2b35efbc7b38980a7389ac90af863df976e2ca`:
  - `cli/src/pure_domains/backup.rs::BackupSourceObservationV1`.
- A workspace search for `SourceObservation`, `IngestPlan`, `monotonic ingest`, `locator class`, and `snapshot profile`.
- `durable-file-publication` at `951c27f59003cea9bfdb40ed4d89653d50fada1f`, including its no-replace and required-durability outcomes.

Mantlepkgs owns update-service observations. Those records contain package queries, candidate versions, response schemas, and collection timing. Their semantics do not match source-byte admission.

OnixOS owns backup file observations. Those records contain paths, file types, and byte lengths. They do not bind repository revisions, projections, snapshot profiles, or source content.

The existing Mantle fetch types remain useful at the shell boundary. `FetchUrl`, `GitRevision`, and typed fetch requests prevent primitive argument substitution. They are not a backend-neutral no-std source contract.

## Decision

Add a narrow `crunch-source-core` component. It owns source-observation admission, canonical BLAKE3 identity, v1 compatibility disposition, and monotonic ingest planning.

Reuse existing checked fetch URL and revision values in adapters. Do not copy the Mantlepkgs or OnixOS observation types. Their owners and evidence meanings differ.

Reuse the pinned `durable-file-publication` component for Linux source-state publication. Its bounded no-replace contract matches the required mechanical file semantics.

Keep source-bundle file I/O, durable state, URL parsing, Git access, release assembly, and signature verification in existing shells.

## Owner

Mantle owns source-byte observation and source-state ingest policy. Package ownership, repository ownership, source review, release signatures, and cross-project evidence remain with their current owners.

## Next action

Record baseline bytes and tests. Then add the pure core before changing source-bundle or release-shell behavior.
