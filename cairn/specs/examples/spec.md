# Examples Specification

## Purpose

Defines the `examples` capability.

## Requirements

### Requirement: Example support catalog

r[examples.support_catalog] Mantle MUST maintain a source-controlled examples catalog that names every supported example, its support tier, required runtime capabilities, network expectations, and validation rail.

#### Scenario: supported example is added
GIVEN a new file is added under `examples/` and is intended for users
WHEN the examples inventory check runs
THEN the catalog MUST include that example with a stable id, relative path, support tier, required capabilities, and expected validation rail.
AND the check MUST reject unsupported tier names, missing paths, duplicate ids, or duplicate relative paths.

#### Scenario: example is generated or intentionally manual
GIVEN an example depends on generated seed material, heavyweight bootstrap work, or real external networks
WHEN it is listed in the catalog
THEN the catalog MUST mark the dependency explicitly.
AND fast validation MUST skip only the documented capability boundary instead of silently skipping the example.

### Requirement: Example documentation drift rail

r[examples.documentation_drift] Mantle MUST keep example documentation synchronized with the catalog and with actual checked-in example paths.

#### Scenario: README index is stale
GIVEN `examples/README.md` or the root README examples section references examples
WHEN the documentation drift check runs
THEN every documented example path MUST exist and every cataloged user-facing example MUST appear in the appropriate README index.
AND missing links, stale links, and omitted supported examples MUST fail the check.

#### Scenario: user-facing prose names the product
GIVEN example comments, README entries, or command snippets describe user-facing behavior
WHEN the drift check inspects those files
THEN the prose SHOULD use Mantle naming.
AND exact `crunch` names MAY remain only for current binary, crate, package, path, or compatibility identifiers that still require that spelling.

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

### Requirement: Rust project compatibility gallery [r[examples.rust_project_compatibility_gallery]]

Mantle SHOULD document the representative Rust compatibility rail in the examples gallery with explicit support tier, prerequisites, command, expected output shape, validation rail, and non-claims.

#### Scenario: Gallery identifies the Rust compatibility rail [r[examples.rust_project_compatibility_gallery.scenario.catalog]]

- GIVEN the representative Rust compatibility rail is added or updated
- WHEN the examples catalog and README are checked
- THEN they SHOULD name the fixture or generated example, support tier, required capabilities, validation command, and expected binary/output behavior
- AND they SHOULD distinguish fast local checks from heavyweight or Linux-only integration checks.

#### Scenario: Gallery does not overclaim Cargo compatibility [r[examples.rust_project_compatibility_gallery.scenario.non-claims]]

- GIVEN the Rust compatibility rail appears in user-facing docs
- WHEN the docs describe what the rail proves
- THEN they MUST state whether the evidence came from sandboxed offline Cargo, native rust-plan, or both
- AND they MUST NOT present the rail as proof of full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness.
