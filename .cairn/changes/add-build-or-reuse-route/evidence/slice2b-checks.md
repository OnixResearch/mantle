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
