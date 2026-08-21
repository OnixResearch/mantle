# Reconciliation Pilot Specification

## Purpose

Defines the `reconciliation-pilot` capability.

## Requirements

### Requirement: Shared reconciliation core is pinned and piloted
r[mantle.reconciliation_pilot.shared_core] Mantle MUST consume the published reconciliation contract through one immutable Radicle revision selected identically by Cargo and Nix, with GC semantics, store mutation, and evidence authority retained by Mantle.

#### Scenario: Shared core is selected
r[mantle.reconciliation_pilot.shared_core.scenario.selected]
- GIVEN a reviewed published revision
- WHEN the composition root selects the shared planner
- THEN Cargo and Nix resolve the same immutable source without an ambient sibling dependency

### Requirement: Reconciliation pilot validation
r[mantle.reconciliation_pilot.validation] The pilot MUST classify a current plan identity as current and any changed plan identity as stale through the shared planner.

#### Scenario: Changed plan is stale
r[mantle.reconciliation_pilot.validation.scenario.stale]
- GIVEN an exact GC plan and a second plan with a changed identity
- WHEN each plan binds through the shared planner against observed facts
- THEN the first classifies as current and the second classifies as stale
