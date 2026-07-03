# Native rust-plan validation

This page records the focused validation rails for Mantle's native `rust-plan`
unit tests. It distinguishes evidence commands from broader Cargo-free proof
claims: these commands prove test fixture determinism for `src/rust_plan.rs`, not
full Cargo compatibility, compiler correctness, release reproducibility, or
bootstrap correctness.

## Host setup

Use the checked-in Rust toolchain and the same local build prerequisites as the
ordinary Mantle Rust validation lane. On the current Nix host, the focused rail
uses this environment shape:

```bash
export PATH="$HOME/.cargo/bin:$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CARGO_TARGET_DIR=/tmp/mantle-rust-plan-validation-target
```

Use a scratch `CARGO_TARGET_DIR` outside the checkout when collecting evidence so
parallel runs do not share stale build products with unrelated local work.

## Focused rails

Serial evidence rail:

```bash
cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=1
```

Parallel evidence rail:

```bash
cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=8
```

Repeated parallel stress rail:

```bash
RUST_PLAN_STRESS_RUNS=3
for run in $(seq 1 "$RUST_PLAN_STRESS_RUNS"); do
  cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=8
done
```

Negative ambient-env rail:

```bash
cargo test -p mantle --bin mantle \
  rust_plan::tests::build_script_child_env_ignores_ambient_profile_env \
  -- --exact --nocapture
cargo test -p mantle --bin mantle \
  rust_plan::tests::rust_plan_tests_do_not_mutate_process_global_environment_in_process \
  -- --exact --nocapture
cargo test -p mantle --bin mantle \
  rust_plan::tests::process_global_env_mutation_guard_detects_set_and_remove_var_calls \
  -- --exact --nocapture
```

## Fixture inventory

| Fixture family | Shared resource risk | Current isolation rule | Serial-only status |
|---|---|---|---|
| Rustc wrapper fixtures | Wrapper executable path and produced fake artifacts | `TempDir` per test via `write_fake_rustc(...)` | Parallel-safe |
| Compiler-policy fixtures | Provider manifest, adapter report, lint/config files, policy report paths | `TempDir` per test plus report paths scoped by `output_root` and unit ID | Parallel-safe |
| Build-script metadata fixtures | `OUT_DIR`, generated metadata, executable current directory | `build_script_out_dir(...)` scopes by absolute output root and safe unit ID | Parallel-safe |
| Topology execution fixtures | Unit output directories and cached execution receipts | Per-test `TempDir` output roots; reuse tests intentionally reuse only within one test | Parallel-safe |
| Ambient profile env negative test | Process-global env values such as `OPT_LEVEL`, `DEBUG`, and `NUM_JOBS` | Parent spawns an exact child test with conflicting env; no in-process `set_var` / `remove_var` calls in `src/rust_plan.rs` tests | Parallel-safe |
| Cargo shim / Cargo-free proof scripts | Failing shim marker and execution root | Script-created proof bundles and explicit `--execution-output-root`; keep bundle roots outside the source tree | Not part of focused unit-test rail |

There are no remaining known serial-only `src/rust_plan.rs` focused tests. The
serial rail remains documented as a baseline evidence command, not because a test
requires serialization.

## Evidence rules

- Claim parallel safety only from a same-tree parallel or repeated-parallel
  transcript.
- If a test becomes serial-only, name the shared resource here and add a next
  action for isolating or locking it.
- Do not replace the negative ambient-env subprocess pattern with in-process
  process environment mutation.
