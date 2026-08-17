## ADDED Requirements

### Requirement: Source-root host/target Rust topology

r[rust_package_planning.source_root_host_target_topology] Mantle MUST plan and execute Cargo-free Rust topology units with explicit host, target, and host-dependency roles when a source-root or receipt-bound toolchain closure is selected.

#### Scenario: host-dependency units use the host role

GIVEN a Cargo-free topology contains a host custom-build unit whose support crates are also present in the target dependency graph
WHEN Mantle derives native unit and artifact facts
THEN those support crates MUST be represented as host-dependency units for the host execution triple before the custom-build unit consumes them.
AND Mantle MUST NOT satisfy the host custom-build unit from a target-built support artifact.

#### Scenario: target units remain target scoped

GIVEN the same package graph contains target libraries or binaries
WHEN Mantle derives their dependency artifacts
THEN target units MUST keep the requested target triple and target toolchain-closure policy.
AND Mantle MUST NOT rewrite target-library dependencies to host-dependency artifacts unless the dependency edge is a host execution edge.

#### Scenario: artifact identity is role and triple sensitive

GIVEN a package target can appear in more than one role or triple
WHEN Mantle records produced and consumed artifacts
THEN artifact identity MUST include package identity, target identity, role, selected triple, source digest, feature set, metadata hash, and toolchain-policy digest.
AND same-package host and target artifacts MUST NOT collide in receipt maps, output paths, or dependency lookup.

#### Scenario: mismatched artifacts fail before rustc

GIVEN a unit consumes an artifact whose role, triple, source digest, metadata hash, or toolchain-policy digest does not match the planned dependency edge
WHEN Mantle prepares execution for that unit
THEN Mantle MUST fail before invoking `rustc` with a deterministic artifact-mismatch blocker.
AND the blocker MUST identify the expected and observed role and triple without searching Cargo target directories or ambient caches.

#### Scenario: receipts expose the split

GIVEN role-aware topology execution completes or blocks
WHEN Mantle writes topology receipts or blocker summaries
THEN the evidence MUST record each unit's role, selected triple, toolchain-policy digest, produced artifact digest when present, and consumed artifact roles.
AND any Cargo-free or source-root proof claim MUST be bounded to the recorded role-aware evidence.
