# Native Package Contract Parity Specification

## ADDED Requirements

### Requirement: Fixtures reach their intended admission boundary

r[native_package_parity.fixtures] Mantle MUST keep accepted fixtures semantically valid and isolate each negative fixture's intended rejection condition.
Repairs MUST preserve deterministic rejection categories, absence of protected effects, and the frontend-neutral build-tool boundary.

#### Scenario: Valid import fixture reaches graph admission

- GIVEN a derivation has consistent output/environment paths and valid syntax
- WHEN the import adapter admits its reachable graph
- THEN the fixture MUST reach the intended graph boundary
- AND unrelated invalid fixtures MUST NOT conceal whether missing reachable inputs are rejected.

#### Scenario: Raw module inventory is supplied

- GIVEN raw module inventory is not a concrete derivation
- WHEN Mantle plans it as a build input
- THEN it MUST reject without build actions or store output
- AND its diagnostic assertion MUST check the current rejection rather than require obsolete prose.

### Requirement: Inventories preserve complete bounded coverage

r[native_package_parity.inventories] Mantle MUST derive generated operator artifacts and machine-contract cohort coverage from their declared owners.
The example catalog and architecture scan MUST cover their complete required surface within explicit bounds.
Documentation assertions MUST preserve each accepted non-claim.

#### Scenario: The declared inventory grows

- GIVEN a reviewed inventory contains additional surfaces or examples
- WHEN parity checks run
- THEN they MUST check exact membership, uniqueness, prerequisites, and generated freshness
- AND they MUST NOT accept a changed count without checking those facts.

#### Scenario: An entry or scan member is missing

- GIVEN an inventory entry is absent, duplicated, stale, or outside the scan budget
- WHEN parity checks run
- THEN they MUST fail explicitly rather than truncate, skip, or insert permissive placeholder data.

### Requirement: Narrow store capabilities preserve composed observations

r[native_package_parity.overlay] Mantle MUST preserve admitted base-only data, layer identity, precedence, trust, and generation facts through store capabilities and inspection.
Reads MUST NOT mutate or backfill a base or writable overlay. Writes MUST remain overlay-owned.

#### Scenario: Reopened base supplies a build or query

- GIVEN an explicitly trusted read-only base contains a valid signed path absent from the writable overlay
- WHEN the composed store supplies an input or inspection result after reopening
- THEN it MUST return the exact base path and layer observation
- AND the base and overlay MUST remain unchanged by the read.

#### Scenario: A base or shadow is unsafe

- GIVEN base permissions, prefix, generation, content, signature, or higher-layer shadow facts are invalid
- WHEN the composed store reads or prepares an effect
- THEN it MUST reject before the protected effect
- AND it MUST NOT borrow lower-layer trust or expose mutable service authority.

### Requirement: Remote status preserves the observed capture source

r[native_package_parity.capture] Mantle MUST preserve distinct worker and coordinator debug observations when it reports a failed remote attempt.
Capture success MUST require observed allowed artifact bytes. Diagnostic failure MUST NOT change build failure or output-admission truth.

#### Scenario: Worker capture succeeds before cleanup

- GIVEN the worker captures an explicitly allowed bounded artifact and records its content-bound bundle
- WHEN coordinator status reports the failed attempt
- THEN it MUST preserve the matching worker capture fact and bundle reference
- AND coordinator metadata-only diagnostics MUST NOT overwrite that fact.

#### Scenario: Worker capture is unavailable

- GIVEN no accepted worker capture observation exists or capture violates its policy
- WHEN status reports the attempt
- THEN it MUST retain the unavailable or rejected state without inventing captured bytes
- AND the original build failure MUST remain unchanged.

### Requirement: Native package acceptance uses complete bounded evidence

r[native_package_parity.verification] Mantle MUST retain focused baselines, positive and negative controls, exact source identities, and final native package status.
Acceptance MUST require the complete selected no-fail-fast package gate and applicable lint, freshness, and lifecycle checks without suppression.

#### Scenario: Selected gates pass

- GIVEN the committed source passes all selected package targets and applicable focused gates
- WHEN this change reports completion
- THEN every checked task MUST link current observed evidence
- AND the report MUST distinguish ignored cases, consumer admission, compiler correctness, reproducibility, and release authority.

#### Scenario: A gate fails or exhausts its budget

- GIVEN a selected gate fails, times out, or lacks evidence
- WHEN the change status is recorded
- THEN the relevant task MUST remain open with the exact blocker
- AND the change MUST NOT sync, archive, integrate into main, or authorize downstream activation.
