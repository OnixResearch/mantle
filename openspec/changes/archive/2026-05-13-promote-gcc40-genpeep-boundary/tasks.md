## Phase 1: Boundary implementation

- [x] [serial] Split `genpeep` from the grouped generated-source wrapper and emit checked empty-peephole boundary output.
- [x] [serial] Add derivation-local checks and parity regression coverage for the `genpeep` boundary.
- [x] [serial] Refresh placeholder inventory and update cumulative bootstrap spec without dropping prior GCC ladder scenarios.
- [x] [serial] Verify eval/shell/build/Rust/OpenSpec gates, archive the change, commit, and push.

## Evidence

- `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` wrote `/tmp/gcc40-genpeep.json`: `gcc-4.0.4 script_bytes 71263 inputs 11`.
- `/bin/sh -n /tmp/gcc40-genpeep.sh` passed.
- `cargo fmt --check` passed.
- `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` passed: 50 passed, 0 failed.
- `cargo run -q -p crunch -- --json bootstrap parity-report` passed; `gcc.4.0` remains `partial`, blockers remain live-bootstrap 5, guix 6, stagex 2.
- `nix-shell -p bubblewrap --run './target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'` passed with hermeticity practical and artifact `.crunch-drain/store/nkbyxa16zvd4krb2fprzm2sd4h07pryc-gcc-4.0.4`.
- `openspec validate promote-gcc40-genpeep-boundary --strict` passed.
- `openspec validate --all --strict` passed: 49 passed, 0 failed before archive.
- `git diff --check` passed before build/all-spec validation.
