# Host/target topology split validation

Task-ID: I7
Covers: rust_package_planning.source_built_toolchain_closure

## Change

Implemented native topology split for Cargo-free execution:

- host custom-build/proc-macro units keep host execution kind and host toolchain;
- dependency libraries reachable from host units are cloned as `host-dependency` derivations;
- host dependency artifacts are named with separate `:host-dependency` unit IDs, so host and target variants can coexist;
- target lib/bin units continue to use the requested target triple and receipt-bound source-root target toolchain;
- combined topology ordering includes host dependency participants before host consumers;
- `self-build --cargo-free --target <triple>` threads repeated target triples into stage1/stage2 `rust-plan` calls.

## Focused tests

Command:

```sh
cargo test -p mantle --bin mantle host_dependency -- --nocapture
```

Result: passed 6/6.

Covered cases:

- `host_dependency_derivations_clone_libs_for_host_consumers_without_rewriting_targets`: host consumers receive host-cloned dependency libraries while target units keep target artifacts.
- `combined_unit_topology_orders_host_dependency_lib_before_host_unit_not_target_lib`: combined topology orders the host dependency before the host custom-build/proc-macro consumer and does not use the target lib as that host prerequisite.
- `host_dependency_derivations_do_not_guess_ambiguous_package_fallbacks`: ambiguous package-only dependency facts do not fabricate host clones or pick an arbitrary producer.
- Existing `host_dependency_topology` coverage still checks selected host-unit filtering and native dependency fact handling.

Additional commands:

```sh
cargo test -p mantle --bin mantle cargo_free_self_build -- --nocapture
cargo test -p mantle --bin mantle self_build_cli_accepts_cargo_free_target_triple -- --nocapture
```

Results:

- `cargo_free_self_build`: passed 23/23.
- `self_build_cli_accepts_cargo_free_target_triple`: passed 1/1.

New focused coverage:

- `fixed_point_plan_threads_targets_into_stage_commands`: stage1 and stage2 fixed-point commands both receive `--target x86_64-unknown-linux-musl`.
- `receipt_bound_path_aliases_expose_declared_linker_for_collect2`: receipt-bound PATH now exposes declared `ld`, `cc`, `ar`, and `ranlib` aliases, using executable wrappers instead of symlinks.

## Build validation

Command:

```sh
cargo build -p mantle --bin mantle
```

Result: pueue task `72` succeeded after the host/target split plus `ar`/`ranlib` alias fix and the ambiguous-package negative guard.

## Proof unblock evidence

The previous blocker was:

```text
error E0461: couldn't find crate `cc` with expected target triple x86_64-unknown-linux-gnu
note: found crate `cc`, target triple x86_64-unknown-linux-musl
```

After this split, pueue task `63` ran:

```sh
mantle --json self-build \
  --cargo-free \
  --out /tmp/mantle-source-built-closure-self-build-target-20260601T012138Z \
  --rustc .pi/source-built-closure-proof-20260601T004426Z/rustc-source-root-target-wrapper \
  --target x86_64-unknown-linux-musl \
  --toolchain-closure .pi/source-built-closure-proof-20260601T004426Z/toolchain-closure-with-binutils.json
```

Result summary copied to `evidence/source-built-closure-self-build-success-summary.json`:

- `status`: `success`
- `execution_status`: `success`
- `unit_count`: 686
- `failed_unit_count`: 0
- `cargo_marker_absent`: true
- `smoke_status_code`: 0
- `binary_blake3`: `990505fac1c7c6ef584b7a4d2d9c09d29b4f2036c03f023bad85ded44294aaef`
- policy digest: `d9743a184149959bcb6cf3ad77f6b090e80ddceb06194bc3dadd573b30e949a4`

This proves the `aws-lc-sys` host/target blocker moved to success for one full Cargo-free self-build with the source-root target triple.
