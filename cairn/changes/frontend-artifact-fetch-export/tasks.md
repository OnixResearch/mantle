# Tasks

## Contract

- [ ] [serial] Define the admitted artifact export request and result model with artifact ref, expected digest, spec-admission attestation, provenance, destination mode, and export receipt identity. r[build_tool_boundary.admitted_artifact_fetch_export]
- [ ] [serial] Define fail-closed validation for missing proof, unsupported ref scheme, artifact/proof mismatch, content digest mismatch, unavailable content, and hidden Nix fallback markers. r[build_tool_boundary.admitted_artifact_fetch_export]

## Implementation

- [ ] [serial] Add a frontend-neutral CLI/API seam that exports admitted artifacts by `mantle://...` ref without requiring `/nix/store` paths or frontend-specific artifact semantics. r[build_tool_boundary.admitted_artifact_fetch_export]
- [ ] [serial] Preserve spec-admission attestation and build provenance in export receipts or sidecars. r[build_tool_boundary.admitted_artifact_fetch_export] r[verification_evidence.admitted_artifact_export_proof_before_claim]
- [ ] [serial] Keep Onix-like artifact kinds opaque under frontend specs and reject any core interpretation of Onix activation, roles, tags, providers, or deploy policy. r[build_tool_boundary.admitted_artifact_fetch_export]

## Verification

- [ ] [serial] Add a positive test that exports a minimal spec-admitted frontend artifact and verifies artifact ref, digest, spec proof, provenance, and receipt hash. r[build_tool_boundary.admitted_artifact_fetch_export] r[verification_evidence.admitted_artifact_export_proof_before_claim]
- [ ] [serial] Add negative tests for missing attestation, unsupported ref scheme, wrong artifact ref, digest mismatch, missing content, and attempted Nix fallback. r[build_tool_boundary.admitted_artifact_fetch_export]
- [ ] [serial] Record evidence before claiming any frontend artifact can be exported, transferred, or used by a deploy path. r[verification_evidence.admitted_artifact_export_proof_before_claim]
- [ ] [serial] Run `cairn validate --root .` and tasks gate, then archive only after completed tasks cite durable evidence. r[verification_evidence.admitted_artifact_export_proof_before_claim]
