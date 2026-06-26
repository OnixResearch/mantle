## Implementation

- [x] [serial] I1 Add a pure release-core provider fixed-point artifact binding check that returns the matched release binary artifact or a deterministic validation error. r[rust_package_planning.provider_fixed_point_release_artifact_binding]
  - Evidence: `crates/crunch-release-core/src/manifest.rs` defines `validate_provider_fixed_point_release_artifact_binding(...)`; pueue task 52 passed the four focused positive/negative release-core binding tests.
- [x] [serial] I2 Enforce the binding during release creation before writing trusted manifest evidence for bundled provider fixed-point proof artifacts. r[rust_package_planning.provider_fixed_point_release_artifact_binding]
  - Evidence: `src/release_evidence.rs` validates provider proof stage digest against input binary artifact records before bundle creation; pueue task 45 passed the matched create test and mismatched-binary rejection test.
- [x] [serial] I3 Enforce the binding during release verification for bundled and external provider fixed-point proofs, and report matched release artifact path/digest when successful. r[rust_package_planning.provider_fixed_point_release_artifact_binding]
  - Evidence: `src/release_cmd.rs` binds provider verification results to manifest binaries and adds `release_artifact_relative_path` / `release_artifact_digest_blake3`; pueue task 45 passed matching and mismatched verifier helper tests.
- [x] [serial] I4 Update release documentation and evidence wording so provider-backed release artifact claims cite the artifact binding and preserve non-claims. r[rust_package_planning.provider_fixed_point_release_artifact_binding]
  - Evidence: README and `docs/operator-workflows.md` document provider proof binary binding and non-claims.

## Verification

- [x] [serial] V1 Add positive and negative release-core tests for provider stage digest matching, missing binaries, malformed digest, and mismatched release artifacts. r[rust_package_planning.provider_fixed_point_release_artifact_binding]
  - Evidence: pueue task 52 passed `provider_fixed_point_release_artifact_binding_*` tests: 4 passed, 0 failed.
- [x] [serial] V2 Add positive and negative release create/verify tests for matched provider proof, mismatched provider proof, and required verifier failure. r[rust_package_planning.provider_fixed_point_release_artifact_binding]
  - Evidence: pueue task 45 passed `cargo test -p mantle --bin mantle provider_fixed_point -- --nocapture`: 13 passed, 0 failed.
- [x] [serial] V3 Run focused Rust tests, build or check if needed, diff checks, and Cairn validation/gates before archiving. r[rust_package_planning.provider_fixed_point_release_artifact_binding]
  - Evidence: `evidence/final-validation-2026-06-25.md` records baseline/post-change tests, fmt, build, diff check, and final Cairn validation/gates.
