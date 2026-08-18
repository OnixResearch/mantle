# Current blocker

## Baseline focused tests

- `cargo test -p mantle --bin mantle host_dependency_topology_accepts_proc_macro_host_producer -- --nocapture`
  - Result: `1 passed; 0 failed` before this change.
- `cargo test -p mantle --bin mantle unit_derivation_graph_binds_proc_macro_host_artifacts_to_target_units -- --nocapture`
  - Result: `1 passed; 0 failed` before this change.

## Self-probe frontier before change

From `target/mantle-self-rust-plan-probe-after-eb0760a7-clean/blocker-summary.txt`:

```text
topology_execution=blocked
topology_unit_executions=5
metadata_runs=3

metadata packages:
registry+https://github.com/rust-lang/crates.io-index#anyhow@1.0.102 status=success
registry+https://github.com/rust-lang/crates.io-index#assert_cmd@2.2.0 status=success
registry+https://github.com/rust-lang/crates.io-index#async-io@2.6.0 status=success

blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: error[E0432]: unresolved import `proc_macro`
 --> /home/brittonr/git/mantle/vendor-deps/async-stream-impl/src/lib.rs:1:5
  |
1 | use proc_macro::TokenStream;
  |     ^^^^^^^^^^ use of unresolved module or unlinked crate `proc_macro`
```

The same unit also lacks normal dependency artifacts in the reviewable graph:

```json
{
  "package_id": "registry+https://github.com/rust-lang/crates.io-index#async-stream-impl@0.3.6",
  "target_kind": "proc-macro",
  "execution_kind": "host",
  "dependency_artifacts": []
}
```
