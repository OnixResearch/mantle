## ADDED Requirements

### Requirement: Native topology preserves duplicate host unit identity

r[rust_package_planning.native_host_real_unit_identity] Native Rust host topology MUST preserve every selected Cargo host unit identity when planning proc-macro and custom-build artifacts.

#### Scenario: duplicate proc-macro units remain distinct

GIVEN Cargo selects two proc-macro host units with the same package ID, target name, and target kind
WHEN Mantle plans native host artifacts
THEN Mantle MUST keep both selected unit IDs distinct
AND it MUST NOT overwrite one with the other through package/name/kind indexing.

#### Scenario: duplicate custom-build units remain distinct

GIVEN Cargo selects two custom-build host units with the same package ID, target name, and target kind
WHEN Mantle plans native host artifacts and build-script metadata producers
THEN Mantle MUST keep both selected unit IDs distinct
AND consumers MUST bind to the selected producer identity or fail closed on ambiguity.

#### Scenario: host fallback ambiguity blocks before rustc

GIVEN host artifact lookup lacks exact selected producer identity
AND multiple host candidates match the same package ID, target name, and target kind
WHEN Mantle prepares a consuming rustc invocation
THEN Mantle MUST fail before invoking rustc with a deterministic ambiguous-host-producer blocker.
