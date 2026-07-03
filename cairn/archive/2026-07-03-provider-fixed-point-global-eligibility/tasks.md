## Implementation

- [x] [serial] I1 Reuse provider fixed-point verifier facts in release-derived global surface evidence, and admit provider surfaces only when verifier validity, meta digest, closure-policy digest, and stage/release digest match are present. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records `cargo test -p mantle --bin mantle global_reproducibility_release` with the positive full-release provider admission test passing.
- [x] [serial] I2 Preserve fail-closed blockers for invalid provider fixed-point proof facts and copied proof path failures. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records invalid-provider blocker coverage and provider verifier copied-bundle regression coverage.
- [x] [serial] I3 Update operator docs/release notes with the full-release eligible report and bounded non-claims. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `docs/release-notes/provider-bound-release-evidence-2026-07-02-source-policy-fixed.md` records report digest `ddcf15edea2b0b2e10ef653fcf1efe89803bfa6db49ee3c4ab7ee62ceb6e7b8f`, evidence digest `06b99a0968cbe4afb705b42318fdb3f16a01a3061b45c1cee3962175828c14fb`, and preserved non-claims.

## Verification

- [x] [serial] V1 Positive: focused tests prove the full release universe is eligible only when provider verifier facts are valid and digest-bound. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records the full-release report regeneration with `claim_class="eligible"`, no blockers, and report digest `ddcf15edea2b0b2e10ef653fcf1efe89803bfa6db49ee3c4ab7ee62ceb6e7b8f`.
- [x] [serial] V2 Negative: focused tests prove invalid provider verifier facts keep the provider surface blocked with a concrete reason. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records `release_derived_full_release_blocks_invalid_provider_fixed_point_surface` and provider verifier negative tests passing.
- [x] [serial] V3 Run focused CLI/core tests, provider verifier regression, formatting, `git diff --check`, Cairn validation/gates, and archive with post-archive validation evidence. r[verification_evidence.global_reproducibility_release_surface_evidence]
  - Evidence: `evidence/validation-2026-07-03.md` records focused tests, `cargo fmt --check -p mantle`, `git diff --check`, `cairn validate`, and proposal/design/tasks gates all exiting 0. Post-archive validation will be appended after archive execution.
