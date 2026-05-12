# Project Management Specification

## Purpose

Defines Nickel project manifests, JSON lockfiles, generated locked inputs,
refresh and stale-detection behavior, resolver ownership, upgrade paths,
and drift checks for crunch projects.
## Requirements
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

The system MUST provide project-level refresh and stale-detection behavior
backed by real source resolution whose locked hashes match the build
pipeline.

`refresh` MUST resolve each non-frozen input against its upstream source and
record the hash semantics required by the downstream fetch helpers:
- For git inputs, it MUST resolve the configured reference to a concrete
  commit SHA. Branch and tag refs MUST be resolved with `git ls-remote`.
  Rev refs MUST be validated as full 40-character hex SHAs.
- For git inputs, it MUST also compute the recursive/NAR hash of the
  checkout at the resolved commit, matching `crunch.fetchGit`.
- For file inputs, it MUST download the URL content and compute the
  manifest-declared flat content hash over the raw downloaded bytes.
- For tarball inputs, it MUST download, unpack, and hash the unpacked tree
  using the same recursive/NAR semantics as `crunch.fetchTarball`. It MUST
  NOT lock a flat hash of the compressed archive bytes.
- When `refresh` writes the lockfile, any referenced local patch MUST be
  resolved relative to the project root, hashed from disk, and recorded in
  `Lockfile.patches`. Any referenced remote patch MUST be downloaded and
  hashed with the declared algorithm.

`refresh` MUST NOT silently return no-op results. If the resolver cannot
reach an upstream source or hash a referenced patch (network failure,
missing git, invalid URL, missing local file), it MUST return an error for
that item and continue resolving remaining inputs.

`list-stale` MUST use the same resolution logic to compare upstream state
against locked state. It MUST NOT mutate project files.

`list-stale` MUST report stale inputs and resolution failures separately.
It MUST NOT report `all inputs up to date` if any input could not be
checked.

#### Scenario: Git input refresh resolves current rev and checkout hash

- GIVEN a git input configured to follow the `main` branch
- WHEN `crunch refresh` runs
- THEN the resolver calls `git ls-remote <repository> refs/heads/main`
- AND the lockfile entry is updated with the resolved commit SHA
- AND the lockfile hash is updated with the recursive hash of the checkout at that SHA
- AND `.crunch/inputs.ncl` is regenerated

#### Scenario: Git tag reference resolved with peeled tag target

- GIVEN a git input configured to follow the `v1.0` tag
- WHEN `crunch refresh` runs
- THEN the resolver calls `git ls-remote <repository> refs/tags/v1.0`
- AND it prefers the peeled `^{}` line when present
- AND the lockfile records the commit SHA that the tag points to

#### Scenario: Git rev reference validated

- GIVEN a git input configured with an explicit rev `abc123...`
- WHEN `crunch refresh` runs
- THEN the resolver validates the rev string as 40-character hex
- AND returns that rev unchanged
- AND refresh still computes the recursive checkout hash for the lock entry

#### Scenario: Tarball input hash resolved from unpacked tree

- GIVEN a tarball input whose declared hash algorithm is `sha256`
- WHEN `crunch refresh` runs
- THEN the resolver downloads the tarball
- AND unpacks it using the same semantics as `crunch.fetchTarball`
- AND computes the sha256 recursive/NAR hash of the unpacked tree
- AND updates the lockfile with the SRI-encoded tree hash

#### Scenario: File input hash resolved from downloaded bytes

- GIVEN a file input whose declared hash algorithm is `blake3`
- WHEN `crunch refresh` runs
- THEN the resolver downloads the file content
- AND computes the blake3 hash over the raw bytes
- AND updates the lockfile with the SRI-encoded hash

#### Scenario: Local patch hash resolved during refresh

- GIVEN an input that references a local patch
- WHEN `crunch refresh` runs
- THEN the patch path is resolved relative to the project root
- AND the patch file is hashed from disk
- AND `Lockfile.patches` records a non-empty locked hash for that patch

#### Scenario: Network failure on one input does not block others

