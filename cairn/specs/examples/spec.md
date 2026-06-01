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
