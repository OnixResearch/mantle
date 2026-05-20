## Change Status

- Status: implemented; validation passed.
- Scope: bounded GCC 4.0 libiberty demangle deeper nested-name semantic slice.

## 1. Bounded native demangle deeper nested-name slice

- [x] [serial] 1.1 Confirm the current libiberty demangle frontier markers and choose the exact deeper nested-name input/output contract to promote. Evidence: selected `_ZN3foo3bar3bazEv -> foo::bar::baz()` while preserving `_Z3foov -> foo()` and `_ZN3foo3barEv -> foo::bar()` regressions.
- [x] [depends:1.1] 1.2 Add or update derivation/evidence artifacts so the selected deeper nested-name demangle case has checked bounded semantic evidence. Evidence: `bootstrap/gcc-4.0.ncl` now supports the selected three-component zero-argument nested Itanium shape, keeps static-buffer/no-libc-heavy smoke style, and `bootstrap/evidence/gcc-4.0-native-demangle-slice.json` records schema `mantle-gcc40-native-demangle-slice-v2` with BLAKE3 transcript/output digests.
- [x] [depends:1.2] 1.3 Update bootstrap parity validation to require the new/updated demangle evidence, fail closed on drift, and keep `gcc.4.0` evidence-backed `partial`. Evidence: `src/bootstrap_parity.rs` now requires the v2 receipt, deep nested contract, preserved flat/two-component regressions, stale-marker rejection, and bounded parity effect; `bootstrap/evidence/gcc-4.0-native-boundary.json` names the promoted deep nested slice.
- [x] [depends:1.3] 1.4 Add positive and negative tests for receipt acceptance, missing receipt, marker/digest drift, unsupported schema or selected shape, malformed/deeper unsupported cases, and accidental full-parity overclaim. Evidence: `cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture` passed 34 focused GCC 4.0 parity tests.
- [x] [depends:1.4] 1.5 Run focused verification: targeted bootstrap parity tests, `./scripts/check-bootstrap-parity-snapshot.sh`, `git diff --check`, and strict OpenSpec validation. Evidence: see `validation.md`; snapshot report BLAKE3 `2d7a65542fecfe1225fa5a0e1715fa2817b96f793d4e1c1339286c96daf1a8e6`, blockers remain live-bootstrap 5, guix 6, stagex 2.

## 2. Closeout

- [x] [parallel] 2.1 Scaffold proposal, design, task plan, and bootstrap spec delta.
- [x] [depends:1.5] 2.2 Update task completion notes with exact evidence/commands. Evidence: this file and `validation.md` record the exact commands and bounded claim.
- [x] [depends:2.2] 2.3 Archive the OpenSpec only after implementation and verification are complete. Evidence: ready for same-session sync/archive after strict validation.

## Verification Coverage

Completed evidence:

- `cargo fmt --check`
- `cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture`
- `./scripts/check-bootstrap-parity-snapshot.sh`
- `openspec validate promote-gcc40-demangle-deep-nested-name-slice --strict`
- `openspec validate --all --strict`
- `git diff --check`
