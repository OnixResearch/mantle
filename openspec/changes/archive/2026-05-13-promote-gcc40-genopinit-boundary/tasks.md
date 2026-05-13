## Phase 1: Boundary implementation

- [x] [serial] Split `genopinit` from the grouped generated-source wrapper and emit checked empty-opinit boundary output.
- [x] [serial] Add derivation-local checks and parity regression coverage for the `genopinit` boundary.
- [x] [serial] Refresh placeholder inventory and update cumulative bootstrap spec without dropping prior GCC ladder scenarios.
- [x] [serial] Verify eval/shell/build/Rust/OpenSpec gates, archive the change, commit, and push.

## Evidence

- `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` wrote `/tmp/gcc40-genopinit.json`: `gcc-4.0.4 script_bytes 72372 inputs 11`.
- `/bin/sh -n /tmp/gcc40-genopinit.sh` passed.
- `cargo fmt --check` passed.
- `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` passed: 51 passed, 0 failed.
- `cargo run -q -p crunch -- --json bootstrap parity-report` passed; `gcc.4.0` remains `partial`, blockers remain live-bootstrap 5, guix 6, stagex 2.
- `nix-shell -p bubblewrap --run './target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'` passed with hermeticity practical and artifact `.crunch-drain/store/0vdsammmrwbhz961c4amdln9c9w3lvm1-gcc-4.0.4`.
- `openspec validate promote-gcc40-genopinit-boundary --strict` passed.
- `openspec validate --all --strict` passed: 49 passed, 0 failed before archive.
- `git diff --check` passed.
