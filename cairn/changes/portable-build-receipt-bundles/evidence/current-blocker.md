# Current Blocker — Portable build receipt bundles

Date: 2026-07-01

## Question

Can `portable-build-receipt-bundles` be honestly drained after the store archive transport landed?

## Inspected evidence

- `store-archive-transport` is implemented, synced into `cairn/specs/store-transports/spec.md`, archived at `cairn/archive/2026-06-30-store-archive-transport`, and committed as `cb486e3f`.
- `offline-source-bundle-manifest` is now archived at `cairn/archive/2026-07-01-offline-source-bundle-manifest`, so source-bundle manifest semantics have an accepted foundation rather than an active scaffold blocker.
- `src/portable_receipt.rs` already provides a first pure/CLI slice: a `mantle-build-receipt-bundle-v1` model, deterministic record ordering, BLAKE3 bundle digests, trust snapshot fields, policy-hash binding, named limits, duplicate/conflict rejection, strong-claim completeness classification, non-claim wording, idempotent evidence-state import, and `mantle receipt bundle export|list|verify|import` wiring.
- The 2026-07-01 live-gathering slice adds `mantle receipt bundle export --output <...>` collection for local PathInfo output records, artifact/runtime-closure attestation sidecars, semantic graph edges, graph-derived source/action/sandbox/trust-basis records, and matching imported source-bundle records without placeholder fabrication. Evidence: `evidence/live-evidence-gathering-2026-07-01.md`.
- The 2026-07-01 output-verification slice adds `mantle receipt bundle verify --output <...> --policy-hash <expected>` checks against local PathInfo facts, including store-prefix binding, policy-hash binding, output-ref presence, and stale local digest rejection. Evidence: `evidence/output-verification-2026-07-01.md`.
- The 2026-07-01 verified-import slice adds explicit trust replay context, expiration-window and revoked-key rejection, verified import requiring local output facts/policy hash, semantic graph node/edge persistence, graph conflict rejection, and a `mantle why` core proof over imported graph evidence. Evidence: `evidence/verified-import-2026-07-01.md`.
- The active tasks still require deeper export/list coverage for live action receipt and reference-scan sidecars beyond semantic graph summaries.
- The active tasks also require archive-provided output fact verification, source-bundle ref verification beyond imported state matching, real signature cryptographic verification against trust snapshots, attestation sidecar persistence from embedded/copied payloads, CLI-level import/list/verify conflict diagnostics, and a store-archive/remote-output integration test.

## Decision

Still blocked by missing archive-output verification, real signature verification, sidecar payload persistence, and broader CLI/integration coverage. The source-bundle foundation, live-gathering slice, local output verification slice, and verified graph-import/trust replay slice exist, but receipt bundles still do not import and explain the full evidence chain required by the remaining tasks.

## Owner

Mantle evidence/receipt transport owner after live evidence gathering and semantic graph import surfaces land.

## Next action

Drain after prerequisites or split into smaller accepted increments:

1. gather PathInfo and artifact/closure attestation facts from local/store-archive outputs;
2. attach and verify source-bundle refs from archived source-bundle manifest semantics;
3. export/list available action/sandbox/reference-scan/graph material without placeholders;
4. verify/import with archive-provided output facts, real signatures, attestation payload persistence, and CLI conflict diagnostics;
5. prove a store-archive or remote-output handoff keeps `mantle why` explanations when evidence exists and preserves incomplete-graph diagnostics for missing evidence.
