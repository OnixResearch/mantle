# Fetchers Specification

## Purpose

Defines how crunch downloads external resources (URLs, tarballs, git
repos) and stores them as fixed-output derivations. Fetchers bridge
the gap between the network and the hermetic build sandbox.

## Requirements

### Requirement: fetchurl — download a file by URL

The Nickel stdlib MUST provide `crunch.fetchurl` that accepts a URL
and a content hash, and produces a fixed-output derivation record.

```nickel
let src = crunch.fetchurl {
  url = "https://example.com/foo-1.0.tar.gz",
  hash = "sha256-XXXX...",
} in
```

The resulting derivation MUST have:
- `builder = "builtin:fetchurl"`
- `system = "builtin"`
- `fixed_output.mode = 'flat` (hash of the raw file bytes)
- `env.url` set to the URL

The output store path MUST be computed from the declared hash
(standard FOD path computation via `build_ca_path`).

#### Scenario: Fetch a single file

- GIVEN a `.ncl` file with `crunch.fetchurl { url = "https://...", hash = "sha256-..." }`
- WHEN `crunch build` runs
- THEN the file is downloaded, hash-verified, and stored at the FOD output path

#### Scenario: Cached fetch is skipped

- GIVEN the FOD output path already exists in the store
- WHEN `crunch build` runs again
- THEN no download occurs (standard FOD caching)

#### Scenario: Name defaults to URL basename

- GIVEN `crunch.fetchurl { url = "https://example.com/foo-1.0.tar.gz", hash = "..." }`
- WHEN the derivation is constructed
- THEN `name` is `"foo-1.0.tar.gz"`

#### Scenario: Explicit name override

- GIVEN `crunch.fetchurl { url = "https://...", hash = "...", name = "source" }`
- WHEN the derivation is constructed
- THEN `name` is `"source"`

### Requirement: fetchTarball — download and unpack

The Nickel stdlib MUST provide `crunch.fetchTarball` that downloads a
tarball, decompresses it, extracts the contents, strips the top-level
directory component, and stores the result.

```nickel
let src = crunch.fetchTarball {
  url = "https://github.com/user/repo/archive/v1.0.tar.gz",
  hash = "sha256-XXXX...",
} in
```

The resulting derivation MUST have:
- `builder = "builtin:fetchurl"`
- `system = "builtin"`
- `fixed_output.mode = 'recursive` (NAR hash of unpacked tree)
- `env.url` set to the URL
- `env.unpack = "1"`

Supported compression formats: gzip (`.gz`, `.tgz`), xz (`.xz`, `.txz`),
bzip2 (`.bz2`, `.tbz2`), zstd (`.zst`, `.zstd`), and uncompressed `.tar`.

#### Scenario: Fetch and unpack a tarball

- GIVEN a `.ncl` file with `crunch.fetchTarball { url = "https://...tar.gz", hash = "..." }`
- WHEN `crunch build` runs
- THEN the tarball is downloaded, decompressed, extracted with top-level
  dir stripped, and the unpacked tree is stored

#### Scenario: Top-level directory stripping

- GIVEN a tarball where all entries are under `project-v1.0/`
- WHEN extracted
- THEN the store path contains the contents directly (no `project-v1.0/` prefix)

#### Scenario: Unsupported compression detected

- GIVEN a URL ending in `.tar.lz4` (unsupported)
- WHEN crunch attempts to fetch
- THEN it falls back to raw read (gzip magic detection), or fails with
  a clear error naming the unsupported format

### Requirement: fetchGit — clone a git repository

The Nickel stdlib MUST provide `crunch.fetchGit` that clones a git
repository at a specific revision and stores the working tree (without
`.git/`).

```nickel
let src = crunch.fetchGit {
  url = "https://github.com/user/repo.git",
  rev = "abc123def456...",
  hash = "sha256-XXXX...",
} in
```

The resulting derivation MUST have:
- `builder = "builtin:fetchurl"`
- `system = "builtin"`
- `fixed_output.mode = 'recursive` (NAR hash of checkout)
- `env.url` set to the URL
- `env.type = "git"`
- `env.rev` set to the commit hash

The implementation MUST shell out to the `git` binary. The `.git/`
directory MUST NOT appear in the output.

#### Scenario: Clone at specific rev

- GIVEN `crunch.fetchGit { url = "...", rev = "abc123...", hash = "..." }`
- WHEN `crunch build` runs
- THEN the repo is cloned, checked out at the specified rev, `.git/`
  removed, and the tree stored

#### Scenario: Git not installed

- GIVEN `git` is not in PATH
- WHEN a fetchGit derivation is built
- THEN the error message says "git command not found" and suggests
  installing it

