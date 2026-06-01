# Final source-built closure validation

Task-ID: V6
Covers: rust_package_planning.source_built_toolchain_closure

## Focused Rust tests

Commands:

```sh
cargo test -p mantle --bin mantle host_dependency -- --nocapture
cargo test -p mantle --bin mantle cargo_free_self_build -- --nocapture
cargo test -p mantle --bin mantle self_build_cli_accepts_cargo_free_target_triple -- --nocapture
```

Results:

- `host_dependency`: 6 passed / 0 failed.
- `cargo_free_self_build`: 23 passed / 0 failed.
- `self_build_cli_accepts_cargo_free_target_triple`: 1 passed / 0 failed.

## Formatting and whitespace

Commands:

```sh
cargo fmt --check -p mantle -v
git diff --check
```

Results: both passed.

## Build

Command:

```sh
cargo build -p mantle --bin mantle
```

Result: pueue task `72` succeeded.

## Cairn validation

Command:

```sh
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate tasks source-built-toolchain-closure --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "input_hash": "99ec24b5c59b906319c316329169080228d75f9a06fee48098361ee0365ece80",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "470ce6b1227f80c70e8b0011a6ddf5e21aa4d80dc8ca6b7f18cfdd1dec09c6e7",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
