## ADDED Requirements

### Requirement: Distributed Scheduler Wiring Behind Local Defaults [r[distributed-scheduler-wiring]]
Crunch MUST wire distributed build interface seams into scheduler/orchestration only behind explicit local-only-preserving defaults until a provider implementation is separately accepted.

#### Scenario: Default behavior remains local [r[distributed-scheduler-wiring.1]]
- GIVEN no distributed resolver, publisher, or remote realizer is configured
- WHEN a build is scheduled
- THEN the existing local build path and goal dedup semantics are preserved

#### Scenario: Opt-in resolver decision is visible [r[distributed-scheduler-wiring.2]]
- GIVEN an explicit resolver profile is configured
- WHEN a realization key is available before local dispatch
- THEN resolver hit, miss, fallback, or rejection is represented in distributed diagnostics
