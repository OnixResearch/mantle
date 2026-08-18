## ADDED Requirements

### Requirement: Native dependency cap-lints parity

r[rust_package_planning.native_dependency_cap_lints] Mantle MUST apply Cargo-compatible lint capping to native Rust units that come from non-local dependency sources.

#### Scenario: Registry dependency unit receives cap-lints allow

GIVEN a native Rust unit's source closure entry has kind `registry`
WHEN Mantle derives that unit's rustc arguments
THEN Mantle MUST include deterministic `--cap-lints allow` arguments.
AND Mantle MUST include those arguments for both target and host units.

#### Scenario: Git dependency unit receives cap-lints allow

GIVEN a native Rust unit's source closure entry has kind `git`
WHEN Mantle derives that unit's rustc arguments
THEN Mantle MUST include deterministic `--cap-lints allow` arguments.
AND Mantle MUST NOT require ambient Cargo state to make the decision.

#### Scenario: Local path unit remains uncapped

GIVEN a native Rust unit's source closure entry has kind `path`
WHEN Mantle derives that unit's rustc arguments
THEN Mantle MUST NOT add `--cap-lints allow` for that unit.
AND Mantle MUST keep local path crate lint behavior visible to the build.

#### Scenario: Derive-builder lint frontier moves

GIVEN topology execution currently blocks while compiling `derive_builder_core@0.20.2` because a dependency warning is treated as an error
WHEN native dependency cap-lints parity is applied
THEN self-probe verification MUST show that this lint-cap blocker no longer stops the topology at `derive_builder_core@0.20.2`.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
