# Slice 2b checks: negative CLI tests, effect plan, operator contract, docs

All commands ran from the worktree with the prescribed build environment
(`CARGO_TARGET_DIR=/datapool/mantle-store/build-or-reuse-20261122/mantle-target`,
bubblewrap and clang-wrapper on `PATH`).

| Check | Result |
|---|---|
| `cargo check -p mantle --bin mantle` | exit 0 |
| `cargo build -p mantle --bin mantle --bin generate-operator-command-contract` | exit 0 |
| `mantle __operator-contract --mode raw-descriptors > config/operator-command-descriptors.json` | exit 0 (new `--trusted-builder-public-key` descriptor) |
| `nickel typecheck config/operator-surfaces.ncl` + export to `config/operator-surfaces.json` | exit 0 |
| `mantle __operator-contract --mode catalog/reference/workflow` regenerated | exit 0 |
| `mantle __operator-contract --mode check` | exit 0: `PASS (commands=215)` |
| `cargo test -p mantle --test cache_inputs_build_route` | exit 0: 14 passed; 0 failed |
| `cargo test -p mantle --bin mantle -- foreign_import_cmd::tests::realize_effect_plan` | exit 0: 1 passed |
| `cargo test -p mantle --bin mantle -- foreign_realization foreign_executable_plan` | exit 0: 27 passed |
| `cargo fmt --check -p mantle` | exit 0 (plain workspace `cargo fmt --check` also reports pre-existing vendored/nix-compat diffs) |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 101: 11 `cloned_ref_to_slice_refs` errors in `src/foreign_realization.rs` unit tests (pre-existing on this branch) |
