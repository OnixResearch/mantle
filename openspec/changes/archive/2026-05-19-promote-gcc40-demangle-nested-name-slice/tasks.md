## 1. Bounded native demangle nested-name slice

- [x] 1.1 Confirm the current libiberty demangle frontier markers and choose the exact nested-name input/output contract to promote. Evidence: selected `_ZN3foo3barEv -> foo::bar()` with flat `_Z3foov -> foo()` regression and malformed nested-name rejects.
- [x] 1.2 Add or update derivation/evidence artifacts so the selected nested-name demangle case has checked bounded semantic evidence. Evidence: `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-native-demangle-slice.json`, boundary receipt, and placeholder inventory updates.
- [x] 1.3 Update bootstrap parity validation to require the new/updated demangle evidence, fail closed on drift, and keep `gcc.4.0` evidence-backed `partial`. Evidence: `src/bootstrap_parity.rs` validates schema, selected shape, bounded I/O contract, source markers, transcript digest, forbidden stale markers, and partial-only parity effect.
- [x] 1.4 Add positive and negative tests for receipt acceptance, missing receipt, marker/digest drift, unsupported schema or selected shape, and accidental full-parity overclaim. Evidence: focused `bootstrap_parity::tests::gcc40` tests include demangle missing/digest/schema/shape/overclaim cases plus real derivation partial acceptance.
- [x] 1.5 Run focused verification: targeted bootstrap parity tests, `./scripts/check-bootstrap-parity-snapshot.sh`, `git diff --check`, and strict OpenSpec validation. Evidence: targeted tests passed 34/34; snapshot report BLAKE3 `2d7a65542fecfe1225fa5a0e1715fa2817b96f793d4e1c1339286c96daf1a8e6`; `git diff --check` passed; strict OpenSpec validation passed.

## 2. Closeout

- [x] 2.1 Update task completion notes with exact evidence/commands.
- [x] 2.2 Archive the OpenSpec only after implementation and verification are complete. Evidence: moved to `openspec/changes/archive/2026-05-19-promote-gcc40-demangle-nested-name-slice/` after implementation and verification.
