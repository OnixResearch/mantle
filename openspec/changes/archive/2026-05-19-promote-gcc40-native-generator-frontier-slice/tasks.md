## 1. Bounded native generator frontier slice

- [x] 1.1 Confirm the current `gcc.4.0` native frontier markers and choose the exact `genattrtab` bounded output contract to promote. ✅ 2026-05-19T23:03:21Z
  - Evidence: `bootstrap/evidence/gcc-4.0-native-boundary.json` now records `genattrtab-bounded-output` as a promoted native slice while leaving `generator-empty-boundaries` on the remaining `genoutput` boundary marker.
- [x] 1.2 Add or update derivation/evidence artifacts so `genattrtab` has a checked bounded native-generator receipt rather than only an empty-boundary shim. ✅ 2026-05-19T23:03:21Z
  - Evidence: `bootstrap/gcc-4.0.ncl` emits `gcc40_genattrtab_bounded_output_slice`/`HAVE_ATTR_enabled`; `bootstrap/evidence/gcc-4.0-native-generator-slice.json` records the bounded output/transcript BLAKE3 digests and forbidden old empty-attrtab markers.
- [x] 1.3 Update bootstrap parity validation to require the new receipt, fail closed on drift, and keep `gcc.4.0` evidence-backed `partial`. ✅ 2026-05-19T23:03:21Z
  - Evidence: `src/bootstrap_parity.rs` requires `bootstrap/evidence/gcc-4.0-native-generator-slice.json` through the existing GCC 4.0 evidence check; the real derivation test asserts `StageStatus::Partial` and `blocks_parity()`.
- [x] 1.4 Add positive and negative tests for receipt acceptance, missing receipt, marker/digest drift, unsupported schema, and accidental full-parity overclaim. ✅ 2026-05-19T23:03:21Z
  - Evidence: targeted `gcc40` bootstrap parity unit tests cover matching receipt acceptance, missing generator receipt, frontier marker drift, transcript digest drift, unsupported generator schema, and the real derivation staying partial/blocking.
- [x] 1.5 Run focused verification: targeted bootstrap parity tests, `./scripts/check-bootstrap-parity-snapshot.sh`, `git diff --check`, and `openspec validate promote-gcc40-native-generator-frontier-slice --strict`. ✅ 2026-05-19T23:03:21Z
  - Evidence: `cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture` passed 28/28; `./scripts/check-bootstrap-parity-snapshot.sh` passed with report BLAKE3 `a5b863adbab6c7df18dfd71f8db082f081d277fed87ee45a258ba18c0c25b800`; `git diff --check` passed; `openspec validate promote-gcc40-native-generator-frontier-slice --strict` passed; `openspec validate --all --strict` passed 50/50.

## 2. Closeout

- [x] 2.1 Update task completion notes with exact evidence/commands. ✅ 2026-05-19T23:03:21Z
- [x] 2.2 Archive the OpenSpec only after implementation and verification are complete. ✅ 2026-05-19T23:03:21Z
  - Evidence: archived to `openspec/changes/archive/2026-05-19-promote-gcc40-native-generator-frontier-slice/` after focused verification passed.
