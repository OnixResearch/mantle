# Tasks

## Spec

- [x] [serial] Add Cargo-free CLI requirement and design. r[rust_package_planning.no_cargo_oracle_cli]

## Implementation

- [x] [serial] Add explicit Cargo-free CLI flag or subcommand. r[rust_package_planning.no_cargo_oracle_cli]
  - Evidence: added `rust-plan --no-cargo-oracle`; focused CLI test below passes.
- [x] [serial] Route Cargo-free mode through native parser, resolver, unit graph, and executor only. r[rust_package_planning.no_cargo_oracle_cli]
  - Evidence: `rust_plan_cli_no_cargo_oracle_executes_path_workspace_without_invoking_cargo` passed; receipt has `unit_derivation_graph.ready=true` and `topology_execution.execution_status=success`.
- [x] [serial] Add guard/audit that rejects Cargo invocation in Cargo-free mode. r[rust_package_planning.no_cargo_oracle_cli]
  - Evidence: focused CLI test passes a failing Cargo shim and asserts the shim marker is absent.
- [x] [serial] Extend JSON receipts with Cargo-free mode, compatibility class, blockers, and non-claims. r[rust_package_planning.no_cargo_oracle_cli]
  - Evidence: focused CLI test asserts `cargo_mode.no_cargo_oracle=true`, `compatibility_class=cargo-free-native-path-topology-v1`, and empty mode blockers for the supported path slice.
- [x] [serial] Add CLI tests with Cargo absent or replaced by a failing shim. r[rust_package_planning.no_cargo_oracle_cli]
  - Evidence: `cargo test -p mantle --test rust_plan_cli rust_plan_cli_no_cargo_oracle_executes_path_workspace_without_invoking_cargo -- --nocapture` passed.

## Verification

- [x] [serial] Run CLI tests, failing-Cargo-shim tests, Cargo-free smoke build, and Cairn validation. r[rust_package_planning.no_cargo_oracle_cli]
  - Evidence: `evidence/verification.md` records `cargo test -p mantle --test rust_plan_cli rust_plan_cli_no_cargo_oracle_executes_path_workspace_without_invoking_cargo -- --nocapture`, `cargo test -p mantle --test rust_plan_cli rust_plan_cli_executes_vendored_registry_dependency_in_unified_topology -- --nocapture`, `cargo test -p mantle --bin mantle rust_plan::tests::native_unit_graph -- --nocapture`, and `cairn validate --root .` (`valid: true`).
