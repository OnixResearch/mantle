# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_workspace_dependency_topology_execution]

Mantle MUST execute bounded workspace-inherited vendored-registry dependency topologies from explicit native planning and execution receipt material, and MUST fail closed before `rustc` when execution would require unsupported Cargo resolver, cache, or network behavior.

#### Scenario: Supported workspace-inherited registry topology executes from native facts

- GIVEN native planning for `native_registry_workspace_dependency_topology_planning` is ready
- AND a workspace root declares a supported vendored-registry dependency in `[workspace.dependencies]`
- AND a workspace member declares that dependency with `{ workspace = true }`
- WHEN Mantle executes the workspace-dependency topology
- THEN Mantle MUST execute inherited registry producer units before the workspace member consumer using explicit derivation args and env.
- AND Mantle MUST bind produced artifacts into declared dependency/input/`--extern` surfaces by digest evidence.
- AND Mantle MUST NOT invoke Cargo as planner, resolver, executor, cache provider, or topology repair mechanism.

#### Scenario: Execution receipt preserves bounded workspace-dependency evidence

- GIVEN a supported workspace-dependency topology executes
- WHEN Mantle records the execution receipt
- THEN the receipt MUST identify the workspace root, member package, inherited dependency package ids, ordered unit execution receipts, artifact digests, toolchain identity, rustc argument digests, bounded claim, blocker when present, and stable receipt hash.
- AND the receipt MUST NOT claim full Cargo resolver compatibility, version solving, network/index access, ambient registry cache fallback, or generalized workspace scheduling.

#### Scenario: Unsupported or stale workspace-dependency surfaces block before rustc

- GIVEN a workspace-dependency topology requires unsupported member-side features/default-feature/platform behavior, missing/stale vendor material, missing source facts, missing unit derivations, or resolver behavior outside the bounded fragment
- WHEN Mantle evaluates the topology for execution
- THEN Mantle MUST emit a deterministic blocker before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT fall back to Cargo, `$CARGO_HOME`, registry caches, target directories, or network access.

#### Scenario: Explicit CLI receipt is separate from general topology execution

- GIVEN a package has workspace-dependency topology evidence
- WHEN Mantle executes normal `--execute-topology`
- THEN Mantle MUST NOT silently emit the dedicated `native_registry_workspace_dependency_topology_execution` receipt.
- AND the dedicated receipt MUST only be emitted by the explicit workspace-dependency topology execution path.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_registry_workspace_dependency_topology_execution`
- WHEN the relevant `rust_plan_cli` tests run
- THEN they MUST include at least one supported execution fixture and one unsupported fixture with deterministic assertions.
