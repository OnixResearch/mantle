## ADDED Requirements

### Requirement: Native topology binds artifacts by unit variant identity

r[rust_package_planning.native_unit_variant_artifacts] Native Rust topology execution MUST bind dependency artifacts and rustc dependency search paths using the selected producer unit variant identity, not package ID alone.

#### Scenario: same-package variants do not overwrite direct binding

GIVEN native topology execution has produced two library artifacts for the same package ID from distinct unit variants
AND a later consumer dependency edge selects one of those producer variants
WHEN Mantle prepares the consumer rustc invocation
THEN the consumer `--extern` argument MUST point to the selected producer variant output
AND it MUST NOT be overwritten by another artifact with the same package ID.

#### Scenario: same-crate variants outside selected closure are excluded from rustc search

GIVEN two produced artifacts have the same Rust crate name and package ID but distinct unit variant identities
AND only one variant is in the consumer's selected dependency closure
WHEN Mantle prepares the consumer rustc invocation
THEN its `-L dependency` search paths MUST include the selected variant closure
AND MUST NOT include the unrelated same-crate variant path.

#### Scenario: ambiguous package-only producer fails before rustc

GIVEN a consumer dependency artifact lacks unit variant identity
AND multiple produced candidate artifacts share the same package ID and crate name
WHEN Mantle prepares the consumer rustc invocation
THEN Mantle MUST fail before invoking rustc with a deterministic ambiguous-producer blocker.
