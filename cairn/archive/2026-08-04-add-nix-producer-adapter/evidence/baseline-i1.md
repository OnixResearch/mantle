# I1 baseline evidence

Date: 2026-08-04. Worktree: `.pi/worktrees/add-fix-nix-producer`, branch `cairn/add-fix-nix-producer`, base `origin/main` (`6b956dbc`).

Environment: host rustup nightly, clang-wrapper 21.1.8, mold 2.41.0, pkg-config wrapper, openssl-3.6.1-dev `PKG_CONFIG_PATH`, `SNIX_BUILD_SANDBOX_SHELL` = busybox-static 1.37.0, bubblewrap 0.11.2 on `PATH`. `nix develop` was not usable in this session: its flake evaluation failed fetching a private input (`git.onix.computer`), so the documented host toolchain paths from `AGENTS.md` were used.

## Commands

```text
cargo test -p mantle --bin mantle foreign_derivation_import
cargo test -p mantle --bin mantle source_bundle
cargo test -p mantle --test foreign_import_cli
```

Note: `--lib` filters match 0 tests because both modules live in the binary target (`src/main.rs`), not `src/lib.rs`.

## Results

```text
foreign_derivation_import: test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2147 filtered out
source_bundle:             test result: ok. 85 passed; 0 failed; 0 ignored; 0 measured; 2082 filtered out
foreign_import_cli:        test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Baseline gotcha recorded

Without `bwrap` on `PATH`, `foreign_import_cli_realizes_two_node_graph_and_reuses_exact_outputs` fails with `bubblewrap executable 'bwrap' not found; put 'bwrap' on PATH or set SNIX_BUILD_BWRAP`. With bubblewrap 0.11.2 on `PATH`, all 14 CLI tests pass. This is an environment requirement, not a code regression.
