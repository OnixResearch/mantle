## MODIFIED Requirements

### Requirement: Parallel multi-root forcing uses only bounded safe concurrency

The system MUST separate multi-root forcing semantics from the concrete local
execution backend used to run root-force assignments.

The `crunch-eval` core MUST provide one backend-neutral bounded multi-root
forcing interface whose semantics stay the same across inline and any shipped
non-inline backends.

The `crunch-eval` core MUST provide a serial inline backend that can execute a
bounded multi-root request without background worker threads, subprocesses,
daemon lifecycle, or binary self-spawn.

Any shipped threaded or subprocess backend MUST remain optional and MUST
preserve the same typed derivation values, requested-label ordering, and
labeled failure semantics as the serial inline backend for the same requested
roots.

Any backend that evaluates more than one root concurrently MUST still use
bounded concurrency and MUST drive only immutable worker input across the
concurrency boundary unless stronger compile-time safety evidence exists for the
actual concurrently used evaluator state.

#### Scenario: Portable inline backend forces multiple roots without host worker helpers

- GIVEN a multi-root forcing request on a host or embedding target that does
  not want background worker threads or subprocesses
- WHEN `crunch-eval` materializes that request through its required inline
  backend
- THEN it returns the requested typed derivation values
- AND it preserves the requested label order
- AND it does not require a daemon, global worker pool, or binary self-spawn

#### Scenario: Optional non-inline backend matches inline semantics

- GIVEN the same source input and requested root labels
- WHEN `crunch-eval` forces them through the serial inline backend and through
  any shipped threaded or subprocess backend
- THEN both paths produce the same typed derivation values for each label
- AND both paths preserve the same requested-label ordering
- AND a failure in either path identifies the same failed label
