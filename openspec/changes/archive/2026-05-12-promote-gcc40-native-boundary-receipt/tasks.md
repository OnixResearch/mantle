## Phase 1: Boundary Evidence

- [x] [serial] Add GCC 4.0 native-boundary receipt and parity validation.
  Evidence: `bootstrap/evidence/gcc-4.0-native-boundary.json` plus `validate_gcc40_native_boundary_receipt()`.
- [x] [serial] Add matching/missing/drift regression coverage.
  Evidence: `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` passed with 41 tests.
- [x] [serial] Update bootstrap spec and verify Rust/OpenSpec gates.
  Evidence: `cargo fmt --check`, parity-report JSON, GCC eval shell syntax check, `openspec validate promote-gcc40-native-boundary-receipt --strict`, `openspec validate bootstrap --strict`, `openspec validate --all --strict`, and `git diff --check` passed.
- [x] [serial] Archive the change after verification.
  Evidence: ready for `openspec archive promote-gcc40-native-boundary-receipt --yes` after all tasks completed.
