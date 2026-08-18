# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native self target-cfg predicate scope

r[rust_package_planning.native_self_target_cfg_predicate_scope] Mantle MUST evaluate bounded build-relevant target dependency cfg predicates from native manifest facts before normal build topology execution, without consulting Cargo as an executor or resolver fallback.

#### Scenario: Supported nested target-cfg predicates are deterministic

GIVEN a native package manifest with target dependency tables
WHEN the tables use supported atoms (`unix`, `windows`, exact target triple names, `target_os`, `target_arch`, `target_family`, `target_vendor`, `target_env`, `target_abi`, `target_endian`, `target_pointer_width`, `target_has_atomic`) and supported composition operators (`not`, `any`, `all`)
THEN Mantle MUST decide each target dependency table as selected or not-selected from the explicit active target triple
AND selected dependencies MUST enter native dependency facts only through existing path or declared-registry source bindings.

#### Scenario: Unsupported or malformed cfg syntax remains fail-closed

GIVEN a native package manifest with a target dependency table
WHEN the table uses cfg syntax outside the bounded predicate fragment
THEN Mantle MUST emit an `unsupported-target-cfg-surface` blocker before rustc execution
AND MUST NOT silently ask Cargo to decide the dependency surface.
