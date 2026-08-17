# Current Blocker — Portable build receipt bundles

Date: 2026-07-01

## Question

Is `portable-build-receipt-bundles` still blocked after the signature, sidecar, CLI, and archive-backed graph explanation slice?

## Inspected evidence

- Prior slices implemented the receipt bundle format, live PathInfo/attestation/graph/source-state gathering, local and archive output-fact verification, source-ref verification, trust-window/revocation replay, verified semantic graph import, and non-claim classification. See the dated evidence files in this directory.
- The latest slice adds real PathInfo signature verification against configured trusted public keys, same-name-different-key rejection, unsigned-output rejection under trusted verification, trusted public key digest snapshots on export, embedded attestation sidecar payload validation/import, CLI export/list/verify/import human+JSON coverage, conflict diagnostics, and archive-backed import proof for `why` explanations. Evidence: `signature-sidecar-cli-2026-07-01.md`.
- Focused current validation shows `cargo test -p mantle --bin mantle portable_receipt::tests` passing with 31 tests, CLI parser coverage for `--trusted-public-key` passing, `semantic_graph::tests` passing, and formatting/diff checks passing.

## Decision

No current blocker remains for this Cairn change. Action/ref-scan records are still exported only when existing records or graph-derived material are available; the implementation intentionally reports missing evidence rather than fabricating placeholders, matching the non-claim requirements.

## Owner

Mantle evidence/receipt transport owner.

## Next action

Mark tasks complete, run Cairn gates and validation, sync accepted specs, archive the change, then validate the post-archive lifecycle state.
