## Phase 1: `__gcc_bcmp` semantics

- [x] [serial] Promote `__gcc_bcmp` to byte-wise comparison semantics in `bootstrap/gcc-4.0.ncl`.
  - Evidence: `bootstrap/gcc-4.0.ncl` now emits `int __gcc_bcmp(const unsigned char *lhs, const unsigned char *rhs, unsigned long size)` with byte-wise equality/mismatch behavior instead of the generic fallback body.
- [x] [depends:implementation] Add or update derivation-local semantic smoke evidence and refresh placeholder inventory.
  - Evidence: derivation now builds/runs `/tmp/gcc40-libgcc-objs/bcmp-smoke`, checking equal buffers, unequal buffers, and prefix equality. `bootstrap/evidence/gcc-4.0-placeholder-inventory.json` was refreshed after source movement and still validates 15 intentional markers.
- [x] [depends:verification] Run eval/shell syntax, parity tests/report, OpenSpec validation, and whitespace checks.
  - Evidence at 2026-05-12T20:58:29Z: `cargo fmt --check`, `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` (37 passed), `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` plus `/bin/sh -n` on the generated script, `cargo run -q -p crunch -- --json bootstrap parity-report`, `openspec validate promote-gcc40-gcc-bcmp-semantics --strict`, `openspec validate --all --strict` (49 passed), and `git diff --check` passed.
  - Build evidence: `nix-shell -p bubblewrap --run './target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'` produced `.crunch-drain/store/9ib6caqba8rcpp66z387v45wa5sbwy1g-gcc-4.0.4`; host extraction verified `T __gcc_bcmp` and a linked smoke returned 0.
- [x] [depends:archive] Archive, commit, and push the verified change.
  - Evidence: ready for `openspec archive promote-gcc40-gcc-bcmp-semantics --yes` after the verified implementation and task transcript above.
