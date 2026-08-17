## ADDED Requirements

### Requirement: Cargo-free Rust planning CLI

r[rust_package_planning.no_cargo_oracle_cli] Mantle MUST expose an explicit Rust planning and topology execution mode that does not invoke Cargo as planner or build orchestrator.

#### Scenario: no-Cargo mode uses native planner

GIVEN a supported Rust workspace
WHEN the user requests Cargo-free Rust planning and execution
THEN Mantle MUST derive planning facts from native manifest, lockfile, feature, unit graph, and source logic
AND it MUST NOT invoke Cargo for metadata, unit graph, or build orchestration.

#### Scenario: accidental Cargo use fails the run

GIVEN Cargo-free mode is requested
WHEN a code path attempts to invoke Cargo
THEN Mantle MUST fail the run with a deterministic cargo-forbidden blocker or audit failure.

#### Scenario: receipt names compatibility class

GIVEN Cargo-free mode completes or blocks
WHEN Mantle emits JSON receipt evidence
THEN the receipt MUST state the Cargo-free mode, supported compatibility class, blockers, and non-claims.
