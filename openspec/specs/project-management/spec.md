## ADDED Requirements

### Requirement: Dedicated project-management crate

The workspace MUST contain a dedicated crate, `crunch-project`, that owns
project manifests, lockfiles, refresh logic, stale detection, upgrades, and
input materialization.

The `crunch` binary MUST delegate project-management commands to this crate.
The existing build, pipeline, and store crates MUST remain responsible for
fetch execution, sandboxed builds, and store persistence.

#### Scenario: Project command delegation

- GIVEN `crunch refresh` is invoked
- WHEN the CLI dispatches the command
- THEN the binary delegates the operation to `crunch-project`
- AND `crunch-build` is not used as the primary home for manifest or lock logic

### Requirement: Nickel-native project manifest

The system MUST support a human-edited Nickel manifest file named
`crunch-project.ncl`.

The manifest MUST support, at minimum:
- named inputs
- input kind metadata for file, tarball, and git sources
- optional mirrors
- optional patch lists
- frozen inputs
- refresh metadata needed to discover new revisions or hashes

The system MUST NOT require KDL for project input management.

#### Scenario: Project manifest defines inputs

- GIVEN a `crunch-project.ncl` file with named source inputs
- WHEN `crunch check` reads the project state
- THEN the manifest is validated through the project layer
- AND invalid input shapes are reported before any refresh or build work starts

### Requirement: Machine-edited JSON lockfile

The system MUST maintain a machine-edited lockfile named `crunch.lock`.

The lockfile MUST use JSON as its on-disk format. It MUST record the resolved
values needed to build inputs repeatably, including source URLs or revisions,
selected mirrors, patch locks, and verified hashes.

#### Scenario: Refresh writes lockfile

- GIVEN a valid project manifest
- WHEN `crunch refresh` updates an input
- THEN `crunch.lock` is rewritten with the new resolved state
- AND the result is sufficient to reproduce the same input later without
  consulting the network for freshness data

#### Scenario: Lockfile is JSON

- GIVEN a current project state
- WHEN the project layer writes `crunch.lock`
- THEN the file is valid JSON
- AND the project layer can read it back without loss of resolved input data

### Requirement: Generated inputs file

The system MUST materialize a generated file at `.crunch/inputs.ncl` from the
current lockfile.

Package Nickel code MUST be able to import that file to access locked inputs.
Normal `crunch eval` and `crunch build` flows MUST stay pure with respect to
network updates: they read the manifest, lock, and generated inputs file, but
MUST NOT refresh inputs implicitly.

#### Scenario: Package code imports locked inputs

- GIVEN a current `crunch.lock`
- WHEN the project layer generates `.crunch/inputs.ncl`
- THEN a package file can `import ".crunch/inputs.ncl"`
- AND the imported values correspond to the current lock state

#### Scenario: Build does not refresh inputs

- GIVEN a project with an existing manifest, lockfile, and generated inputs file
- WHEN `crunch build` evaluates a package file that imports `.crunch/inputs.ncl`
- THEN the build uses the locked inputs already on disk
- AND no stale detection or network refresh is triggered implicitly

### Requirement: Refresh and stale detection

The system MUST provide project-level refresh and stale-detection behavior.

`refresh` updates selected or all non-frozen inputs, rewrites `crunch.lock`,
and regenerates `.crunch/inputs.ncl`.

`list-stale` reports which inputs would change without mutating project files.

#### Scenario: Frozen input skipped on refresh

- GIVEN an input marked frozen in `crunch-project.ncl`
- WHEN `crunch refresh` runs
- THEN that input is left unchanged
- AND the command reports that it was skipped

#### Scenario: Stale input reported without writes

- GIVEN an input whose latest upstream revision differs from the current lock
- WHEN `crunch list-stale` runs
- THEN the input is reported as stale
- AND neither `crunch.lock` nor `.crunch/inputs.ncl` is modified

### Requirement: Mirrors and patches are first-class project input metadata

The manifest and lockfile MUST support mirrors and patch lists as project input
metadata.

The resolved mirrors and patches MUST feed into the existing fetch/build
pipeline instead of introducing a second fetch implementation.

#### Scenario: Mirror fallback is locked

- GIVEN an input with a primary URL and one or more mirrors
- WHEN the project layer resolves that input
- THEN the lockfile records the mirror data needed for repeatable fetches

#### Scenario: Patch list materialized with input

- GIVEN an input that references named patches
- WHEN `.crunch/inputs.ncl` is generated
- THEN the materialized input data preserves the locked patch information
- AND downstream package code can apply those patches through the existing
  crunch fetch/build mechanisms

### Requirement: Versioned manifest and lock formats

Both `crunch-project.ncl` and `crunch.lock` MUST carry explicit schema
versions. The project layer MUST provide migrations between supported schema
versions.

#### Scenario: Upgrade rewrites old schema

- GIVEN a project manifest and lockfile written in an older supported version
- WHEN `crunch upgrade` runs
- THEN the files are rewritten to the current schema version
- AND the resulting project state remains loadable by `crunch-project`

### Requirement: Drift detection

The project layer MUST detect drift between `crunch.lock` and
`.crunch/inputs.ncl`.

#### Scenario: Generated inputs file out of date

- GIVEN `crunch.lock` changed but `.crunch/inputs.ncl` was not regenerated
- WHEN `crunch check` runs
- THEN the command reports drift
- AND exits non-zero until the generated inputs file is refreshed
