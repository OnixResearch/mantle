# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_patch_source_topology_execution]

Mantle MUST execute bounded local patch-source registry topologies from explicit native planning and execution receipt material, and MUST fail closed before `rustc` when execution would require unsupported Cargo resolver, cache, registry, or network behavior.

#### Scenario: Supported local patch-source topology executes from native facts

- GIVEN native planning for `native_registry_patch_source_planning` is ready
- AND a package declares a supported local `[patch.crates-io]` replacement for a dependency
- WHEN Mantle executes the patch-source topology
- THEN Mantle MUST execute the patched local source producer before the consumer using explicit derivation args and env.
- AND Mantle MUST bind produced artifacts into declared dependency/input/`--extern` surfaces by digest evidence.
- AND Mantle MUST NOT invoke Cargo as planner, resolver, executor, cache provider, or topology repair mechanism.

#### Scenario: Execution receipt preserves bounded patch-source evidence

- GIVEN a supported patch-source topology executes
- WHEN Mantle records the execution receipt
- THEN the receipt MUST identify the consumer package, patch source package ids, ordered unit execution receipts, artifact digests, toolchain identity, rustc argument digests, bounded claim, blocker when present, and stable receipt hash.
- AND the receipt MUST NOT claim full Cargo resolver compatibility, version solving, non-crates.io patch registry support, git patch support, network/index access, ambient registry cache fallback, or generalized patch scheduling.

#### Scenario: Unsupported or stale patch-source surfaces block before rustc

- GIVEN a patch-source topology requires unsupported patch registry/source behavior, missing/stale patch material, missing source facts, missing unit derivations, or resolver behavior outside the bounded fragment
- WHEN Mantle evaluates the topology for execution
- THEN Mantle MUST emit a deterministic blocker before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT fall back to Cargo, `$CARGO_HOME`, registry caches, target directories, or network access.

#### Scenario: Explicit CLI receipt is separate from general topology execution

- GIVEN a package has patch-source topology evidence
- WHEN Mantle executes normal `--execute-topology`
- THEN Mantle MUST NOT silently emit the dedicated `native_registry_patch_source_topology_execution` receipt.
- AND the dedicated receipt MUST only be emitted by the explicit patch-source topology execution path.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_registry_patch_source_topology_execution`
- WHEN the relevant `rust_plan_cli` tests run
- THEN they MUST include at least one supported execution fixture and one unsupported fixture with deterministic assertions.
