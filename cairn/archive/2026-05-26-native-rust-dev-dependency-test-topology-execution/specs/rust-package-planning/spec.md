# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_rust_dev_dependency_test_topology_execution]

Mantle MUST execute bounded dev-dependency test topologies from explicit native planning and execution receipt material, and MUST fail closed before `rustc` when execution would require unsupported Cargo test behavior.

#### Scenario: Supported dev-dependency test topology executes from native facts

- GIVEN native planning for `native_rust_dev_dependency_test_topology_planning` is ready
- AND the workspace declares a bounded test target that consumes a vendored-registry dev-dependency with ready source facts
- WHEN Mantle executes the dev-dependency test topology
- THEN Mantle MUST execute required producer units before the test consumer using explicit derivation args and env.
- AND Mantle MUST bind produced artifacts into declared dependency/input/`--extern` surfaces by digest evidence.
- AND Mantle MUST NOT invoke Cargo as planner, test runner, executor, cache provider, or topology repair mechanism.

#### Scenario: Execution receipt preserves bounded test evidence

- GIVEN a supported dev-dependency test topology executes
- WHEN Mantle records the execution receipt
- THEN the receipt MUST identify the package/member, test target, selected dev-dependency, source closure digest, ordered unit execution receipts, artifact digests, toolchain identity, rustc argument digest, bounded claim, and stable receipt hash.
- AND the receipt MUST NOT claim full Cargo test compatibility, doctest/run/bench support, hidden harness behavior, or feature resolver parity beyond the bounded fixture.

#### Scenario: Unsupported test surfaces block before rustc

- GIVEN a dev-dependency test topology requires unsupported Cargo behavior such as doctest/run/bench modes, hidden harness discovery, missing/stale vendor material, missing test-unit derivations, or unsupported resolver behavior
- WHEN Mantle evaluates the topology for execution
- THEN Mantle MUST emit a deterministic blocker before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT fall back to Cargo, `$CARGO_HOME`, registry caches, target directories, or network access.

#### Scenario: Normal build topology does not consume dev-dependency material

- GIVEN a package has dev-dependency test topology evidence
- WHEN Mantle executes normal build topology rather than the bounded test topology
- THEN Mantle MUST NOT silently include dev-dependency artifacts in the normal build claim.
- AND any normal-build shape that would require dev-dependency material MUST emit deterministic blocker evidence.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_rust_dev_dependency_test_topology_execution`
- WHEN the relevant `rust_plan` or `rust_plan_cli` tests run
- THEN they MUST include at least one supported execution fixture and one unsupported fixture with deterministic assertions.
