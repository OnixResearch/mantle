# Native rust-plan validation

This page records focused validation rails for the `no_std + alloc`
`mantle-rust-plan-core`, the `mantle-rust-plan-app` application, and their
`src/rust_plan.rs` adapter. These rails exercise bounded planning, typed
observations, and local cache behavior; they do not prove full Cargo
compatibility, rustc correctness, release reproducibility, or bootstrap
correctness.

## Host setup

Use the checked-in Rust toolchain and the Nix development environment.
Keep temporary files and build products in writable scratch space outside
the checkout; a full `/tmp` or concurrent Cargo target can invalidate a
validation run without implicating Rust-plan:

```bash
export TMPDIR="$HOME/scratch"
export CARGO_TARGET_DIR="$HOME/scratch/rust-plan-validation-target"
nix develop --offline --no-write-lock-file -c cargo test \
  -p mantle-rust-plan-core --locked --offline
nix develop --offline --no-write-lock-file -c cargo test \
  -p mantle-rust-plan-app --locked --offline
nix develop --offline --no-write-lock-file -c cargo check \
  -p mantle-rust-plan-core --target wasm32-unknown-unknown --locked --offline
```

The core and application packages test the pure policy and orchestration
contracts separately. The target check verifies that core policy compiles
without host filesystem, process, or cache facilities; it does not execute
Rust code under WebAssembly.

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

Cargo-oracle metadata and unit-graph children are captured with bounded pipe
readers: at most 64 MiB of stdout and 1 MiB of stderr. If either limit is
exceeded, the adapter kills and reaps the child and returns a build failure;
it never waits for an unbounded `Command::output()` capture. The focused
`cargo test -p mantle --bin mantle rust_plan::tests::cargo_oracle_` rail covers
a real small `/bin/sh` child and both oversized pipe directions.

## Local castore cache

The local Rust unit cache is optional and disabled by default. Select one mode
with an explicit execution command:

```bash
mantle rust-plan --execute-topology --local-rust-cache read
mantle rust-plan --execute-topology --local-rust-cache read-write
```

`read` permits verified restore operations. `read-write` also permits
publication after successful compiler execution. Mantle keeps prior execution
output reuse first. It checks local castore results second and runs the compiler
only after a miss or rejection.

The action identity includes the declared source, compiler content, toolchain
closure, platform, target, profile, mode, features, semantic arguments,
admitted environment, dependency artifacts, host artifacts, build-script
facts, native-link facts, and compiler policy. An unclassified absolute path
makes that action ineligible for cache reuse. Compilation can still continue.

Each execution receipt can include a `local_cache` report. The report gives the
disposition, reason codes, selected result reference, candidate count, artifact
count, restored and reused byte counts, and whether the compiler ran. A restored
unit gets a new execution receipt. The stored result tree excludes that receipt.

Mantle stores local records under the resolved state directory at
`rust-unit-cache/`. Result records and indexes are private, bounded, and
no-follow. `mantle store gc` validates committed retained results and passes
their castore nodes as explicit roots. GC keeps live Rust result content and
removes unrooted content. It also removes unretained result records. The
operator who runs `mantle store gc` has deletion authority.

This cache does not prove compiler correctness, output reproducibility,
hermeticity, remote trust, or release eligibility. Object presence alone never
authorizes reuse.

Focused cache and performance rail:

```bash
cargo test -p crunch-rust-cache-core
cargo test -p crunch-rust-cache
cargo test -p crunch-store \
  gc::tests::explicit_castore_root_survives_while_unreachable_blob_is_reclaimed \
  -- --exact --nocapture
cargo test -p mantle --bin mantle \
  rust_plan::tests::local_castore_restoration_skips_second_compiler_invocation \
  -- --exact --nocapture
```

The final test prints cold and restored wall-clock microseconds. It also checks
that the restored run does not invoke the compiler.

## Shared Rust result cache

Shared cache use requires an enabled local cache. This rule keeps local output
and local castore reuse ahead of remote transfer.

Focused positive and negative rails:

```bash
cargo test -p crunch-rust-cache shared::tests:: -- --nocapture
cargo test -p mantle --bin mantle \
  rust_plan::tests::clean_client_shared_cache_hit_skips_second_compiler_invocation \
  -- --exact --nocapture
cargo test -p mantle --bin mantle \
  rust_plan::tests::untrusted_shared_candidate_records_rejection_before_compiler_fallback \
  -- --exact --nocapture
```

These tests cover signed publication, clean-client restore, zero additional
compiler calls, fallback evidence, conflicts, offline behavior, redirects,
timeouts, truncation, corruption, bounds, and atomic visibility.

See [`shared-rust-unit-cache.md`](shared-rust-unit-cache.md) for operator
commands, trust policy, protocol paths, receipt fields, and non-claims.

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
