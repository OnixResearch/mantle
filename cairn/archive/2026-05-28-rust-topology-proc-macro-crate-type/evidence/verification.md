# Verification

## Baseline before changes

```sh
pueue_log 88
```

Result:

- `native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only`: `1 passed`.
- `native_host_derivation_adds_compiler_proc_macro_extern`: `1 passed`.

## Focused tests after implementation

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CARGO_TARGET_DIR=/tmp/mantle-proc-macro-crate-type-test
cargo test -p mantle --bin mantle proc_macro_crate_type -- --nocapture
cargo test -p mantle --bin mantle native_manifest_proc_macro_alias_feeds_target_planning -- --nocapture
cargo test -p mantle --bin mantle native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only -- --nocapture
```

Result:

- `proc_macro_crate_type`: `3 passed`.
- `native_manifest_proc_macro_alias_feeds_target_planning`: `1 passed`.
- `native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only`: `1 passed`.

Additional checks:

```sh
/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --check src/rust_plan.rs
git diff --check
```

Result: both passed with no output.

## Dirty self-probe

First dirty probe (`pueue_log 95`) proved the planned crate-type-only fix was incomplete: `spez@0.1.2` still executed as `lib`, because native package target planning missed Cargo-normalized `[lib] proc_macro = true`.

Final dirty probe:

```sh
pueue_log 99
```

Summary from `target/mantle-self-rust-plan-probe-proc-macro-crate-type-dirty2/blocker-summary.txt`:

```text
probe_status=0
topology_execution=blocked
topology_unit_executions=378
metadata_runs=63

spez executions:
registry+https://github.com/rust-lang/crates.io-index#spez@0.1.2 proc-macro status=success

blocker classes:
      1 rustc-failed

topology blocker:
- rustc-failed: error[E0463]: can't find crate for `rand`
  --> /home/brittonr/git/mantle/vendor-deps/object_store/src/client/backoff.rs:18:5
```

## Cairn validation and gates

Initial updated validation:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Result: `valid=true`, `changes=1`, `specs_validated=2`.

Updated stage gates:

```sh
pueue_log 105
```

Result:

- Proposal gate: `valid=true`, `verdict=PASS`.
- Design gate: `valid=true`, `verdict=PASS`.
- Tasks gate: `valid=true`, `verdict=PASS`.

Sync and validation:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- sync rust-topology-proc-macro-crate-type --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Result:

- `sync`: `blocked=false`, `mutated=true`.
- `validate`: `valid=true`, `changes=1`, `specs_validated=2`.

## Clean self-probe

```sh
pueue_log 114
```

Summary from `target/mantle-self-rust-plan-probe-after-a360a9d4-clean/blocker-summary.txt`:

```text
head: a360a9d4
git_status_short_bytes=0

probe_status=0
topology_execution=blocked
topology_unit_executions=378
metadata_runs=63

spez executions:
registry+https://github.com/rust-lang/crates.io-index#spez@0.1.2 proc-macro status=success

blocker classes:
      1 rustc-failed

topology blocker:
- rustc-failed: error[E0463]: can't find crate for `rand`
  --> /home/brittonr/git/mantle/vendor-deps/object_store/src/client/backoff.rs:18:5
```

## Oracle checkpoint

- Question: Does the code-bearing implementation tree (clean probe head `a360a9d4`; later evidence-only amend does not change `src/rust_plan.rs` or accepted specs) move native topology past the `spez@0.1.2` proc-macro crate-type blocker without hiding the next failure?
- Inspected evidence: focused tests above; dirty self-probe task `99`; clean self-probe task `114`; `target/mantle-self-rust-plan-probe-after-a360a9d4-clean/receipt.json`; `target/mantle-self-rust-plan-probe-after-a360a9d4-clean/blocker-summary.txt`.
- Decision: Yes. `spez@0.1.2` now runs as `proc-macro` and succeeds in a clean tree (`git_status_short_bytes=0`). The topology advances from 352 to 378 unit executions. The next deterministic frontier is `object_store` failing to find crate `rand` during direct rustc execution.
- Owner: coding agent.
- Next action: archive this completed Cairn change, then treat `object_store` missing `rand` artifact binding as the next native topology frontier.
