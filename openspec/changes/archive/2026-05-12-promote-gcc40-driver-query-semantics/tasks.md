## Phase 1: GCC 4.0 driver query slice

- [x] [serial] Implement bounded GCC driver query flags for version, machine, libgcc file, and search directories.
  - Evidence: `bootstrap/gcc-4.0.ncl` generated driver now handles `-dumpversion`, `-dumpmachine`, `-print-libgcc-file-name`, and `-print-search-dirs` before delegating compilation to the TinyCC pass1 handoff.
- [x] [depends:implementation] Add derivation-local and parity regression evidence for the query contract.
  - Evidence: derivation checks `-dumpversion=4.0.4`, `-dumpmachine=x86_64-unknown-linux-musl`, `-print-libgcc-file-name` equals the installed `libgcc.a`, the queried file exists, and `-print-search-dirs` includes the installed GCC libdir. `src/bootstrap_parity.rs` includes `gcc40_real_derivation_contains_driver_query_smoke`.
- [x] [depends:verification] Run eval/shell syntax, build, driver smoke, parity tests/report, OpenSpec validation, and whitespace checks.
  - Evidence at 2026-05-12T21:38:08Z: `cargo fmt --check`; `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` passed with 38 tests; `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` plus `/bin/sh -n` passed; `nix-shell -p bubblewrap --run './target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'` built `.crunch-drain/store/kx1c8c10wdhn6glzbyp1xwibys195xcr-gcc-4.0.4`; a bwrap smoke with `.crunch-drain/store` bound at logical `/crunch/store` verified the driver query flags and a compile smoke; `cargo run -q -p crunch -- --json bootstrap parity-report` kept `gcc.4.0` partial with no evidence failure; `openspec validate promote-gcc40-driver-query-semantics --strict`, `openspec validate --all --strict`, and `git diff --check` passed.
- [x] [depends:archive] Archive, commit, and push the verified change.
  - Evidence: ready for archive after the verification transcript above.
