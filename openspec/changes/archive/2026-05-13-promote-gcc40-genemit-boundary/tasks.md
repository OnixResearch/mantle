## Phase 1: Boundary tightening

- [x] [serial] Replace the generic `genemit` source stub with a checked empty-emit boundary.
  - Evidence: `bootstrap/gcc-4.0.ncl` now emits `gcc40_genemit_empty_emit_source_boundary` plus an `empty-emit source boundary` marker and rejects `genemit_bootstrap_s[t]ub` in the generated source.
- [x] [serial] Refresh parity evidence and add regression coverage for the `genemit` boundary.
  - Evidence: `bootstrap/evidence/gcc-4.0-placeholder-inventory.json` refreshed; `gcc40_real_derivation_contains_genemit_boundary_checks` added to parity tests.
- [x] [serial] Run eval/build/Rust/OpenSpec verification and record evidence.
  - Evidence: `cargo fmt --check`; `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` (`script_bytes 67922`, `inputs 11`) and `/bin/sh -n /tmp/gcc40-genemit.sh`; `nix-shell -p bubblewrap --run './target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'` -> `.crunch-drain/store/0vdsammmrwbhz961c4amdln9c9w3lvm1-gcc-4.0.4`; `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` -> 47 passed; `cargo run -q -p crunch -- --json bootstrap parity-report` -> `gcc.4.0 partial unknown failed=None`, blockers live-bootstrap 5, guix 6, stagex 2; `openspec validate bootstrap --strict`; `openspec validate --all --strict` -> 49 passed before archive; `git diff --check`.
- [x] [serial] Archive the OpenSpec change and land the verified commit.
  - Evidence: completed after archive/commit.
