# Tasks

## Spec

- [x] [serial] Add native unit-variant artifact binding requirement and design. r[rust_package_planning.native_unit_variant_artifacts]

## Implementation

- [ ] [serial] Preserve producer unit variant identity when lowering native dependency artifacts from Cargo-selected unit edges. r[rust_package_planning.native_unit_variant_artifacts]
- [ ] [serial] Replace package-only direct binding maps with unit-variant-aware binding where dependency edges have selected producer identity. r[rust_package_planning.native_unit_variant_artifacts]
- [ ] [serial] Scope `-L dependency` paths to the selected producer closure for each consumer instead of global historical search paths. r[rust_package_planning.native_unit_variant_artifacts]
- [ ] [serial] Fail closed before rustc on ambiguous package-only producer candidates. r[rust_package_planning.native_unit_variant_artifacts]
- [ ] [serial] Add focused positive and negative tests for same-package direct binding, unrelated same-crate search exclusion, and ambiguous producer blockers. r[rust_package_planning.native_unit_variant_artifacts]

## Verification

- [ ] [serial] Run focused tests, native unit graph/topology tests, clean self-probe, Cairn validation, and archive readiness checks. Evidence: `evidence/verification.md`. r[rust_package_planning.native_unit_variant_artifacts]
