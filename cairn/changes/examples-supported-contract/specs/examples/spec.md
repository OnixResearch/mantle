## ADDED Requirements

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
