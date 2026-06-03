# Tasks

## Contract

- [x] [serial] Define the admitted artifact export request and result model with artifact ref, expected digest, spec-admission attestation, provenance, destination mode, and export receipt identity. r[build_tool_boundary.admitted_artifact_fetch_export] Evidence: `src/frontend_artifact_export.rs::{FrontendArtifactExportExpectation,FrontendArtifactExportContent,FrontendArtifactExportRequest,FrontendArtifactExportReceipt}`; `evidence/validator-slice-2026-06-02.md` records focused tests.
- [x] [serial] Define fail-closed validation for missing proof, unsupported ref scheme, artifact/proof mismatch, content digest mismatch, unavailable content, and hidden Nix fallback markers. r[build_tool_boundary.admitted_artifact_fetch_export] Evidence: `src/frontend_artifact_export.rs::export_frontend_artifact`; `evidence/validator-slice-2026-06-02.md` records positive and negative validation tests.

## Implementation

- [ ] [serial] Add a frontend-neutral CLI/API seam that exports admitted artifacts by `mantle://...` ref without requiring `/nix/store` paths or frontend-specific artifact semantics. r[build_tool_boundary.admitted_artifact_fetch_export]
- [ ] [serial] Preserve spec-admission attestation and build provenance in export receipts or sidecars. r[build_tool_boundary.admitted_artifact_fetch_export] r[verification_evidence.admitted_artifact_export_proof_before_claim]
- [ ] [serial] Keep Onix-like artifact kinds opaque under frontend specs and reject any core interpretation of Onix activation, roles, tags, providers, or deploy policy. r[build_tool_boundary.admitted_artifact_fetch_export]

## Verification

- [x] [serial] Add a positive test that exports a minimal spec-admitted frontend artifact and verifies artifact ref, digest, spec proof, provenance, and receipt hash. r[build_tool_boundary.admitted_artifact_fetch_export] r[verification_evidence.admitted_artifact_export_proof_before_claim] Evidence: `frontend_artifact_export::tests::valid_admitted_artifact_exports_with_receipt`; `evidence/validator-slice-2026-06-02.md`.
- [x] [serial] Add negative tests for missing attestation, unsupported ref scheme, wrong artifact ref, digest mismatch, missing content, and attempted Nix fallback. r[build_tool_boundary.admitted_artifact_fetch_export] Evidence: `frontend_artifact_export::tests::{missing_attestation_fails_before_export,unsupported_ref_scheme_fails_closed,wrong_artifact_ref_fails_closed,digest_mismatch_fails_closed,missing_content_fails_closed,hidden_fallback_marker_fails_closed}`; `evidence/validator-slice-2026-06-02.md`.
- [x] [serial] Record evidence before claiming any frontend artifact can be exported, transferred, or used by a deploy path. r[verification_evidence.admitted_artifact_export_proof_before_claim] Evidence: `evidence/validator-slice-2026-06-02.md` records proof for pure validator support and explicitly lists CLI/storage-backed export as a non-claim.
- [ ] [serial] Run `cairn validate --root .` and tasks gate, then archive only after completed tasks cite durable evidence. r[verification_evidence.admitted_artifact_export_proof_before_claim]
