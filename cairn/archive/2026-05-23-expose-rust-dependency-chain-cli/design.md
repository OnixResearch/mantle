## Design

### CLI shape

Add `--execute-first-dependency-chain` to `mantle rust-plan`. The flag is mutually exclusive with `--execute-first-supported-unit` and requires `--execution-output-root`, matching the existing artifact-root contract.

CLI flow:

1. Capture the normal `RustPlanReceipt` through Cargo oracle metadata/unit-graph evidence.
2. If `--execute-first-supported-unit` is set, keep the existing single-unit path unchanged.
3. If `--execute-first-dependency-chain` is set, call `execute_first_rust_unit_dependency_chain` with the captured `unit_derivation_graph` and explicit `rustc`/output-root options.
4. Print a combined receipt containing the plan receipt plus the chain receipt.

### Receipt shape

Add a small wrapper receipt rather than changing the chain receipt schema:

- `rust_plan`: the normal captured Rust plan receipt.
- `dependency_chain_execution`: the existing `RustUnitDependencyChainExecutionReceipt`.

This preserves the previously landed chain receipt identity and gives the CLI an operator-facing top-level envelope analogous to the existing `RustPlanExecutionReceipt`.

### Boundaries

This change does not add a scheduler, multi-edge planning, host/proc-macro/build-script execution, binary native-link probing, or Cargo compatibility claims. Unsupported or incomplete chains continue to return deterministic blockers from the existing chain executor.
