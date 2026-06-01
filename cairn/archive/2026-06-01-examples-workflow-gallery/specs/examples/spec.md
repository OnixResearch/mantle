## ADDED Requirements

### Requirement: Progressive examples gallery

r[examples.progressive_gallery] Mantle SHOULD organize examples into a progressive gallery from beginner derivations through fetchers, package composition, project workflows, trust/provenance, and advanced bootstrap.

#### Scenario: beginner user follows examples
GIVEN a new user opens the examples README
WHEN they follow the recommended order
THEN they SHOULD encounter local, fast, low-prerequisite examples before examples that require generated seed material, real networks, or heavyweight bootstrap.
AND each lane SHOULD state the command to run and the expected output shape.

#### Scenario: package composition is demonstrated
GIVEN the gallery includes package-set or multi-output examples
WHEN the examples are validated
THEN at least one example SHOULD demonstrate a package consuming another package or output layout rather than only producing independent outputs.
AND validation SHOULD inspect the consumed output contract.

### Requirement: Trust and provenance examples

r[examples.trust_provenance_gallery] Mantle SHOULD include lightweight examples or command recipes that show how to inspect build outputs, artifact attestations, store evidence, or release proof material without overstating trust claims.

#### Scenario: local provenance command is runnable
GIVEN a trust/provenance example is marked runnable
WHEN the examples validation rail or focused smoke test runs it
THEN the command SHOULD produce deterministic local evidence such as JSON output, sidecar paths, or digest material.
AND the test SHOULD assert the evidence shape.

#### Scenario: proof workflow is only a skeleton
GIVEN a release, witness, or heavyweight proof workflow is documented as an example but not executed by the fast rail
WHEN the documentation describes it
THEN the text MUST state the prerequisites and non-claims explicitly.
AND it MUST NOT present placeholder or fake evidence as proof that a release or witness workflow succeeded.
