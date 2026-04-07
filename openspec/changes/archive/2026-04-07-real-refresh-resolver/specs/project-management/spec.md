## MODIFIED Requirements

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

## ADDED Requirements

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
