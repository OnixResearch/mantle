## ADDED Requirements

### Requirement: Offline fetcher examples

r[examples.offline_fetcher_fixtures] Mantle MUST provide deterministic offline examples or fixtures for every public fetch helper family documented in the examples cookbook.

#### Scenario: fetch helper has a local fixture
GIVEN the examples cookbook documents `fetchurl`, `fetchTarball`, or `fetchGit`
WHEN the offline examples validation suite runs
THEN the suite MUST exercise that helper through a local fixture or test-owned local repository.
AND the validation MUST run without reaching external network services.

#### Scenario: real-network cookbook remains available
GIVEN an example intentionally uses a real crates.io, GitHub, or raw-file URL for user clarity
WHEN the examples catalog classifies it
THEN the catalog MUST mark it as real-network.
AND fast validation MUST rely on the matching offline fixture rather than the real-network example.

### Requirement: Fixed-output negative examples

r[examples.fixed_output_negative_cases] Mantle MUST keep deterministic negative examples or fixtures for fixed-output hash mismatches and hash repair workflows.

#### Scenario: wrong hash fails closed
GIVEN a fetcher example or fixture has an intentionally wrong fixed-output hash
WHEN Mantle builds it without a repair flag
THEN the build MUST fail with a fixed-output mismatch diagnostic.
AND the test MUST assert that the failed build is not counted as a successful output.

#### Scenario: repair workflow is tested in a temp copy
GIVEN a fixed-output example is validated through `--fix` or an equivalent repair workflow
WHEN the repair test runs
THEN it MUST run against a temporary copy or generated fixture.
AND it MUST assert the corrected hash or repair diagnostic without mutating checked-in examples unexpectedly.
