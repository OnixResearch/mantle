# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_rust_workspace_member_glob_planning]

Mantle MUST represent `native_rust_workspace_member_glob_planning` as bounded native Rust planning evidence and MUST emit deterministic blockers before execution when the surface would require unsupported Cargo behavior.

#### Scenario: Supported bounded surface is planned from native facts

- GIVEN a Rust workspace declares the bounded surface for `native_rust_workspace_member_glob_planning`
- AND all required local or vendored-registry source facts are ready
- WHEN Mantle runs `rust-plan` for the workspace
- THEN Mantle MUST emit native receipt evidence identifying the selected package/member/dependency facts and source material.
- AND Mantle MUST NOT require Cargo orchestration, network access, `$CARGO_HOME`, registry cache fallback, or lockfile mutation for the native claim.

#### Scenario: Supported topology binds selected artifacts

- GIVEN native planning for `native_rust_workspace_member_glob_planning` is ready
- AND the selected package or dependency edge participates in a supported topology rail
- WHEN Mantle executes the topology
- THEN producer artifacts MUST be built before consumers and bound by declared output digest evidence.
- AND the topology receipt MUST expose enough evidence to review the dependency source and artifact binding.

#### Scenario: Unsupported behavior blocks before rustc

- GIVEN a Rust workspace declares a shape for `native_rust_workspace_member_glob_planning` that is outside Mantle's bounded native fragment
- WHEN Mantle plans or executes the topology
- THEN Mantle MUST fail closed with a deterministic blocker identifying the unsupported class before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT silently fall back to Cargo resolver behavior or ambient registry/cache material.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_rust_workspace_member_glob_planning`
- WHEN the relevant `rust_plan` or `rust_plan_cli` tests run
- THEN they MUST include at least one supported fixture and one unsupported fixture with deterministic assertions.
