## Phase 1: Demangle Boundary

- [x] [serial] Replace the generic libiberty cp-demangle stub marker with a named disabled-demangle boundary source.
- [x] [serial] Update native-frontier evidence and parity tests to require the boundary and reject the old marker.
- [x] [serial] Verify eval/build/Rust/OpenSpec gates and archive the change.

Evidence:
- `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` → `gcc-4.0.4 script_bytes 74352 inputs 11`; `/bin/sh -n /tmp/gcc40-demangle.sh` passed.
- `cargo fmt --check` passed.
- `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` → 55 passed.
- `nix-shell -p bubblewrap --run './target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'` passed, artifact `.crunch-drain/store/0vdsammmrwbhz961c4amdln9c9w3lvm1-gcc-4.0.4`.
- `cargo run -q -p crunch -- --json bootstrap parity-report` kept `gcc.4.0 partial unknown`; blockers live-bootstrap 5, guix 6, stagex 2.
- `openspec validate --all --strict` → 49 passed before archive.
- `git diff --check` passed.
