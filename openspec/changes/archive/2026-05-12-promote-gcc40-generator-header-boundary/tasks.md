## Phase 1: Generator Header Boundary

- [x] [serial] Replace generic genconstants/genflags stub labels with empty-machine boundary headers.
  - Evidence: `bootstrap/gcc-4.0.ncl` now emits `Crunch GCC 4.0 empty-machine constants boundary` and `Crunch GCC 4.0 empty-machine flags boundary` for the early generator-header seam.
- [x] [serial] Add derivation-local and parity regression evidence for the boundary.
  - Evidence: derivation-local checks verify `GCC_INSN_CONSTANTS_H` / `GCC_INSN_FLAGS_H`, require the empty-machine boundary markers, and reject legacy `bootstrap genconstants s[t]ub` / `bootstrap genflags s[t]ub` labels.
  - Evidence: `gcc40_real_derivation_contains_generator_header_boundary_checks` covers the real derivation text.
- [x] [serial] Refresh GCC 4.0 placeholder inventory and verify Rust/OpenSpec/bootstrap gates.
  - Evidence timestamp: `2026-05-12T23:26:25Z`.
  - `cargo fmt --check`.
  - `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` — 42 passed.
  - `cargo run -q -p crunch -- --json bootstrap parity-report` — `gcc.4.0 partial unknown failed=False` by row inspection.
  - `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` plus `/bin/sh -n /tmp/gcc40-generator-header.sh`.
  - `./target/debug/crunch build bootstrap/gcc-4.0.ncl --store /home/brittonr/git/crunch/crunch/.crunch-drain/store --no-substitute` — built `/home/brittonr/git/crunch/crunch/.crunch-drain/store/017lmy24pwq6vrq76w6dyf3034ajn6z7-gcc-4.0.4`.
  - `openspec validate promote-gcc40-generator-header-boundary --strict`.
  - `openspec validate bootstrap --strict`.
  - `openspec validate --all --strict` — 49 passed, 0 failed.
  - `git diff --check`.
- [x] [serial] Archive the change after verification.
