# Native Rust Cargo replacement change index

These are scaffold Cairn changes for the remaining work to make Mantle's Rust planner/executor a real Cargo alternative. Tasks are intentionally unchecked; each change needs implementation and verification before archive.

## Order

1. `rust-topology-host-real-unit-identity` — fix current done-review blocker: duplicate proc-macro/custom-build host unit identity.
2. `native-rust-manifest-lock-planner` — parse manifests and lockfiles without Cargo metadata.
3. `native-rust-feature-resolution` — compute selected features and optional dependency activation natively.
4. `native-rust-unit-graph-parity` — construct supported lib/bin/proc-macro/custom-build unit graphs without Cargo unit-graph oracle.
5. `native-rust-build-script-runtime-parity` — make build-script env and metadata behavior explicit enough for real crates.
6. `native-rust-executor-hardening` — harden cache, receipts, stale artifact detection, and deterministic failures.
7. `native-rust-no-cargo-oracle-cli` — expose a no-Cargo planner/executor CLI mode and guard against Cargo use.
8. `native-rust-cargo-free-proof` — produce audit-grade Cargo-free self-build evidence.

## Validation transcript

Command run after creating all eight changes:

```text
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn status --root .
```

Result summary:

```text
validate: valid=true, changes=8, specs_validated=9, issues=[]
status: 8 active changes, all with proposal/design/tasks/spec delta; all tasks todo
```

Gate check summary:

```text
rust-topology-host-real-unit-identity proposal/design/tasks PASS
native-rust-manifest-lock-planner proposal/design/tasks PASS
native-rust-feature-resolution proposal/design/tasks PASS
native-rust-unit-graph-parity proposal/design/tasks PASS
native-rust-build-script-runtime-parity proposal/design/tasks PASS
native-rust-executor-hardening proposal/design/tasks PASS
native-rust-no-cargo-oracle-cli proposal/design/tasks PASS
native-rust-cargo-free-proof proposal/design/tasks PASS
```
