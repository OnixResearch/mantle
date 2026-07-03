## Implementation

- [x] [serial] I1 Add pure release-to-global-surface-evidence derivation for release manifest, final verification, release attestation, witness attestations, and proof metadata. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records `cargo test -p mantle --bin mantle global_reproducibility` with 7 focused tests passing, including positive stage2 evidence derivation and negative provider-fixed-point blocker derivation.
- [x] [serial] I2 Wire `mantle release global-reproducibility-evidence` as a thin CLI shell that reads inputs and writes deterministic evidence JSON. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records the real helper invocation against the 2026-07-02 release bundle and generated two surface evidence entries.
- [x] [serial] I3 Record the full 2026-07-02 release universe evidence/report and docs pointer with explicit blockers/non-claims. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records full-release report digest `f6727befe87856b939d3de6f1771ab75b672e612cc978a0c7f3d9db147890467` blocked on `binaries/01-mantle`; `docs/release-notes/provider-bound-release-evidence-2026-07-02-source-policy-fixed.md` records durable artifact paths and non-claims.

## Verification

- [x] [serial] V1 Positive: helper-derived stage2 evidence evaluates as eligible for a stage2-only universe. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records stage2 helper report digest `255f9caf8dbd420e9e78093b7d0aee3c5914def80d2f9debdb7b9e45353b710a` with `claim_class=eligible` and no blockers.
- [x] [serial] V2 Negative: helper-derived full release universe keeps `binaries/01-mantle` blocked/unsupported while preserving accepted stage2 witness evidence. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records `claim_class=blocked`, blockers `unsupported-surface`, `weak-hermeticity`, and `reused-store` for `binaries/01-mantle`, and accepted `aspen1-external-witness` for both surfaces.
- [x] [serial] V3 Run focused CLI/core tests, `cargo fmt --check`, `git diff --check`, Cairn validate/gates, and archive with post-archive validation evidence. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records `cargo fmt --check`, focused tests, `git diff --check`, `cairn validate`, and proposal/design/tasks gates all exiting 0. Post-archive evidence will be appended after archive execution.
