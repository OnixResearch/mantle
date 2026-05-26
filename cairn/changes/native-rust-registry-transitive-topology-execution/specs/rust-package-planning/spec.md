# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_transitive_topology_execution]

Mantle MUST execute bounded native Rust topology graphs containing transitive vendored-registry dependency chains only from ready native registry source facts and explicit unit derivation graph evidence, without using Cargo as the build orchestrator, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: Ready transitive registry source facts gate execution

Given a local Rust package depends on vendored registry package A
And vendored registry package A depends on vendored registry package B
And the lockfile and declared vendor source replacement provide supported source facts for both registry packages
When Mantle plans and executes the topology
Then `native_registry_source_planning.ready` is true
And the receipt records ready source facts for registry package A and registry package B
And execution uses those ready source facts rather than ambient Cargo cache material.

#### Scenario: Producer-first transitive topology execution

Given a supported topology `local app -> registry package A -> registry package B`
When Mantle executes the topology
Then registry package B is executed before registry package A
And registry package A is executed before the local app
And each execution receipt records deterministic source, rustc argument, output artifact, and receipt hash evidence.

#### Scenario: Transitive registry dependency artifacts are bound into consumers

Given registry package B produces a library artifact consumed by registry package A
And registry package A produces a library artifact consumed by the local app
When Mantle executes the topology
Then registry package A's execution receipt records dependency artifact digest evidence for registry package B
And the local app's execution receipt records dependency artifact digest evidence for registry package A
And downstream `rustc` material is rewritten only from explicit producer artifacts.

#### Scenario: Missing or unsupported transitive vendor material blocks before rustc

Given a registry package in the transitive closure has missing, stale, or unsupported vendored source material
When Mantle is asked to execute the topology
Then Mantle reports deterministic native-registry blockers before executing affected downstream `rustc` commands
And the receipt does not claim fallback to Cargo orchestration, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.
