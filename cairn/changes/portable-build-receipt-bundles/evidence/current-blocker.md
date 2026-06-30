# Current Blocker — Portable build receipt bundles

Date: 2026-06-30

## Question

Can `portable-build-receipt-bundles` be honestly drained after the store archive transport landed?

## Inspected evidence

- `store-archive-transport` is now implemented, synced into `cairn/specs/store-transports/spec.md`, archived at `cairn/archive/2026-06-30-store-archive-transport`, and committed as `cb486e3f`.
- `src/portable_receipt.rs` now provides a first pure/CLI slice: a `mantle-build-receipt-bundle-v1` model, deterministic record ordering, BLAKE3 bundle digests, trust snapshot fields, policy-hash binding, named limits, duplicate/conflict rejection, strong-claim completeness classification, non-claim wording, idempotent evidence-state import, and `mantle receipt bundle export|list|verify|import` wiring.
- The active tasks still require export/list to gather existing live action receipts, source refs, PathInfo identities, artifact/closure attestations, sandbox reports, semantic graph edges, and trust-basis summaries without fabricating missing evidence.
- The active tasks also require verify/import against local outputs or archive-provided output facts, source-bundle refs, signatures, revocation, expiration windows, policy hashes, and semantic graph query integration.
- `offline-source-bundle-manifest` remains active and blocked before source-bundle refs/offline source-state readiness can be used as receipt evidence.

## Decision

Still blocked by missing live evidence-gathering, source-bundle readiness, and semantic graph persistence/query integration. The pure receipt schema and explicit-record CLI are useful, but they do not yet satisfy the full export/list/verify/import and `mantle why` integration tasks.

## Owner

Mantle evidence/receipt transport owner after source bundle foundations and semantic graph import surfaces land.

## Next action

Drain after prerequisites or split into smaller accepted increments:

1. gather PathInfo and artifact/closure attestation facts from local/store-archive outputs;
2. attach source-bundle refs once source bundle planning/import is complete;
3. export/list available action/sandbox/reference-scan/graph material without placeholders;
4. verify/import with current or replayed trust policy, revocation, expiration, and conflict detection;
5. prove `mantle why` uses imported graph evidence and preserves incomplete-graph diagnostics for missing evidence.
