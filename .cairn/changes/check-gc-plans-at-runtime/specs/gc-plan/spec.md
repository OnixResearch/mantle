# Specification: GC plan runtime check

## ADDED Requirements

### Requirement: Runtime check before deletion

r[mantle.gc_plan.runtime_check] Every core GC plan that drives deletion MUST be
accepted by the imported Terrane kernel's `check_plan`, through a projection
that refuses duplicate entries and unknown or repeated planned paths and
requires the deletion intents to name exactly the candidates. The projection
from path identifiers to indices MUST itself be validated by the kernel's
verified `project_keys` and `key_mask`, not trusted. A refused plan
MUST fail before any removal or fence write.

#### Scenario: Planner drops a reference

- GIVEN a `plan_gc` defect that leaves a path reachable from a root out of the
  retained set
- WHEN the store plans GC
- THEN planning MUST fail with `gc-plan-check-failed`
- AND no path MUST be removed.

#### Scenario: Projection names the wrong entry

- GIVEN a computed reference or root index that names an entry whose path
  differs from the referenced path, or a path that names no entry
- WHEN the store plans GC
- THEN the verified projection MUST be refused and no path removed.

#### Scenario: Planner refuses the request

- GIVEN a request that `plan_gc` refuses
- WHEN the store plans GC
- THEN the planner's existing error MUST be reported.

## MODIFIED Requirements

### Requirement: Exact kernel import

r[mantle.gc_plan.kernel_import] Mantle MUST import the Terrane reclaim-plan
kernel and its tests unchanged from a pinned revision, MUST check their exact
BLAKE3 identities, and MUST compile the kernel in production only within
`crunch-gc-check`.

#### Scenario: Vendored kernel drifts

- GIVEN a vendored kernel file whose bytes differ from the recorded identity
- WHEN the importer check runs
- THEN it MUST fail and name the drifted member.
