## ADDED Requirements

### Requirement: Cargo-free Rust self-build proof

r[rust_package_planning.cargo_free_self_build_proof] Mantle MUST provide audit-grade evidence for a bounded Cargo-free self-build topology proof that plans and executes the checked-out Mantle workspace without invoking Cargo as planner or build orchestrator.

#### Scenario: self-build proof forbids Cargo

GIVEN the Cargo-free self-build proof is launched
WHEN Mantle plans and executes the checked-out workspace
THEN the proof environment MUST replace Cargo with a failing guard
AND any attempted Cargo metadata, unit-graph, or build-orchestration invocation MUST fail the proof.

#### Scenario: self-build proof binds local source closure

GIVEN the workspace has path, declared vendored registry, and captured git source inputs
WHEN Cargo-free mode derives source and package facts
THEN it MUST derive source identities from checked-in manifests, `Cargo.lock`, declared vendor roots, and captured local source material
AND it MUST NOT use network fetches, ambient Cargo caches, or Cargo metadata output.

#### Scenario: self-build proof emits durable evidence

GIVEN the Cargo-free self-build proof completes or blocks
WHEN the proof exits
THEN it MUST write durable receipts, command streams, source identity, tool identity, output digests, Cargo guard status, blocker summaries, and non-claims to an audit bundle.

#### Scenario: self-build proof validates final CLI output

GIVEN the Cargo-free self-build proof reports success
WHEN final outputs are inspected
THEN the audit bundle MUST include a smoke check of the produced Mantle CLI binary
AND the audit bundle MUST state that the proof is not Crunch fixed-point self-hosting, source-built bootstrap, or release reproducibility evidence.