#### Scenario: Invalid rev

- GIVEN a rev that doesn't exist in the remote
- WHEN the fetch runs
- THEN the error includes the git stderr with the failed checkout

### Requirement: Fetch BuildRequest encoding

Fetcher derivations MUST reuse the existing `BuildRequest` fields.
They MUST NOT require a new request format.

A fetch request is encoded as follows:
- `command_args[0] == "builtin:fetchurl"`
- `environment_vars` carries the fetch parameters by their existing
  names: `url`, `unpack`, `type`, `rev`, `executable`
- `outputs[0]` is the fetch output path

#### Scenario: fetchurl request uses existing BuildRequest fields

- GIVEN a fetchurl derivation converted to `BuildRequest`
- WHEN the request is inspected
- THEN `command_args[0]` is `builtin:fetchurl`
- AND `environment_vars` still contains `url`
- AND no new BuildRequest fields are needed

### Requirement: FetchBuildService

The system MUST provide a `FetchBuildService` that implements the
`BuildService` trait. When `do_build()` receives a fetch request, it
MUST parse fetch parameters from the existing `BuildRequest` fields,
perform the download or extraction, ingest the result into castore, and
produce a `BuildResult` with the output node.

`FetchBuildService` MUST NOT perform fixed-output hash verification,
`PathInfo` persistence, or disk export. Those steps stay in the shared
post-build path.

#### Scenario: Fetcher produces BuildResult

- GIVEN a BuildRequest with `command_args[0] = "builtin:fetchurl"`
- AND `environment_vars` contains `url`
- WHEN `fetch_service.do_build(request)` is called
- THEN the resource is downloaded or unpacked
- AND a `BuildResult` is returned with the output node

#### Scenario: Non-fetch request rejected

- GIVEN a BuildRequest with `command_args[0] = "/bin/sh"`
- WHEN `fetch_service.do_build(request)` is called
- THEN an error is returned (this service only handles fetchers)

### Requirement: DispatchBuildService

The system MUST provide a `DispatchBuildService` that wraps a
`FetchBuildService` and a sandbox `BuildService`. It inspects
`request.command_args[0]` and routes to the appropriate implementation.

#### Scenario: Fetch derivation dispatched to fetch service

- GIVEN a DispatchBuildService wrapping fetch + sandbox services
- WHEN a BuildRequest with `command_args[0] = "builtin:fetchurl"` arrives
- THEN it is dispatched to the FetchBuildService
- AND the sandbox service is not called

#### Scenario: Regular derivation dispatched to sandbox

- GIVEN a DispatchBuildService wrapping fetch + sandbox services
- WHEN a BuildRequest with `command_args[0] = "/bin/sh"` arrives
- THEN it is dispatched to the sandbox BuildService

### Requirement: Uniform orchestrator dispatch

The orchestrator MUST NOT contain fetcher-specific branching or inline
fetch execution. The `prepare_build()` method MUST treat all derivations
the same: it constructs a `BuildRequest` and dispatches via the
`BuildService` trait.

Fetcher derivations still bypass the sandbox, but they do so by being
routed to `FetchBuildService` through `DispatchBuildService`, not by
bypassing `BuildService::do_build()` entirely.

#### Scenario: No is_builtin_fetcher check

- GIVEN the orchestrator's `prepare_build()` method
- WHEN inspected
- THEN it does not check `is_builtin_fetcher()` or call `build_fetcher()`
- AND all derivations go through the same dispatch path

### Requirement: Hash verification

After a fetch build returns a `BuildResult`, the shared post-build path
(`finish_build`) MUST verify the output against the declared hash:

- For `mode = 'flat` (fetchurl): hash the raw file bytes from the
  produced file node
- For `mode = 'recursive` (fetchTarball, fetchGit): compute the NAR hash
  of the produced output tree

A mismatch MUST:
1. Delete the produced output before persist/export
2. Report both expected and actual hash in SRI format
3. Report the `.ncl` source file and approximate location of the hash to
   update

#### Scenario: Hash matches

- GIVEN a fetchurl with `hash = "sha256-XXXX..."` and the downloaded
  file hashes to the same value
- WHEN verification runs in the shared post-build path
- THEN the fetch succeeds

#### Scenario: Hash mismatch

- GIVEN a fetchurl with `hash = "sha256-AAAA..."` but the actual
  content hashes to `sha256-BBBB...`
- WHEN verification runs in the shared post-build path
- THEN the output is deleted
- AND the error reports:
  ```
  hash mismatch for fetchurl 'foo-1.0.tar.gz':
    expected: sha256-AAAA...
    got:      sha256-BBBB...
    update hello.ncl to: hash = "sha256-BBBB..."
  ```

