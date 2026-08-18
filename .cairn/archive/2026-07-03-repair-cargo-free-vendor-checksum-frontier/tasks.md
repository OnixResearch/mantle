## Implementation

- [x] [serial] I1 Inspect `Cargo.lock`, `vendor-deps/astral-tokio-tar-*`, Cargo checksum metadata, and native source-material records for the known `astral-tokio-tar@0.6.3` mismatch. r[rust_package_planning.vendor_material_checksum_repair]
  - Evidence: `cairn/changes/repair-cargo-free-vendor-checksum-frontier/evidence/validation.md` records the lock checksum and matching checkout-local vendor package checksum from pueue task 67.
- [x] [serial] I2 Repair the vendored package material or declared checksum metadata without weakening checksum validation. r[rust_package_planning.vendor_material_checksum_repair]
  - Evidence: checkout-local `vendor-deps/` material was refreshed from Cargo's vendored output; the fast no-cargo classifier in pueue task 62 reports zero registry/package/unit/derivation blockers.
- [x] [serial] I3 Add or update diagnostics/tests so source-material drift reports the package identity, expected/actual digest class, and native planning location deterministically. r[rust_package_planning.vendor_source_material_drift_diagnostics]
  - Evidence: `src/rust_plan.rs` adds bounded checksum evidence fields and negative checksum-drift tests; pueue task 80 reports all focused `rust_plan::tests::` passing.
- [x] [serial] I4 Refresh operator proof docs with the latest fixed-point status, receipt digests, and explicit non-claims. r[rust_package_planning.cargo_free_fixed_point_frontier_rerun]
  - Evidence: `docs/operator-proof-guide.md` records the successful proof bundle, stage BLAKE3 digest, rust-plan receipt hash, native registry digest, and bounded non-claims; pueue task 83 reports the proof-guide guard and self-test passed.

## Verification

- [x] [serial] V1 Run focused native registry source planning and Cargo-free blocker classifier tests to prove the known checksum blocker no longer reproduces or is replaced by a new exact blocker. r[rust_package_planning.vendor_material_checksum_repair]
  - Evidence: pueue task 62 reports `registry_blockers = 0`, `package_blockers = 0`, `unit_blockers = 0`, and `derivation_blockers = 0`; pueue task 80 reports `191 passed` focused rust-plan tests.
- [x] [serial] V2 Run `mantle self-build --cargo-free --fixed-point` with an out-of-tree proof bundle and record the resulting success or next-blocker evidence. r[rust_package_planning.cargo_free_fixed_point_frontier_rerun]
  - Evidence: pueue task 28 succeeded with `meta.status = success`, `fixed_point = true`, and matching stage binary BLAKE3 `ca00cd5866b0128434f95a0e0cf63ff2e5cc947eb90b60206cb9078bfe44215d`.
- [x] [serial] V3 Run proof guide guard checks, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[rust_package_planning.cargo_free_fixed_point_frontier_rerun]
  - Evidence: pueue task 83 passed `cargo fmt -p mantle --check`, proof-guide guard/self-test, and `git diff --check`; pueue task 91 passed final post-sync Cairn validate/proposal/design/tasks gates.
