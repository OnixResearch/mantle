## ADDED Requirements

### Requirement: Native topology preserves real Cargo unit producer identity

r[rust_package_planning.native_real_unit_identity] Native Rust topology execution MUST preserve selected Cargo producer unit identity for dependency and host artifact binding.

#### Scenario: dependency edges use selected Cargo unit identity

GIVEN Cargo unit graph dependency material identifies a producer by unit index
WHEN Mantle lowers that dependency into native Rust dependency artifacts
THEN the dependency artifact MUST carry the selected producer unit ID derived from that exact Cargo unit entry
AND it MUST NOT substitute a package/crate/kind/mode pseudo-variant key.

#### Scenario: duplicate selected units remain distinct

GIVEN Cargo selects two supported target units for the same package ID, crate name, target kind, and mode
AND the units differ by selected Cargo identity or dependency facts
WHEN Mantle plans the native target unit graph
THEN both selected units MUST remain distinct native units
AND exact duplicate normalization MUST NOT collapse different unit IDs.

#### Scenario: package-only fallback filters unrelated same-package outputs

GIVEN a legacy dependency artifact lacks selected producer unit identity
AND a produced artifact index contains outputs for the same package ID with different crate names
WHEN Mantle selects fallback candidates for binding or search paths
THEN candidate matching MUST include the dependency crate name
AND Mantle MUST only raise ambiguous-producer blockers for multiple matching package-and-crate candidates.
