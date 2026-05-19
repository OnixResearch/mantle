## 1. Bounded native genoutput slice

- [x] 1.1 Confirm the current `gcc.4.0` native frontier markers and choose the exact `genoutput` bounded output contract to promote. ✅ 2026-05-19T23:27:37Z
  - Evidence: `bootstrap/evidence/gcc-4.0-native-boundary.json` frontier marker `Crunch GCC 4.0 empty-output source boundary: native genoutput promotion pending.` was promoted to bounded `genoutput`; `bootstrap/gcc-4.0.ncl` now emits `GCC40_GENOUTPUT_BOUNDED` and `gcc40_genoutput_bounded_output_slice` with the explicit non-claim marker.
- [x] 1.2 Add or update derivation/evidence artifacts so `genoutput` has checked bounded native-generator receipt evidence rather than only an empty-output boundary shim. ✅ 2026-05-19T23:27:37Z
  - Evidence: `bootstrap/gcc-4.0.ncl` checks the bounded output symbol, marker, and constant; `bootstrap/evidence/gcc-4.0-native-generator-slice.json` schema `mantle-gcc40-native-generator-slice-v2` records bounded `genattrtab` and `genoutput` outputs with BLAKE3 output/transcript digests; `gcc-4.0-native-boundary.json` records `genoutput-bounded-output` under promoted native slices.
- [x] 1.3 Update bootstrap parity validation to require the new/updated receipt, fail closed on drift, and keep `gcc.4.0` evidence-backed `partial`. ✅ 2026-05-19T23:27:37Z
  - Evidence: `src/bootstrap_parity.rs` requires schema v2, `selected_generators` containing `genattrtab` and `genoutput`, per-generator bounded output contracts, source markers, output/transcript digest checks, forbidden stale markers, and unchanged partial/blocking parity effect.
- [x] 1.4 Add positive and negative tests for receipt acceptance, missing receipt, marker/digest drift, unsupported schema or selected generator, and accidental full-parity overclaim. ✅ 2026-05-19T23:27:37Z
  - Evidence: targeted `bootstrap_parity::tests::gcc40` suite passed 29/29, including missing generator receipt, frontier marker drift, generator digest drift, unsupported schema, unsupported selected generator, and `gcc40_real_derivation_reports_inventory_backed_partial`.
- [x] 1.5 Run focused verification: targeted bootstrap parity tests, `./scripts/check-bootstrap-parity-snapshot.sh`, `git diff --check`, and strict OpenSpec validation. ✅ 2026-05-19T23:27:37Z
  - Evidence: `cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture` passed 29/29; `./scripts/check-bootstrap-parity-snapshot.sh` passed with report BLAKE3 `2d7a65542fecfe1225fa5a0e1715fa2817b96f793d4e1c1339286c96daf1a8e6` and blocker counts live-bootstrap=5, guix=6, stagex=2; `git diff --check` passed; `openspec validate promote-gcc40-native-genoutput-slice --strict` passed; `openspec validate --all --strict` passed 50/50.

## 2. Closeout

- [x] 2.1 Update task completion notes with exact evidence/commands. ✅ 2026-05-19T23:27:37Z
  - Evidence: this file records implementation and verification evidence for each task before archive.
- [x] 2.2 Archive the OpenSpec only after implementation and verification are complete. ✅ 2026-05-19T23:27:37Z
  - Evidence: `openspec archive promote-gcc40-native-genoutput-slice --yes` synced `openspec/specs/bootstrap/spec.md`, archived the change as `openspec/changes/archive/2026-05-19-promote-gcc40-native-genoutput-slice/`, and `openspec list` reported no active changes.
