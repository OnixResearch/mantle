## ADDED Requirements

### Requirement: External batch ports and errors have explicit owners

r[external_batch_dispatchers.port_error_ownership] The Mantle application layer MUST own the external batch capability contract. Direct-process and Slurm adapters MUST implement it with typed infrastructure failures and bounded Mantle observations.

#### Scenario: Adapter contract returns one raw failure class

r[external_batch_dispatchers.port_error_ownership.scenario.raw_failure]
- GIVEN serialization, process launch, timeout, cancellation, malformed output, stale response, and provider denial can fail differently
- WHEN the adapter reports failure
- THEN it MUST preserve the typed infrastructure class and safe bounded details
- AND the shell MUST translate that class without fabricating domain rejection or terminal success.

#### Scenario: Adapter decides lifecycle meaning

r[external_batch_dispatchers.port_error_ownership.scenario.policy]
- GIVEN a direct-process or Slurm adapter returns a bounded provider observation
- WHEN lifecycle admission runs
- THEN a deterministic Mantle decision MUST classify freshness, transition, ambiguity, and next effects
- AND the adapter MUST NOT mutate canonical coordinator meaning by itself.

### Requirement: External batch boundary tests include failure paths

r[external_batch_dispatchers.boundary_validation] External batch validation MUST pair successful direct-process and Slurm cases with malformed, stale, unavailable, timeout, cancellation, process-failure, unknown, and exceeded-bound cases.

#### Scenario: Adapter success hides missing error translation

r[external_batch_dispatchers.boundary_validation.scenario.translation]
- GIVEN valid fixture dispatch succeeds but one required infrastructure failure lacks a typed translation test
- WHEN external batch readiness runs
- THEN readiness MUST fail
- AND fixture success MUST NOT prove lifecycle, cancellation, or reconciliation behavior.
