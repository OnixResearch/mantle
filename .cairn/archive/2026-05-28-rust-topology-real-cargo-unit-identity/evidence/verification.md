# Verification: real Cargo unit identity review fix

## Focused checks

Task-ID: V1
Covers: rust_package_planning.native_real_unit_identity

Commands run after implementation:

```text
cargo fmt -p mantle -- src/rust_plan.rs
git diff --check
CARGO_TARGET_DIR=/tmp/mantle-unit-variant-check2 cargo test -p mantle --bin mantle unit_variant -- --nocapture
CARGO_TARGET_DIR=/tmp/mantle-unit-variant-check2 cargo test -p mantle --bin mantle native_unit_graph -- --nocapture
```

Result transcript:

```text
running 5 tests
test rust_plan::tests::target_dependency_producer_index_uses_selected_unit_variant ... ok
test rust_plan::tests::package_only_dependency_artifacts_fail_on_ambiguous_unit_variants ... ok
test rust_plan::tests::bind_dependency_artifacts_uses_selected_unit_variant ... ok
test rust_plan::tests::selected_dependency_search_paths_follow_unit_variant_closure_only ... ok
test rust_plan::tests::native_unit_graph_preserves_duplicate_cargo_unit_variants ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 533 filtered out; finished in 0.00s

running 10 tests
test rust_plan::tests::native_unit_graph_filters_package_self_dependency_from_lib_unit ... ok
test rust_plan::tests::native_unit_graph_preserves_duplicate_cargo_unit_variants ... ok
test rust_plan::tests::native_unit_graph_blocks_selected_dependency_without_native_package_fact ... ok
test rust_plan::tests::native_unit_graph_follows_selected_unit_dependencies_not_all_manifest_dependencies ... ok
test rust_plan::tests::native_unit_graph_keeps_selected_lib_dependency_with_host_target_sibling ... ok
test rust_plan::tests::native_unit_graph_blocks_selected_dependency_without_source_fact ... ok
test rust_plan::tests::native_unit_graph_fragment_blocks_mismatch_and_missing_edges ... ok
test rust_plan::tests::native_unit_graph_blocks_registry_dependency_without_lib_producer ... ok
test rust_plan::tests::native_unit_graph_adds_transitive_registry_producer_unit ... ok
test rust_plan::tests::native_unit_graph_fragment_feeds_supported_unit_derivations ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 528 filtered out; finished in 0.00s
```

## Dirty self-probe

Task-ID: V2
Covers: rust_package_planning.native_real_unit_identity

Pueue task: `254` (`unit-variant-review-fix6-self-probe`)

Probe receipt: `target/mantle-self-rust-plan-probe-unit-variant-review-fix6/receipt.json`

Summary from task log:

```text
probe: target/mantle-self-rust-plan-probe-unit-variant-review-fix6/receipt.json
topology_execution_status=success
executions=609
blocker_class=
blocker_message=
```

## Cairn validation and gates

Task-ID: V3
Covers: rust_package_planning.native_real_unit_identity

Commands:

```text
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate proposal rust-topology-real-cargo-unit-identity --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate design rust-topology-real-cargo-unit-identity --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate tasks rust-topology-real-cargo-unit-identity --root .
```

Result summary:

```text
cairn validate --root .: valid=true, issues=[]
cairn gate proposal rust-topology-real-cargo-unit-identity --root .: verdict=PASS, receipt_hash=cafbf53a53fe2f6eacf376fa2646dfe0fac9ad6eff08289828d45eb429e8fb87
cairn gate design rust-topology-real-cargo-unit-identity --root .: verdict=PASS, receipt_hash=b9f348029e94d4e36cecbec8bbb74268ac3dc3032e1bd3bead7f657bf71b2e63
cairn gate tasks rust-topology-real-cargo-unit-identity --root .: verdict=PASS, receipt_hash=642206aebaafe61d564e9a2761779c26306392753be33cc430c436fc3d9a7ee0
```

## Archive note

Task-ID: V4
Covers: rust_package_planning.native_real_unit_identity

`cairn archive rust-topology-real-cargo-unit-identity --root . --execute` created `cairn/archive/1970-01-01-rust-topology-real-cargo-unit-identity`. Per repo gotcha, the directory was manually renamed to `cairn/archive/2026-05-28-rust-topology-real-cargo-unit-identity`, then `cairn validate --root .` passed with `valid=true` and `cairn status --root .` reported no active changes.

## Clean self-probe after commit

Task-ID: V5
Covers: rust_package_planning.native_real_unit_identity

Pueue task: `255` (`unit-variant-real-identity-clean-self-probe`)

Probe receipt: `target/mantle-self-rust-plan-probe-unit-variant-real-identity-clean/receipt.json`

Summary from task log:

```text
probe: target/mantle-self-rust-plan-probe-unit-variant-real-identity-clean/receipt.json
topology_execution_status=success
executions=609
blocker_class=
blocker_message=
```
