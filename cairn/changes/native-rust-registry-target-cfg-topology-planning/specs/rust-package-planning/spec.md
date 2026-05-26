# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_target_cfg_topology_planning]

Mantle MUST plan bounded native Rust topology graphs containing target-cfg-selected vendored-registry dependency surfaces only from explicit native target-cfg facts, ready native registry source facts, and explicit unit derivation graph evidence, without using Cargo as the build orchestrator, Cargo resolver fallback, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: Supported target-cfg facts select a registry dependency

Given a vendored registry package declares a dependency under a supported `target.'cfg(...)'.dependencies` table
And Mantle can decide that cfg predicate from an explicit active Rust target triple
When Mantle plans the Rust package graph
Then Mantle records deterministic target-cfg selection evidence
And Mantle records the selected registry dependency in native planning evidence
And Mantle requires ready native registry source facts for every selected registry package.

#### Scenario: Unselected target-cfg dependency does not execute

Given a vendored registry package declares a dependency under a supported target-cfg table
And the active Rust target triple does not select that cfg predicate
When Mantle plans and executes the topology
Then the unselected target-cfg registry package does not enter execution
And downstream `rustc` material does not bind an artifact for that unselected dependency.

#### Scenario: Target-cfg-selected registry topology executes from native facts

Given the target-cfg graph is within Mantle's supported bounded cfg surface
And every selected registry package has ready native registry source facts
When Mantle executes the topology
Then selected target-cfg graph units enter execution in producer-first order
And downstream `rustc` material binds selected target-cfg dependency artifacts through explicit topology receipts.

#### Scenario: Unsupported target-cfg behavior blocks before rustc

Given a registry target-cfg surface requires unsupported cfg expressions, target-specific feature activation, ambiguous platform selection, resolver-dependent behavior, or Cargo platform matching beyond Mantle's bounded evaluator
When Mantle plans or executes the topology
Then Mantle reports deterministic target-cfg blockers before executing affected downstream `rustc` commands
And the receipt does not claim fallback to Cargo resolver behavior, Cargo orchestration, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: CLI coverage proves positive and negative target-cfg seams

Given Mantle has positive and negative CLI fixtures for target-cfg vendored registry topologies
When Mantle's CLI test suite runs
Then positive CLI JSON coverage proves explicit target-cfg-selected vendored registry topology planning and execution
And negative CLI JSON coverage proves unsupported target-cfg surfaces block before `rustc`.
