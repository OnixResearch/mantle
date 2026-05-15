## Why

The real release determinism rail now produces validated proof bundles, verify receipts, and portable summary artifacts, but `bootstrap parity-report` still treats `crunch.self-build` as a generic partial row with “full proof bundle required.” Operators need bootstrap parity to consume the same machine-checkable proof evidence without turning a bounded two-clean-store artifact match into a full Guix or StageX bootstrap claim.

## What Changes

- Add parity-report support for a checked real self-build proof evidence bundle tied to `crunch.self-build`.
- Validate the deterministic proof receipt, sandbox evidence, release verify receipt, and summary linkage before the row can report evidence-backed self-rebuild proof.
- Preserve provider-kind gating: legacy fetched proof evidence remains bounded and cannot satisfy Guix or StageX full-source/lineage requirements.
- Add negative tests for missing proof evidence, digest drift, unsupported workflow/version, provider-kind mismatch, missing sandbox evidence, and overclaiming complete parity.

## Capabilities

### Modified Capabilities
- `bootstrap.stagex.selfbuild.proof`: parity consumes bounded real self-build proof evidence for `crunch.self-build` while keeping Guix/StageX completion fail-closed.

## Impact

- **Files**: `src/bootstrap_parity.rs`, `bootstrap/evidence/`, tests for parity/report evidence, README/operator docs as needed.
- **APIs**: `bootstrap parity-report` JSON row details may gain explicit real-proof evidence fields/blocker notes; no existing field is removed.
- **Dependencies**: no new runtime dependency expected beyond existing JSON/BLAKE3 validation patterns.
- **Testing**: parity unit and CLI regressions for positive evidence and each fail-closed negative case; `openspec validate consume-real-self-build-proof-evidence --strict`.
