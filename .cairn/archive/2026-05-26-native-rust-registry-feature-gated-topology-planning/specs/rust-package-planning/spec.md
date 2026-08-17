# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_feature_gated_topology_planning]

Mantle MUST plan bounded native Rust topology graphs containing feature-selected vendored-registry dependency surfaces only from explicit native feature facts, ready native registry source facts, and explicit unit derivation graph evidence, without using Cargo as the build orchestrator, Cargo resolver fallback, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: Explicit selected feature facts activate a bounded optional registry dependency

Given a local Rust package selects a supported feature on a vendored registry package
And that selected feature activates a supported optional vendored registry dependency
When Mantle plans the Rust package graph
Then Mantle records deterministic selected feature facts
And Mantle records the activated optional registry dependency in native planning evidence
And Mantle requires ready native registry source facts for every activated registry package.

#### Scenario: Feature-selected registry topology executes from native facts

Given the selected feature graph is within Mantle's supported bounded feature surface
And every activated registry package has ready native registry source facts
When Mantle executes the topology
Then only selected feature graph units enter execution
And unselected optional registry packages do not execute
And downstream `rustc` material binds selected optional dependency artifacts through explicit topology receipts.

#### Scenario: Unsupported feature behavior blocks before rustc

Given a registry feature surface requires unsupported default-feature behavior, workspace feature inheritance, target-specific feature activation, ambiguous feature names, or resolver-dependent feature unification
When Mantle plans or executes the topology
Then Mantle reports deterministic feature-surface blockers before executing affected downstream `rustc` commands
And the receipt does not claim fallback to Cargo resolver behavior, Cargo orchestration, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: CLI coverage proves positive and negative feature seams

Given Mantle has positive and negative CLI fixtures for feature-gated vendored registry topologies
When Mantle's CLI test suite runs
Then positive CLI JSON coverage proves explicit feature-selected vendored registry topology planning and execution
And negative CLI JSON coverage proves unsupported feature surfaces block before `rustc`.
