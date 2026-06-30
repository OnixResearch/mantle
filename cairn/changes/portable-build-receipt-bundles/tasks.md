# Tasks

## Contract

- [ ] [serial] Define the portable receipt bundle format, record kinds, BLAKE3 digests, policy/store-prefix binding, deterministic ordering, and named limits. r[portable_build_receipts.receipt_bundle_format]
- [ ] [serial] Define trust-root snapshot, public key identity, revocation, expiration-window, and policy-hash binding semantics for portable verification. r[portable_build_receipts.trust_snapshot_revocation]
- [ ] [serial] Define complete evidence-chain classification for strong claims across source refs, action refs, sandbox/network policy, reference scans, output refs, attestations, and signatures. r[portable_build_receipts.evidence_chain_completeness]
- [ ] [serial] Define export/list behavior for action receipts, source refs, PathInfo identities, artifact/closure attestations, sandbox reports, semantic graph edges, and trust-basis summaries. r[portable_build_receipts.receipt_bundle_export_list]
- [ ] [serial] Define verify behavior against local outputs, store archives, source-bundle refs, signatures, action refs, object refs, store prefix, policy hash, revocation state, and expiration windows. r[portable_build_receipts.receipt_bundle_verify] r[portable_build_receipts.trust_snapshot_revocation]
- [ ] [serial] Define idempotent import behavior for verified receipt, attestation, and semantic graph material, including conflict detection and incomplete-graph diagnostics. r[portable_build_receipts.receipt_bundle_import]
- [ ] [serial] Define non-claim behavior for missing receipt material and portable evidence that proves only a narrower handoff. r[portable_build_receipts.receipt_bundle_non_claims] r[portable_build_receipts.evidence_chain_completeness]

## Implementation

- [ ] [serial] Implement pure receipt-bundle core types and validators for record canonicalization, limits, output matching, graph-edge admissibility, trust-basis summaries, trust snapshots, evidence-chain completeness, and import action planning. r[portable_build_receipts.receipt_bundle_format] r[portable_build_receipts.receipt_bundle_verify] r[portable_build_receipts.trust_snapshot_revocation] r[portable_build_receipts.evidence_chain_completeness]
- [ ] [serial] Implement export/list shells that gather existing local receipt/attestation/graph material without fabricating missing evidence. r[portable_build_receipts.receipt_bundle_export_list]
- [ ] [serial] Implement verify/import shells that check bundle evidence against local or archive-provided output facts, current or replayed trust policy, revocation state, and expiration windows before persisting sidecars or graph records. r[portable_build_receipts.receipt_bundle_verify] r[portable_build_receipts.receipt_bundle_import] r[portable_build_receipts.trust_snapshot_revocation]
- [ ] [serial] Integrate verified receipt-bundle imports with semantic graph queries so remote/offline outputs can be explained when evidence exists and fail closed when it does not. r[portable_build_receipts.receipt_bundle_import]

## Verification

- [ ] [serial] Add pure positive tests for valid canonical bundle digests, output-match verification, trust-root snapshot acceptance, complete evidence-chain classification, graph-edge import planning, and idempotent equivalent imports. r[portable_build_receipts.receipt_bundle_format] r[portable_build_receipts.trust_snapshot_revocation] r[portable_build_receipts.evidence_chain_completeness] r[portable_build_receipts.receipt_bundle_import]
- [ ] [serial] Add pure negative tests for unsupported versions, duplicate conflicting records, path traversal, missing strong-claim proof, stale action/output refs, wrong store prefix, untrusted signer, revoked signer, expired trust window, incomplete evidence chain, graph conflicts, and overbroad claims. r[portable_build_receipts.receipt_bundle_verify] r[portable_build_receipts.trust_snapshot_revocation] r[portable_build_receipts.evidence_chain_completeness] r[portable_build_receipts.receipt_bundle_non_claims]
- [ ] [serial] Add CLI tests for export/list/verify/import human output, JSON output, redaction, missing evidence reports, and conflict diagnostics. r[portable_build_receipts.receipt_bundle_export_list] r[portable_build_receipts.receipt_bundle_verify]
- [ ] [serial] Add integration tests pairing a small store archive or fake remote-build output with a receipt bundle, proving verified evidence enables `mantle why` and stale/missing evidence preserves incomplete-graph diagnostics. r[portable_build_receipts.receipt_bundle_import]
- [ ] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before implementation claims, then record focused implementation evidence before checking tasks complete. r[portable_build_receipts.receipt_bundle_format]
