## Implementation

- [ ] [serial] I1 Inspect `Cargo.lock`, `vendor-deps/astral-tokio-tar-*`, Cargo checksum metadata, and native source-material records for the known `astral-tokio-tar@0.6.3` mismatch. r[rust_package_planning.vendor_material_checksum_repair]
- [ ] [serial] I2 Repair the vendored package material or declared checksum metadata without weakening checksum validation. r[rust_package_planning.vendor_material_checksum_repair]
- [ ] [serial] I3 Add or update diagnostics/tests so source-material drift reports the package identity, expected/actual digest class, and native planning location deterministically. r[rust_package_planning.vendor_source_material_drift_diagnostics]
- [ ] [serial] I4 Refresh operator proof docs with the latest fixed-point status, receipt digests, and explicit non-claims. r[rust_package_planning.cargo_free_fixed_point_frontier_rerun]

## Verification

- [ ] [serial] V1 Run focused native registry source planning and Cargo-free blocker classifier tests to prove the known checksum blocker no longer reproduces or is replaced by a new exact blocker. r[rust_package_planning.vendor_material_checksum_repair]
- [ ] [serial] V2 Run `mantle self-build --cargo-free --fixed-point` with an out-of-tree proof bundle and record the resulting success or next-blocker evidence. r[rust_package_planning.cargo_free_fixed_point_frontier_rerun]
- [ ] [serial] V3 Run proof guide guard checks, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[rust_package_planning.cargo_free_fixed_point_frontier_rerun]
