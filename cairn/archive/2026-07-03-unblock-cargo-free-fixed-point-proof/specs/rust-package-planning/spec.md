## ADDED Requirements

### Requirement: Cargo-free fixed-point blocker resolution

r[rust_package_planning.cargo_free_fixed_point_blocker_resolution] Mantle MUST convert Cargo-free fixed-point topology blocked statuses into deterministic, actionable blocker diagnostics and resolve implementation-owned blockers before claiming proof success.

#### Scenario: blocked receipt is classified

GIVEN a Cargo-free fixed-point run emits a topology execution status of blocked
WHEN Mantle records the proof receipt or evidence summary
THEN the summary MUST identify the root blocked unit, package identity, execution role, selected triple, predecessor status, and blocker class when those fields are present in the receipt.
AND missing classifier inputs MUST produce a deterministic diagnostic instead of a generic success or opaque blocked claim.

#### Scenario: implementation-owned blocker advances

GIVEN the classifier identifies a blocker caused by Mantle's native Rust planner or topology executor
WHEN the change is validated
THEN Mantle MUST either fix that blocker and show the next proof frontier advanced, or record a narrower evidence-backed reason why the blocker is not implementation-owned.
AND it MUST NOT mark the fixed-point proof complete from synthetic fixtures alone.

#### Scenario: external blocker remains bounded

GIVEN a proof run is blocked by source-root toolchain material, host kernel capability, disk capacity, or another external prerequisite
WHEN the evidence is reported
THEN Mantle MUST name the external blocker, command, receipt path or bundle, and next action.
AND the report MUST remain narrower than a Nix-free fixed-point success claim.

#### Scenario: malformed blocker evidence fails closed

GIVEN a blocked topology receipt is malformed, truncated, or missing required identity fields
WHEN Mantle classifies the blocker
THEN classification MUST fail with a deterministic diagnostic.
AND it MUST NOT fabricate unit identity, role, triple, or proof-success evidence.
