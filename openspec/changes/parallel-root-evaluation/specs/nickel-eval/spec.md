## MODIFIED Requirements

### Requirement: Nickel evaluation via eval_full_for_export

The system MUST provide a stateful lazy session boundary that retains Nickel
program state across at least two operations:

1. a lazy root-discovery path that evaluates the top-level Nickel result only
   enough to determine whether it is a single derivation, an array of
   derivations, or a record of derivations,
2. a selected-root forcing path that fully evaluates only the requested root
   value for typed extraction or export, and
3. a bounded multi-root forcing path for callers that need more than one root
   without serially deep-exporting every root up front.

Build and planning execution MUST use the lazy discovery path before any
per-root forcing. Whole-program export-ready deep evaluation MAY still be used
for operator-visible `crunch eval` output or other callers that explicitly need
whole-program export semantics.

The multi-root forcing path MUST preserve the caller's requested label order in
its returned results.

If the caller omits an explicit concurrency cap, the multi-root forcing path
MUST default to a cap of `1`.

For any given requested label, the multi-root forcing path MUST return the same
typed derivation value as the serial same-session forcing path for that label.

If one requested root fails during forcing, the surfaced `crunch-eval` error
MUST identify the failed label. The batch API MUST fail the whole request on
that first labeled error rather than returning a partial success vector.

#### Scenario: Batch forcing preserves requested root order

- GIVEN a `.ncl` file exporting a record of named derivations
- WHEN crunch-eval forces a selected batch of root labels through the multi-root
  lazy session API
- THEN the returned typed derivations preserve the caller's requested label
  order
- AND each returned derivation remains associated with the same label that was
  requested

#### Scenario: Failed root reports its label during batch forcing

- GIVEN a `.ncl` file exporting named derivations
- AND one requested root fails during forcing
- WHEN crunch-eval runs the bounded multi-root forcing path
- THEN it returns a `crunch-eval` error
- AND the error identifies the failed root label
- AND it does not return a partial success vector for the same batch request

## ADDED Requirements

### Requirement: Parallel multi-root forcing uses only bounded safe concurrency

The system MUST bound concurrent root-force jobs for the multi-root lazy
session path.

Cross-thread safety for any evaluator state that is driven concurrently MUST be
proven by compile-time trait-bound evidence on the actual concurrently used
state. Without that proof, the implementation MUST use isolated worker
 evaluation states derived from the same source text and import-path set rather
than sharing mutable evaluator state across threads.

#### Scenario: Multi-root forcing honors a concurrency cap

- GIVEN a multi-root forcing request with more requested labels than the chosen
  eval parallelism cap
- WHEN crunch-eval materializes that batch
- THEN it runs at most the bounded number of root-force jobs at once
- AND the remaining labels wait until one in-flight job finishes

#### Scenario: Isolated-worker forcing matches serial same-session results

- GIVEN a `.ncl` file exporting named derivations
- WHEN crunch-eval forces one label through the serial same-session path and
  the same label through the multi-root worker path
- THEN both paths produce the same typed derivation value
- AND the worker path does not change label semantics for that root
