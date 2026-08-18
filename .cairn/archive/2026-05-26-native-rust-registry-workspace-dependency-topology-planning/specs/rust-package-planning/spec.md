# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_workspace_dependency_topology_planning]

Mantle MUST plan bounded native Rust topology graphs containing workspace-inherited vendored-registry dependency surfaces only from explicit root workspace dependency facts, member inheritance facts, ready native registry source facts, and explicit unit derivation graph evidence, without using Cargo as the build orchestrator, Cargo resolver fallback, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: Workspace dependency facts select an inherited registry dependency

Given a workspace root declares a supported vendored-registry dependency in `[workspace.dependencies]`
And a workspace member declares that dependency with `{ workspace = true }`
And the inherited dependency is within Mantle's bounded supported fragment
When Mantle plans the Rust package graph
Then Mantle records deterministic workspace dependency inheritance evidence
And Mantle records the inherited registry dependency in native planning evidence
And Mantle requires ready native registry source facts for every inherited registry package.

#### Scenario: Workspace-inherited registry topology executes from native facts

Given the workspace dependency inheritance graph is within Mantle's supported bounded surface
And every inherited registry package has ready native registry source facts
When Mantle executes the topology
Then inherited workspace dependency graph units enter execution in producer-first order
And downstream `rustc` material binds inherited dependency artifacts through explicit topology receipts.

#### Scenario: Unsupported workspace dependency inheritance blocks before rustc

Given a workspace dependency surface requires missing root entries, unsupported inherited feature/default-feature behavior, target-specific inheritance beyond Mantle's bounded evaluator, ambiguous package renames, resolver-dependent behavior, or Cargo workspace matching beyond Mantle's bounded evaluator
When Mantle plans or executes the topology
Then Mantle reports deterministic workspace dependency blockers before executing affected downstream `rustc` commands
And the receipt does not claim fallback to Cargo resolver behavior, Cargo orchestration, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: CLI coverage proves positive and negative workspace dependency seams

Given Mantle has positive and negative CLI fixtures for workspace-inherited vendored registry topologies
When Mantle's CLI test suite runs
Then positive CLI JSON coverage proves explicit workspace-inherited vendored registry topology planning and execution
And negative CLI JSON coverage proves unsupported workspace dependency inheritance surfaces block before `rustc`.
