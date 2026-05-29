## ADDED Requirements

### Requirement: Cargo-free Rust self-build proof

r[rust_package_planning.cargo_free_self_build_proof] Mantle MUST provide audit-grade evidence for a Cargo-free Rust planning and topology execution proof on a meaningful workspace.

#### Scenario: proof forbids Cargo planning

GIVEN the Cargo-free proof is launched
WHEN Mantle plans and executes the workspace
THEN the proof environment MUST forbid Cargo metadata, unit graph, and build orchestration use
AND any attempted Cargo invocation MUST fail the proof.

#### Scenario: proof emits durable evidence

GIVEN the Cargo-free proof completes or blocks
WHEN the proof exits
THEN it MUST write durable receipts, command streams, source identity, tool identity, output digests, and blocker summaries to an audit bundle.

#### Scenario: proof validates final outputs

GIVEN the Cargo-free proof reports success
WHEN final outputs are inspected
THEN the audit bundle MUST include output artifact digests and an executable smoke check for the produced Mantle or Crunch binary.
