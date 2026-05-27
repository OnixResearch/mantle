# Verification Evidence: Native Dependency Feature Edge Scope

Task-ID: native-dependency-feature-edge-scope.V1
Covers: r[rust_package_planning.native_dependency_feature_edge_scope]
Date: 2026-05-27
Decision owner: coding agent

## Review checkpoint

Same-family review reported that implicit optional dependency features from Cargo's unit graph were ignored. The implementation now treats a selected feature whose name equals an optional dependency key as selecting that optional dependency, in addition to explicit `[features]` entries such as `dep:<name>`.

## Commands and observed outputs

### Formatting

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/usr/bin:/bin \
  cargo fmt -p mantle --check -- src/rust_plan.rs tests/rust_plan_cli.rs
```

Observed output: command exited successfully with no stdout/stderr.

### Focused implicit optional feature regression

Pueue task: 46

Command:

```sh
PATH=/run/current-system/sw/bin:/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/etc/profiles/per-user/brittonr/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
RUSTC_WRAPPER= CARGO_TARGET_DIR=target/roi-feature-edge-check \
  cargo test -p mantle --test rust_plan_cli implicit_optional -- --nocapture
```

Observed output excerpt:

```text
running 1 test
test rust_plan_cli_selects_implicit_optional_dependency_feature_from_unit_graph ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out; finished in 0.07s
```

### Focused feature-edge regression set

Pueue task: 47

Command:

```sh
PATH=/run/current-system/sw/bin:/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/etc/profiles/per-user/brittonr/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
RUSTC_WRAPPER= CARGO_TARGET_DIR=target/roi-feature-edge-check \
  cargo test -p mantle --test rust_plan_cli feature -- --nocapture
```

Observed output excerpt:

```text
running 5 tests
test rust_plan_cli_blocks_all_features_vendored_registry_topology_before_rustc ... ok
test rust_plan_cli_plans_featured_build_dependency_vendored_registry_source_edge ... ok
test rust_plan_cli_selects_implicit_optional_dependency_feature_from_unit_graph ... ok
test rust_plan_cli_does_not_select_disabled_default_feature_dependency_in_unified_topology ... ok
test rust_plan_cli_executes_feature_gated_vendored_registry_dependency_in_unified_topology ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.10s
```

### Mantle self `rust-plan --execute-topology` probe

Baseline pueue task: 19 (`target/mantle-self-rust-plan-probe/receipt.json`)
Current pueue task: 49 (`target/mantle-self-rust-plan-probe-after-review-fix/receipt.json`)

Oracle checkpoint:

- Question: Did the feature-edge change reduce the self-probe blocker classes it targeted?
- Inspected evidence: baseline receipt `target/mantle-self-rust-plan-probe/receipt.json` and current receipt `target/mantle-self-rust-plan-probe-after-review-fix/receipt.json`.
- Baseline receipt SHA-256: `dc5306c3948c044643552883dbf65b67fdf24414f8b14235ebd4a7cb6917219e`.
- Current receipt SHA-256: `f8660268b78ae47a75a16052e79269ee227aae92d5e4129dbfc3f7f136d5bd3e`.
- Decision owner: coding agent.
- Decision: targeted blocker classes decreased; package readiness remains blocked by the separate `snix-castore`/`wu-manber` source fragment.
- Next action: implement native git/non-path source handling or bounded source facts for `wu-manber`.

Baseline receipt excerpt:

```text
topology_execution=blocked
source_closure=true
native_registry_source_planning=true
native_package_target_planning=false
native_unit_graph_planning=false
native_host_unit_graph_planning=false
unit_derivation_graph=true

blocker classes:
     42 unresolved-host-consumer-edge
     28 unresolved-path-dependency-edge
      8 unsupported-build-dependency-options
      4 unsupported-non-path-dependency
      2 native-package-target-planning-blocked
      1 native-unit-graph-planning-blocked
      1 native-missing-cargo-package
      1 native-host-unit-graph-blocked

baseline package target blockers include:
- unsupported-build-dependency-options: build dependency `bindgen` uses unsupported feature/default-feature/optional behavior
- unsupported-build-dependency-options: build dependency `rustc_version` uses unsupported feature/default-feature/optional behavior
- unsupported-build-dependency-options: build dependency `pyo3-build-config` uses unsupported feature/default-feature/optional behavior
- unsupported-non-path-dependency: dependency `ahash` is outside the bounded path-or-declared-registry-dependency fragment
```

Current command output excerpt:

```text
probe_status=0
4739660 target/mantle-self-rust-plan-probe-after-review-fix/receipt.json
```

Current receipt summary from `target/mantle-self-rust-plan-probe-after-review-fix/blocker-summary.txt`:

```text
topology_execution=blocked
source_closure=true
native_registry_source_planning=true
native_package_target_planning=false
native_unit_graph_planning=false
native_host_unit_graph_planning=false
unit_derivation_graph=true

blocker classes:
     14 unresolved-host-consumer-edge
      5 unresolved-path-dependency-edge
      2 native-package-target-planning-blocked
      1 unsupported-non-path-dependency
      1 native-unit-graph-planning-blocked
      1 native-missing-cargo-package
      1 native-host-unit-graph-blocked

package target blockers:
- native-missing-cargo-package: Cargo oracle workspace package is absent from native planning fragment
- unsupported-non-path-dependency: dependency `wu-manber` is outside the bounded path-or-declared-registry-dependency fragment
```

Baseline-vs-current blocker class comparison:

| Class | Baseline task 19 | Current task 49 | Change |
| --- | ---: | ---: | ---: |
| `unresolved-host-consumer-edge` | 42 | 14 | -28 |
| `unresolved-path-dependency-edge` | 28 | 5 | -23 |
| `unsupported-build-dependency-options` | 8 | 0 | -8 |
| `unsupported-non-path-dependency` | 4 | 1 | -3 |
| `native-package-target-planning-blocked` | 2 | 2 | 0 |
| `native-unit-graph-planning-blocked` | 1 | 1 | 0 |
| `native-missing-cargo-package` | 1 | 1 | 0 |
| `native-host-unit-graph-blocked` | 1 | 1 | 0 |

Remaining package-target blocker chain has two reported blocker records: `snix-castore` is absent from native package facts, and that absence is caused by the still-unsupported non-path/git dependency `wu-manber`. Both records are outside this feature-edge scope; the next implementation slice should add native git/non-path source handling or a bounded source fact for `wu-manber`.

### Cairn validation after evidence checkpoint

Pueue task: 50

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
```

Observed output excerpt:

```json
{
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "specs_validated": 1,
  "valid": true
}
```

## Decision

The feature-edge implementation is accepted for this change because explicit default-feature dependencies, disabled default-feature dependencies, build-dependency feature metadata, and implicit optional dependency features are covered by focused CLI tests. The remaining self-probe blocker chain is outside this change: `snix-castore` is still missing from native package facts because its non-path/git dependency `wu-manber` is unsupported by the current native source fragment.
