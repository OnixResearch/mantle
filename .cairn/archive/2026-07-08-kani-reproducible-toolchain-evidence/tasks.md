## Phase 1: Toolchain identity model

- [x] [serial] r[mantle.kani_toolchain_evidence.identity] Add Kani verifier toolchain identity fields for Kani, Rust, CBMC, solver, wrapper, closure, and receipt digest.
- [x] [serial] r[mantle.kani_toolchain_evidence.bundle_linkage] Link Kani receipt artifacts and toolchain identity into Mantle release evidence bundles.
- [x] [serial] r[mantle.kani_toolchain_evidence.valence_boundary] Preserve Valence ownership of Kani evidence semantics and Mantle ownership of bundle identity checks.

## Phase 2: Fixtures and verification

- [x] [parallel] r[mantle.kani_toolchain_evidence.positive_fixtures] Add positive bundle fixtures with matching Kani receipt and toolchain identity.
- [x] [parallel] r[mantle.kani_toolchain_evidence.negative_fixtures] Add negative fixtures for missing Kani version, stale closure identity, unsupported solver metadata, mismatched receipt digest, and missing non-claims.
- [x] [serial] r[mantle.kani_toolchain_evidence.non_claims] Enforce non-claims that Mantle does not prove verifier soundness or Kani semantics.

## Phase 3: Documentation and validation

- [x] [serial] r[mantle.kani_toolchain_evidence.operator_docs] Document how Kani receipts are packaged or referenced as release evidence.
- [x] [serial] r[mantle.kani_toolchain_evidence.bundle_linkage] Run release-bundle fixture tests, Valence compatibility checks, and Cairn validation/gates before archive.

## Verification Coverage

| Scenario | Planned Evidence |
|---|---|
| Bundle with matching Kani toolchain identity passes | Positive fixture test |
| Receipt digest mismatch fails | Negative fixture test |
| Stale closure identity fails | Negative fixture test |
| Unsupported solver metadata fails | Negative fixture test |
| Missing non-claims fail | Negative fixture test |
| Valence/Mantle boundary is documented | Docs check |
