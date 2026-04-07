## MODIFIED Requirements

### Requirement: Refresh and stale detection

The system MUST provide project-level refresh and stale-detection behavior
backed by real network resolution.

`refresh` MUST resolve each non-frozen input against its upstream source:
- For git inputs, the resolver MUST run `git ls-remote` and return the
  current commit SHA for the configured reference.
- For file and tarball inputs, the resolver MUST download the URL content
  and compute a content hash using the algorithm declared in the manifest.

`refresh` MUST NOT silently return no-op results. If the resolver cannot
reach the upstream (network failure, missing git, invalid URL), it MUST
return an error for that input and continue resolving remaining inputs.

`list-stale` MUST use the same resolution logic to compare upstream state
against locked state, reporting which inputs would change. It MUST NOT
mutate project files.

#### Scenario: Git input refresh resolves current rev

- GIVEN a git input with `reference = { ref_type = 'branch, ref_value = "main" }`
- WHEN `crunch refresh` runs
- THEN the resolver calls `git ls-remote <repository> refs/heads/main`
- AND the lockfile entry is updated with the resolved commit SHA
- AND `.crunch/inputs.ncl` is regenerated

#### Scenario: Git tag reference resolved

- GIVEN a git input with `reference = { ref_type = 'tag, ref_value = "v1.0" }`
- WHEN `crunch refresh` runs
- THEN the resolver calls `git ls-remote <repository> refs/tags/v1.0`
- AND the lockfile records the commit SHA that the tag points to

#### Scenario: Git rev reference validated

- GIVEN a git input with `reference = { ref_type = 'rev, ref_value = "abc123..." }`
- WHEN `crunch refresh` runs
- THEN the resolver validates the format of the rev string (40-char hex)
- AND returns it unchanged (revs are already pinned)

#### Scenario: URL content hash resolved

- GIVEN a tarball input with `url` and `hash = { algo = 'sha256 }`
- WHEN `crunch refresh` runs
- THEN the resolver downloads the URL content
- AND computes the sha256 hash over the raw bytes
- AND updates the lockfile with the SRI-encoded hash

#### Scenario: File input hash resolved

- GIVEN a file input with `url` and `hash = { algo = 'blake3 }`
- WHEN `crunch refresh` runs
- THEN the resolver downloads the file content
- AND computes the blake3 hash
- AND updates the lockfile with the SRI-encoded hash

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

## ADDED Requirements

### Requirement: Resolver implementation in binary crate

The `crunch` binary crate MUST provide a concrete `RefreshResolver`
implementation that performs real I/O. The `StubResolver` MUST be removed.

The resolver MUST NOT live in `crunch-project` — that crate remains
pure (no I/O, no async, no subprocess calls). The binary crate is the
imperative shell; `crunch-project` is the functional core.

#### Scenario: No stub resolver in production code

- GIVEN the crunch binary crate source
- WHEN inspected for `RefreshResolver` implementations
- THEN exactly one implementation exists (the live resolver)
- AND it performs actual network/subprocess resolution
- AND no code path returns `Ok(None)` unconditionally

### Requirement: Git resolution via subprocess

The resolver MUST resolve git references by running `git ls-remote`.
It MUST NOT shell out to `nix-prefetch-git` or any Nix-specific tool.

`git` MUST be located via PATH search. If `git` is not found, the
resolver MUST return a clear error naming the missing binary.

#### Scenario: Git not on PATH

- GIVEN `git` is not in PATH
- WHEN the resolver attempts to resolve a git input
- THEN it returns an error message containing "git" and "not found" (or equivalent)
- AND the error does not panic or abort the process

### Requirement: URL content hashing

The resolver MUST download URL content to a temporary location, compute
the hash with the manifest-declared algorithm, and return the SRI string.

Downloads MUST be size-limited (reuse the existing `MAX_DOWNLOAD_BYTES`
constant or equivalent). Temporary files MUST be cleaned up on success
and failure (RAII via `tempfile`).

Supported algorithms: sha256, sha512, blake3 (matching `crunch-project::HashAlgo`).

#### Scenario: Download exceeds size limit

- GIVEN a URL that returns more than the size limit
- WHEN the resolver downloads content
- THEN it aborts the download and returns a size-exceeded error
- AND the temp file is cleaned up
