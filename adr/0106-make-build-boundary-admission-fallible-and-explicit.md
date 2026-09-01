# ADR 0106: Make Build Boundary Admission Fallible and Explicit

**Status:** Accepted

## Context

The strict Tiger Style gate found panic paths and positional interfaces in
content-addressed output planning, build-request construction, execution-profile
registration, and worker failure propagation.

The old content-addressed text helpers returned plain values and panicked when a
nominal constructor rejected input. The execution-profile registry method also
accepted six positional values. These shapes hid admission and authority facts
at important build boundaries.

## Decision

Return typed `Result` values from text-based content-addressed output planning.
Use `CaOutputPathInput` and `ExecutionProfileRegistration` to name semantic
roles. Update repository callers and tests in the same change.

Keep the typed content-addressed core pure. Keep registry mutation, scheduler
state changes, filesystem publication, and error reporting in their existing
shell owners and order.

Remove redundant recursive structured-attribute canonicalization. The admitted
JSON size remains bounded, the active `serde_json::Map` backend remains
key-sorted, arrays retain declared order, and nested byte fixtures guard the
accepted serialization.

Do not add Tiger allowances, warning budgets, finding baselines, or narrower
check scope.

## Consequences

- Invalid content-addressed names now return typed build errors instead of
  aborting the process.
- Registry and worker call sites expose semantic fields rather than positional
  argument clusters.
- Valid content-addressed paths, marker bytes, structured attributes, scheduler
  outcomes, and publication behavior remain unchanged.
- Source callers must construct the new named inputs and handle fallible CA
  planning.
- Later pipeline findings remain a separate blocker.
