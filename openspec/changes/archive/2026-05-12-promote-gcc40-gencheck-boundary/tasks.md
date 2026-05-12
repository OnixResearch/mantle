## Phase 1: Gencheck Boundary

- [x] [serial] Replace generic gencheck stub labels with disabled-tree-checking boundary output.
  - Evidence: `bootstrap/gcc-4.0.ncl` now emits `gcc40_gencheck_disabled_tree_check_boundary` for the object shim and `Crunch GCC 4.0 disabled-tree-checking boundary` in generated `tree-check.h` output.
- [x] [serial] Add derivation-local and parity regression evidence for the boundary.
  - Evidence: derivation-local checks require `$WORK/build/gcc/build/gencheck`, verify `GCC_TREE_CHECK_H`, require the `disabled-tree-checking boundary` marker, and reject the legacy `bootstrap gencheck s[t]ub` label.
  - Evidence: `gcc40_real_derivation_contains_gencheck_boundary_checks` covers the real derivation text.
- [x] [serial] Refresh GCC 4.0 placeholder inventory and verify Rust/OpenSpec/bootstrap gates.
  - Evidence timestamp: `2026-05-12T23:41:14Z`.
  - `cargo fmt --check`.
  - `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` — 43 passed.
  - `cargo run -q -p crunch -- --json bootstrap parity-report` — `gcc.4.0 partial unknown failed=False` by row inspection.
  - `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` plus `/bin/sh -n /tmp/gcc40-gencheck.sh`.
  - `./target/debug/crunch build bootstrap/gcc-4.0.ncl --store /home/brittonr/git/crunch/crunch/.crunch-drain/store --no-substitute` — built `/home/brittonr/git/crunch/crunch/.crunch-drain/store/017lmy24pwq6vrq76w6dyf3034ajn6z7-gcc-4.0.4`.
  - `openspec validate promote-gcc40-gencheck-boundary --strict`.
  - `openspec validate bootstrap --strict`.
  - `openspec validate --all --strict` — 49 passed, 0 failed before archive.
  - `git diff --check`.
- [x] [serial] Archive the change after verification.
