# Verification: rust topology unit variant artifacts

Task-ID: V1
Covers: rust_package_planning.native_unit_variant_artifacts

## Focused checks

Command:

```sh
cargo fmt -p mantle -- src/rust_plan.rs
git diff --check
CARGO_TARGET_DIR=/tmp/mantle-unit-variant-check cargo test -p mantle --bin mantle unit_variant -- --nocapture
CARGO_TARGET_DIR=/tmp/mantle-unit-variant-check cargo test -p mantle --bin mantle native_unit_graph -- --nocapture
```

Result:

```text
running 4 tests
... 4 passed; 0 failed ...

running 9 tests
... 9 passed; 0 failed ...
```

## Dirty self-probe

Command stored in pueue task 241 (`unit-variant-dirty10-self-probe`).

Result from `pueue_log 241`:

```text
probe: target/mantle-self-rust-plan-probe-unit-variant-dirty10/receipt.json
topology_execution_status=success
executions=592
blocker_class=
blocker_message=
```

This proves the prior duplicate same-package producer frontier advanced: native topology executed successfully with unit-variant artifact binding/search paths.

## Cairn gates

Command:

```sh
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate proposal rust-topology-unit-variant-artifacts --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate design rust-topology-unit-variant-artifacts --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate tasks rust-topology-unit-variant-artifacts --root .
```

Result:

```text
validate: valid=true
gate proposal: verdict=PASS
gate design: verdict=PASS
gate tasks: verdict=PASS
```
