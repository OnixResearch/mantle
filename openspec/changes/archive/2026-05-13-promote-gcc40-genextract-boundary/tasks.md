## Phase 1: Boundary implementation

- [x] [serial] Split `genextract` from the grouped generated-source wrapper and emit checked empty-extraction boundary output. ✅ verified in `bootstrap/gcc-4.0.ncl`
- [x] [serial] Add derivation-local checks and parity regression coverage for the `genextract` boundary. ✅ `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` → 49 passed
- [x] [serial] Refresh placeholder inventory and update cumulative bootstrap spec without dropping prior GCC ladder scenarios. ✅ `bootstrap/evidence/gcc-4.0-placeholder-inventory.json` refreshed; `openspec validate bootstrap --strict` passed
- [x] [serial] Verify eval/shell/build/Rust/OpenSpec gates, archive the change, commit, and push. ✅ eval/shell/build/parity/OpenSpec/diff gates passed before archive

## Evidence

- `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` → `gcc-4.0.4 script_bytes 70176 inputs 11`
- `/bin/sh -n /tmp/gcc40-genextract.sh` → passed
- `nix-shell -p bubblewrap --run './target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'` → `.crunch-drain/store/0vdsammmrwbhz961c4amdln9c9w3lvm1-gcc-4.0.4`
- `cargo fmt --check` → passed
- `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` → 49 passed
- `cargo run -q -p crunch -- --json bootstrap parity-report` → `gcc.4.0 partial unknown failed=False`; live-bootstrap blocking 5, guix blocking 6, stagex blocking 2
- `openspec validate promote-gcc40-genextract-boundary --strict` → valid
- `openspec validate bootstrap --strict` → valid
- `openspec validate --all --strict` → 49 passed before archive
- `git diff --check` → passed
