## ADDED Requirements

### Requirement: Native topology emits deterministic crate disambiguators

r[rust_package_planning.native_crate_disambiguators] Native Rust topology execution MUST pass deterministic rustc crate metadata disambiguators for every native rustc unit so same-name crate versions in one dependency graph do not collide.

#### Scenario: same crate name different package IDs

GIVEN two native units share the same Rust crate name
AND their package IDs or source digests differ
WHEN Mantle creates reviewable rustc derivations
THEN each derivation MUST include `-C metadata=<hash>`
AND the metadata values MUST differ.

#### Scenario: stable unit identity

GIVEN the same native unit identity and selected features
WHEN Mantle creates reviewable rustc derivations repeatedly
THEN the `-C metadata=<hash>` value MUST be stable
AND the value MUST be derived from reviewable unit facts, not from ambient Cargo target directories.
