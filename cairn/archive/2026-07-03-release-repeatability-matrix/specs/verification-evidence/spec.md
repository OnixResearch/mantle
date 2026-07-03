## ADDED Requirements

### Requirement: Release repeatability matrix

r[verification_evidence.release_repeatability_matrix] Mantle MUST provide a deterministic repeatability matrix before widening a release reproducibility claim beyond a single recorded proof context.

#### Scenario: matrix axes are explicit

GIVEN an operator requests a release repeatability matrix for a release evidence bundle
WHEN Mantle creates the matrix plan
THEN the plan MUST bind the release id, artifact surfaces, expected output digest set, matrix profile digest, cache mode, store isolation mode, environment controls, temp-root controls, user controls, host class, and run count
AND any omitted axis MUST be recorded as an explicit non-claim.

#### Scenario: fresh-store cells cannot reuse prior outputs silently

GIVEN a matrix cell is configured as a fresh-store rebuild
WHEN Mantle executes that cell
THEN the cell MUST use isolated output and store roots and record their identities
AND reused-store or substitution evidence MUST be recorded as a blocker unless the cell explicitly tests reuse.

#### Scenario: mismatches block wider admission

GIVEN any matrix cell produces a missing or mismatched output digest for an included release surface
WHEN Mantle emits the matrix report or global reproducibility evidence
THEN the report MUST preserve the expected digest, observed digest when present, failing axis, and blocker class
AND the affected surface MUST NOT be admitted as repeatability-proven.
