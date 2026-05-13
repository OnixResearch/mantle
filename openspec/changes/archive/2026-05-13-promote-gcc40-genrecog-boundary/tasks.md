## Phase 1: Boundary tightening

- [x] [serial] Split `genrecog` from the grouped generated-source wrapper and emit a checked empty-recognition boundary.
  - Evidence: `bootstrap/gcc-4.0.ncl` now has a dedicated `build/genrecog` wrapper that emits `gcc40_genrecog_empty_recognition_source_boundary` and an `empty-recognition source boundary` marker, while the remaining grouped wrapper starts at `genextract`.
- [x] [serial] Refresh parity evidence and add regression coverage for the `genrecog` boundary.
  - Evidence: `bootstrap/evidence/gcc-4.0-placeholder-inventory.json` refreshed; `gcc40_real_derivation_contains_genrecog_boundary_checks` added.
- [x] [serial] Run eval/build/Rust/OpenSpec verification and record evidence.
  - Evidence: `cargo fmt --check`; `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` (`script_bytes 69036`, `inputs 11`) and `/bin/sh -n /tmp/gcc40-genrecog.sh`; `nix-shell -p bubblewrap --run './target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'` -> `.crunch-drain/store/nkbyxa16zvd4krb2fprzm2sd4h07pryc-gcc-4.0.4`; `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` -> 48 passed; `cargo run -q -p crunch -- --json bootstrap parity-report` -> `gcc.4.0 partial unknown failed=None`, blockers live-bootstrap 5, guix 6, stagex 2; `openspec validate bootstrap --strict`; `openspec validate --all --strict` -> 49 passed before archive; `git diff --check`.
- [x] [serial] Archive the OpenSpec change and land the verified commit.
  - Evidence: completed after archive/commit.