- GIVEN a manifest with inputs A (reachable) and B (unreachable)
- WHEN `crunch refresh` runs
- THEN input A is resolved and updated in the lockfile
- AND input B's error is reported to the user
- AND the command exits non-zero

#### Scenario: Stale detection uses live resolution

- GIVEN a locked git input at rev `aaa...` but upstream `main` now points to `bbb...`
- WHEN `crunch list-stale` runs
- THEN the input is reported as stale with the current and new revs
- AND neither `crunch.lock` nor `.crunch/inputs.ncl` is modified

#### Scenario: Stale detection reports failures distinctly

- GIVEN one input is stale and another input cannot be checked
- WHEN `crunch list-stale` runs
- THEN the stale input is reported
- AND the failed check is reported separately
- AND the command does not print `all inputs up to date`
- AND the command exits non-zero

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

### Requirement: Resolver implementation in binary crate

The `crunch` binary crate MUST provide a concrete `RefreshResolver`
implementation that performs real I/O. The `StubResolver` MUST be removed.

The resolver MUST NOT live in `crunch-project` — that crate remains
pure (no I/O, no async, no subprocess calls). The binary crate is the
imperative shell; `crunch-project` is the functional core.

The live resolver MUST implement every applicable resolver hook needed for
production refresh, including local file hashing used for patch locking.
For local patches, it MUST interpret manifest paths relative to the project
root.

#### Scenario: No stub resolver in production code

- GIVEN the crunch binary crate source
- WHEN inspected for `RefreshResolver` implementations
- THEN exactly one implementation exists (the live resolver)
- AND it performs actual network/subprocess/path-based resolution
- AND no production code path returns `Ok(None)` unconditionally for git, remote URL, or local patch hashing

### Requirement: Git resolution via subprocess

The resolver MUST resolve git references by running `git ls-remote`.
It MUST NOT shell out to `nix-prefetch-git` or any Nix-specific tool.

`git` MUST be located via PATH search. If `git` is not found, the
resolver MUST return a clear error naming the missing binary.

#### Scenario: Git not on PATH

- GIVEN `git` is not in PATH
- WHEN the resolver attempts to resolve a git input
- THEN it returns an error message containing `git` and `not found` (or equivalent)
- AND the error does not panic or abort the process

### Requirement: Source hashing semantics

The resolver interface and implementation MUST distinguish flat byte hashes
from recursive tree hashes so refresh writes lock values that the existing
fetch/build helpers will accept.

- Plain files and patch files use flat content hashes.
- Tarballs and git checkouts use recursive/NAR tree hashes.

Downloads and temp materialization MUST be size-limited (reuse
`MAX_DOWNLOAD_BYTES` or equivalent). Temporary files/directories MUST be
cleaned up on success and failure.

Supported algorithms remain sha256, sha512, and blake3.

#### Scenario: Download exceeds size limit

- GIVEN a URL that returns more than the size limit
- WHEN the resolver downloads content
- THEN it aborts the download and returns a size-exceeded error
- AND temporary files/directories are cleaned up

### Requirement: Stale reporting preserves failures

The project-management layer MUST preserve stale-check failures in its API
instead of collapsing results to a list of stale names.

CLI callers MUST be able to render stale inputs and failed checks
separately.

#### Scenario: Empty stale set is not mistaken for success

- GIVEN stale detection cannot check any inputs because resolution fails
- WHEN the result is returned to the CLI
- THEN the API distinguishes that case from `no stale inputs`
- AND the CLI does not print `all inputs up to date`

### Requirement: Bootstrap Drain State Hygiene [r[bootstrap-state-handoff-hygiene]]
Crunch MUST keep bootstrap drain handoff state consistent with live OpenSpec queue state so agents do not resume stale blockers as active work.

#### Scenario: Stale state is not presented as active [r[bootstrap-state-handoff-hygiene.1]]
- GIVEN there are no active OpenSpec changes
- WHEN a root drain-state file is present
- THEN it is either current, archived, or removed with context preserved elsewhere

#### Scenario: Cleanup is verified [r[bootstrap-state-handoff-hygiene.2]]
- GIVEN the handoff state is reconciled
- WHEN `openspec validate --all --strict` and git status are checked
- THEN the repository has no misleading active-drain artifact
