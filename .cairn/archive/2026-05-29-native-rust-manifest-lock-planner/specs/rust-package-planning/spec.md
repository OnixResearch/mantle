## ADDED Requirements

### Requirement: Native Rust manifest and lockfile planning

r[rust_package_planning.native_manifest_lock_planner] Mantle MUST derive Rust workspace, package, dependency, target, and source-lock facts from manifest and lockfile inputs without invoking Cargo.

#### Scenario: workspace package facts are parsed natively

GIVEN a Rust workspace with root and member manifests
WHEN Mantle runs native manifest planning
THEN it MUST emit normalized workspace members, package identities, package metadata, target definitions, dependency tables, and edition/version inheritance facts without calling Cargo.

#### Scenario: lockfile facts are parsed natively

GIVEN a `Cargo.lock` with registry, git, and path package records
WHEN Mantle runs native lockfile planning
THEN it MUST emit source identities, versions, checksums or revisions, and dependency lock edges without calling Cargo.

#### Scenario: unsupported manifest surface fails closed

GIVEN a manifest or lockfile uses Cargo behavior outside Mantle's supported parser subset
WHEN Mantle plans the package
THEN it MUST emit a deterministic unsupported-manifest blocker
AND it MUST NOT silently drop that manifest material.
