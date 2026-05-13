## Phase 1: Bounded Demangle Semantics

- [x] [serial] Add a bounded Itanium zero-argument demangle implementation and derivation-local smoke.
  - Evidence: `bootstrap/gcc-4.0.ncl` now provides `gcc40_cplus_demangle_bounded_itanium_v0_boundary` / `gcc40_cp_demangle_bounded_itanium_v0_boundary` and a build-local smoke for `_Z3foov -> foo()`, `_Z3barv -> bar()`, plus negative cases.
- [x] [serial] Update native-frontier evidence and parity tests to require the bounded semantic marker and reject disabled-demangle marker drift.
  - Evidence: `bootstrap/evidence/gcc-4.0-native-boundary.json` now records `libiberty-demangle-bounded-semantics`; `src/bootstrap_parity.rs` rejects `gcc40_cp_demangle_disabled_boundary` and the legacy `libiberty_cp_demangle_bootstrap_stub`.
- [x] [serial] Verify eval/build/Rust/OpenSpec gates and archive the change.
  - Evidence: `crunch eval bootstrap/gcc-4.0.ncl` passed (`gcc-4.0.4 script_bytes 76330 inputs 11`); `/bin/sh -n /tmp/gcc40-demangle-sem.sh` passed; `cargo fmt --check` passed; `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` passed (`55 passed`); `crunch build bootstrap/gcc-4.0.ncl --store .crunch-drain/store -j 4 --trust-unsigned` passed (`.crunch-drain/store/f5qmcxvxkxmc1r6cm7hm5l0padcbwwmq-gcc-4.0.4`); `cargo run -q -p crunch -- --json bootstrap parity-report` kept `gcc.4.0 partial`; `openspec validate --all --strict` passed (`49 passed` before archive); `git diff --check` passed.