### Requirement: Auto-fix hash mismatches

The CLI MUST support `crunch build --fix <file.ncl>`. When a FOD
hash mismatches with `--fix`:

1. Compute the correct hash
2. Find the old hash string in the `.ncl` source file
3. Replace it with the correct hash
4. Report the change
5. Exit with an error instructing the user to re-run

The build MUST NOT continue after a `--fix` rewrite. The hash
change alters the derivation's ATerm, which changes its `.drv`
store path and output path. The current session's `KnownPaths`,
goal registry, and any in-flight dependency edges reference the
old paths. Re-evaluating from scratch is the only safe option.

Without `--fix`, the mismatch is reported as an error with the
suggested fix (but the file is not modified).

#### Scenario: Auto-fix updates hash and exits

- GIVEN `hello.ncl` with `hash = "sha256-AAAA..."` and actual is
  `sha256-BBBB...`
- WHEN `crunch build --fix hello.ncl` runs
- THEN `hello.ncl` is updated: `sha256-AAAA...` → `sha256-BBBB...`
- AND crunch exits with an error: "re-run to build with the corrected hash"

#### Scenario: Auto-fix without --fix just reports

- GIVEN the same mismatch
- WHEN `crunch build hello.ncl` (no `--fix`) runs
- THEN the error includes the correct hash but the file is NOT modified

### Requirement: Hash algorithms

Fetchers MUST support SHA-256 as the primary hash algorithm. They
SHOULD support SHA-512 and SHA-1 for compatibility with existing
hash databases. BLAKE3 MUST be supported for flat hashes (extending
the `HashAlgo` enum).

Hash strings MUST be accepted in SRI format (`sha256-<base64>`)
and hex format (`<hex>`). SRI is preferred for `.ncl` files.

#### Scenario: SRI hash

- GIVEN `hash = "sha256-Q3QXOoy+iN4VK2CflvRulYvPZXYgF0dO7FoF7CvWFTA="`
- WHEN the fetcher verifies
- THEN the SRI string is parsed and compared correctly

#### Scenario: Hex hash

- GIVEN `hash = "4374173a8cbe88de152b609f96f46e958bcf65762017474eec5a05ec2bd61530"`
- WHEN the fetcher verifies
- THEN the hex string is parsed and compared correctly

### Requirement: Decompression support

The tarball fetcher MUST support these compression formats, detected
by URL suffix:

| Suffix | Format |
|---|---|
| `.gz`, `.tgz` | gzip |
| `.xz`, `.txz` | xz/lzma |
| `.bz2`, `.tbz2` | bzip2 |
| `.zst`, `.zstd` | zstd |
| `.tar` | uncompressed |

For unknown suffixes, the system SHOULD attempt gzip magic byte
detection, then fall back to raw read.

All decompression MUST use pure-Rust implementations (flate2 with
rust_backend, lzma-rs, bzip2-rs, ruzstd) for portability.

#### Scenario: xz tarball

- GIVEN a URL ending in `.tar.xz`
- WHEN fetched and unpacked
- THEN xz decompression is applied before tar extraction

### Requirement: Network isolation

Fetcher execution MUST happen outside the build sandbox. Fetch
requests MAY flow through a composite `BuildService`, but they MUST be
routed to a non-sandbox implementation such as `FetchBuildService`.

Regular (non-fetcher) derivations MUST NOT have network access. The
sandbox BuildService MUST continue to block network for all non-builtin
builders.

#### Scenario: Fetch request bypasses the sandbox service

- GIVEN a fetch derivation encoded as a BuildRequest
- WHEN DispatchBuildService handles it
- THEN FetchBuildService runs with network access
- AND the sandbox BuildService is not called

#### Scenario: Regular build has no network

- GIVEN a derivation with `builder = "/bin/sh"`
- WHEN built in the sandbox
- THEN network access is blocked (existing behavior, unchanged)

### Requirement: Fetch derivation contract in Nickel

The Nickel stdlib MUST validate fetch derivations at eval time:

- `url` is a non-empty string
- `hash` is a non-empty string matching SRI or hex format
- `rev` (for fetchGit) is a non-empty string
- `name` if provided is a valid derivation name

Invalid fetch parameters MUST be caught by Nickel contracts,
not at build time.

#### Scenario: Empty URL rejected

- GIVEN `crunch.fetchurl { url = "", hash = "sha256-..." }`
- WHEN evaluated
- THEN Nickel reports a contract violation for empty URL

#### Scenario: Invalid hash format rejected

- GIVEN `crunch.fetchurl { url = "...", hash = "not-a-hash" }`
- WHEN evaluated
- THEN Nickel reports a contract violation for invalid hash format
