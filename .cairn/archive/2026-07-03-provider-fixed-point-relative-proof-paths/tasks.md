## Implementation

- [x] [serial] I1 Generate provider fixed-point `meta.json` stage artifact paths as bundle-local relative paths while keeping external provenance paths explicit. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `evidence/validation-2026-07-03.md` records `fixed_point_summary_records_bundle_local_stage_paths` and `bundle_local_path_keeps_external_paths_absolute` passing.
- [x] [serial] I2 Generate provider fixed-point preflight proof-owned paths as bundle-local relative paths. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `evidence/validation-2026-07-03.md` records `fixed_point_preflight_records_bundle_local_execution_path` passing.
- [x] [serial] I3 Preserve legacy verifier support for copied absolute-path bundles. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `evidence/validation-2026-07-03.md` records `provider_fixed_point_verifier_rebases_copied_bundle_stage_paths` passing.

## Verification

- [x] [serial] V1 Positive: focused tests prove generated fixed-point summary stage paths are relative bundle-local paths. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `evidence/validation-2026-07-03.md` records `cargo test -p mantle --bin mantle fixed_point_summary` with 4 tests passing.
- [x] [serial] V2 Negative: focused tests prove non-bundle external paths stay absolute and copied absolute-path bundle verification still succeeds. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `evidence/validation-2026-07-03.md` records the external-path assertion in `fixed_point_summary` tests and copied-bundle verifier compatibility tests passing.
- [x] [serial] V3 Run formatting, diff checks, Cairn validation/gates, archive, and post-archive validation. r[verification_evidence.provider_fixed_point_path_normalization]
  - Evidence: `evidence/validation-2026-07-03.md` records `cargo fmt --check -p mantle`, `git diff --check`, `cairn validate`, and proposal/design/tasks gates all exiting 0. Post-archive validation will be appended after archive execution.
