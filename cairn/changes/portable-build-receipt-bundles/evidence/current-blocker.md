# Current Blocker — Portable build receipt bundles

Date: 2026-07-01

## Question

Can `portable-build-receipt-bundles` be honestly drained after the store archive transport landed?

## Inspected evidence

- `store-archive-transport` is implemented, synced into `cairn/specs/store-transports/spec.md`, archived at `cairn/archive/2026-06-30-store-archive-transport`, and committed as `cb486e3f`.
- `offline-source-bundle-manifest` is now archived at `cairn/archive/2026-07-01-offline-source-bundle-manifest`, so source-bundle manifest semantics have an accepted foundation rather than an active scaffold blocker.
- `src/portable_receipt.rs` already provides a first pure/CLI slice: a `mantle-build-receipt-bundle-v1` model, deterministic record ordering, BLAKE3 bundle digests, trust snapshot fields, policy-hash binding, named limits, duplicate/conflict rejection, strong-claim completeness classification, non-claim wording, idempotent evidence-state import, and `mantle receipt bundle export|list|verify|import` wiring.
- The 2026-07-01 live-gathering slice adds `mantle receipt bundle export --output <...>` collection for local PathInfo output records, artifact/runtime-closure attestation sidecars, semantic graph edges, graph-derived source/action/sandbox/trust-basis records, and matching imported source-bundle records without placeholder fabrication. Evidence: `evidence/live-evidence-gathering-2026-07-01.md`.
- The active tasks still require deeper export/list coverage for live action receipt and reference-scan sidecars beyond semantic graph summaries.
- The active tasks also require verify/import against local outputs or archive-provided output facts, source-bundle refs, signatures, revocation, expiration windows, policy hashes, conflict diagnostics, persisted graph imports, and semantic graph query integration.

## Decision

Still blocked by missing verify/import trust and semantic graph persistence/query integration. The source-bundle foundation and a first live-gathering slice exist, but receipt bundles still do not verify, import, and explain the full evidence chain required by the remaining tasks.

## Owner

Mantle evidence/receipt transport owner after live evidence gathering and semantic graph import surfaces land.

## Next action

Drain after prerequisites or split into smaller accepted increments:

1. gather PathInfo and artifact/closure attestation facts from local/store-archive outputs;
2. attach source-bundle refs from archived source-bundle manifest semantics;
3. export/list available action/sandbox/reference-scan/graph material without placeholders;
4. verify/import with current or replayed trust policy, revocation, expiration, and conflict detection;
5. prove `mantle why` uses imported graph evidence and preserves incomplete-graph diagnostics for missing evidence.
