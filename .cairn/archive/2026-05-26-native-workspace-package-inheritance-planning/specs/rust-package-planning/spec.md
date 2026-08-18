# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_workspace_package_inheritance_planning]

Mantle MUST model bounded Cargo workspace package-version inheritance in native package/target planning, and MUST fail closed when inherited package identity cannot be resolved from explicit workspace manifest facts.

#### Scenario: Native package planning resolves workspace package version inheritance

- GIVEN a workspace root declares `[workspace.package].version`
- AND a package manifest declares `version.workspace = true`
- WHEN Mantle computes native package/target planning facts
- THEN Mantle MUST resolve the package version from the workspace root manifest.
- AND Mantle MUST use the resolved version for native package identity, oracle comparison, target facts, and downstream unit graph material.

#### Scenario: Unsupported workspace package inheritance blocks deterministically

- GIVEN a package manifest requests unsupported package version inheritance material or the workspace root lacks `[workspace.package].version`
- WHEN Mantle computes native package/target planning facts
- THEN Mantle MUST emit a deterministic native package planning blocker.
- AND Mantle MUST NOT fall back to Cargo-derived package identity as the native fact.

#### Scenario: CLI coverage proves inherited version facts

- GIVEN the implementation claims support for `native_workspace_package_inheritance_planning`
- WHEN the relevant `rust_plan_cli` tests run
- THEN at least one fixture MUST prove `version.workspace = true` is resolved into native package facts.
