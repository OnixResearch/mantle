## ADDED Requirements

### Requirement: Example validation matrix

r[examples.validation_matrix] Mantle MUST validate cataloged examples through deterministic rails that match each example's declared support tier and capabilities.

#### Scenario: fast example validates without external services
GIVEN an example is cataloged as fast, local, and supported
WHEN the examples validation suite runs on a capable Linux host
THEN the suite MUST evaluate the Nickel file, convert it to Mantle derivation or project data where applicable, build it in temporary store/state roots when its tier requires building, and fail on unexpected diagnostics.
AND fast validation MUST NOT depend on real external network services or persistent host store mutation.

#### Scenario: heavyweight example is not hidden
GIVEN an example requires heavyweight bootstrap work, generated seed inputs, or real external networks
WHEN ordinary examples validation runs
THEN the suite MUST report an explicit skip or ignored-test boundary tied to the cataloged capability.
AND the repository MUST keep an explicit command or ignored test that can validate the example when that capability is intentionally enabled.

### Requirement: Example output execution checks

r[examples.output_execution] Mantle MUST execute or inspect built example outputs when the example claims a runnable artifact or named output layout.

#### Scenario: runnable output is produced
GIVEN an example claims to produce a runnable script or binary
WHEN the build smoke test succeeds
THEN the test MUST execute the artifact or inspect its deterministic output contract.
AND the assertion MUST check expected stdout, created files, or exit status rather than only checking that a path exists.

#### Scenario: invalid example fails intentionally
GIVEN an example is cataloged as a negative or diagnostic example
WHEN the validation suite runs the example
THEN the suite MUST assert the expected failure class or diagnostic shape.
AND a negative example MUST NOT be counted as an ordinary successful build.
