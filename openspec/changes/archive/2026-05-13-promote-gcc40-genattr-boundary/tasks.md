## Phase 1: Boundary promotion

- [x] [serial] Replace generic `genattr` object/header labels with named empty-attribute boundary outputs.
- [x] [serial] Add derivation-local `genattr` header checks.
- [x] [serial] Refresh GCC 4.0 placeholder inventory and regression coverage.
- [x] [serial] Run focused eval/build/Rust/OpenSpec verification and archive the change.

## Evidence

- `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` → `gcc-4.0.4 script_bytes 65926 inputs 11`
- `/bin/sh -n /tmp/gcc40-genattr.sh` → passed
- `git diff --check -- bootstrap/gcc-4.0.ncl src/bootstrap_parity.rs bootstrap/evidence/gcc-4.0-placeholder-inventory.json openspec/changes/promote-gcc40-genattr-boundary` → passed
- `cargo fmt --check` → passed
- `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` → 45 passed
- `nix-shell -p bubblewrap --run './target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'` → `/home/brittonr/git/crunch/crunch/.crunch-drain/store/0vdsammmrwbhz961c4amdln9c9w3lvm1-gcc-4.0.4`
- `cargo run -q -p crunch -- --json bootstrap parity-report` → `gcc.4.0 partial unknown failed=None`; blockers live-bootstrap 5, guix 6, stagex 2
- `openspec validate promote-gcc40-genattr-boundary --strict` → passed
- `openspec validate bootstrap --strict` → passed
- `openspec validate --all --strict` → 49 passed, 0 failed
