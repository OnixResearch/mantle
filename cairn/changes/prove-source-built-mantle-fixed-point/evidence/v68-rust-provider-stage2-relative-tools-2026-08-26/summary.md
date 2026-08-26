# V68 Rust-provider stage-tool boundary

## Result

V68 failed closed after the validated V61 native-prefix import. The first Rust-provider action observed 67,339 executions. It matched 67,337 and denied two.

The denied paths were:

- `output/prefix-s/bin/rustc`
- `output/prefix-s/bin/cargo`

Both paths came from command recipes in `run_rustc/Makefile`. The protected-exec supervisor rejected them because they were relative executable paths.

## Repair

The first-stage script now rewrites only tab-prefixed Make command recipes. It changes direct `$(BINDIR_S)rustc` and `$(BINDIR_S)cargo` executions to Make `abspath` expressions after all other `run_rustc` source patches complete.

The rewrite does not change target or prerequisite paths. It rejects repeated executable tokens and fails if either reviewed recipe family is absent. The authenticated MRustC source archive still supplies the bounded Makefile authority.

## Validation

- Baseline Rust-provider suite: 64 passed before the change.
- Focused positive and negative stage-tool tests: 2 passed.
- Post-change Rust-provider suite: 66 passed, 0 failed.
- `cargo check --bin mantle`: passed.
- `rustfmt --edition 2024 --check src/rust_source_provider.rs`: passed.
- `git diff --check`: passed.
- Strict bin-scoped Clippy reached only existing findings in unrelated files. It reported no finding in `src/rust_source_provider.rs`. The retained baseline findings are in `cargo_free_self_build.rs`, `remote_nominal.rs`, `source_built_rust_action_plan.rs`, `nix_free_demo_cmd.rs`, and `rust_plan.rs`.

The full transcript is in `post-repair-validation.log`.
