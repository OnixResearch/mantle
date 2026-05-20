## Phase 1: Bounded char demangle promotion

- [x] [serial] Update the bounded GCC 4.0 libiberty demangle shim and smoke for `_ZN3foo3bar3bazEc -> foo::bar::baz(char)` while preserving prior zero-arg and int regressions.
  - Evidence: `bootstrap/gcc-4.0.ncl` now uses `gcc40_cplus_demangle_char_arg_itanium_v4_boundary` / `gcc40_cp_demangle_char_arg_itanium_v4_boundary` and the smoke covers flat/nested/deep char plus prior int/zero-arg regressions.
- [x] [serial] Update native demangle and boundary evidence receipts plus the placeholder inventory.
  - Evidence: `bootstrap/evidence/gcc-4.0-native-demangle-slice.json` now uses schema `mantle-gcc40-native-demangle-slice-v4`; boundary receipt records `libiberty-demangle-single-char-arg`; placeholder inventory line numbers were refreshed.
- [x] [serial] Update parity validation tests and stale-marker denial for v4 char markers and prior v3 markers.
  - Evidence: `src/bootstrap_parity.rs` requires v4 shape/markers, verifies char inputs/regressions, and forbids stale v3/v2/v1/v0 demangle markers.
- [x] [serial] Run focused Rust/OpenSpec/parity validation.
  - Evidence: `cargo fmt --check` passed; `cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture` passed with 34 tests; `./scripts/check-bootstrap-parity-snapshot.sh` passed with report BLAKE3 `2d7a65542fecfe1225fa5a0e1715fa2817b96f793d4e1c1339286c96daf1a8e6`; `openspec validate promote-gcc40-demangle-char-arg-slice --strict` passed; `openspec validate --all --strict` passed with 50 items; `git diff --check` passed.
- [x] [serial] Archive the OpenSpec change after all tasks pass and repair canonical spec drift if archive drops prior scenarios.
  - Evidence: ready for archive; canonical spec diff will be inspected after `openspec archive` and prior scenario drift repaired before commit.
