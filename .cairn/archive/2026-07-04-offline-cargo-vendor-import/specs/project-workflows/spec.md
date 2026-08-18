## ADDED Requirements

### Requirement: Cargo import accepts declared vendored source material

r[project_workflows.cargo_import_vendored_sources] Mantle MUST allow `mantle import cargo` to scaffold the offline Cargo project-build lane for workspaces with registry or git dependencies only when the required dependency material is present as an explicit vendored source input. The import planner MUST bind accepted vendored material to `Cargo.lock`, Cargo checksum metadata, source replacement configuration, Mantle BLAKE3 identities, and generated `vendor_src` / `vendor_name` inputs, and MUST fail closed instead of consulting ambient Cargo caches or enabling network access.

#### Scenario: vendored registry material is accepted

GIVEN a Cargo workspace has a `Cargo.lock` entry for a registry dependency
AND source replacement configuration points to a local vendored directory containing that package
AND the vendored package files match Cargo checksum metadata
WHEN an operator runs `mantle import cargo --plan` or `--apply`
THEN Mantle MAY report the dependency material as an accepted vendored source input
AND generated project files MUST pass that source input explicitly to `mantle.offlineCargoPackage`.

#### Scenario: undeclared dependency material blocks import

GIVEN a Cargo workspace has registry or git dependencies without accepted vendored source material
WHEN an operator runs `mantle import cargo --plan` or `--apply`
THEN Mantle MUST emit deterministic blockers naming the missing or unsupported dependency material
AND it MUST NOT generate partial files that imply the package is ready for `mantle build`.

#### Scenario: stale vendor checksums fail closed

GIVEN a vendored package directory exists but its files, lockfile identity, package version, source identity, or `.cargo-checksum.json` metadata do not match the selected dependency facts
WHEN Mantle validates the import plan
THEN Mantle MUST reject the vendored material before applying generated files
AND diagnostics MUST distinguish stale checksum, ambiguous package identity, missing lock entry, and unsupported source replacement classes.

#### Scenario: import does not run vendoring

GIVEN a workspace could be made buildable by running `cargo vendor`, fetching registry material, cloning git dependencies, or reading a user Cargo cache
WHEN Mantle plans Cargo import
THEN Mantle MUST NOT perform those actions
AND the plan MUST report that pre-existing declared source material is required.
