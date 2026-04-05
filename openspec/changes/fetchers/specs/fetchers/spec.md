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

### Requirement: Builtin fetcher bypass in orchestrator

When the build orchestrator encounters a derivation with
`builder == "builtin:fetchurl"`, it MUST NOT invoke the sandbox
(`BuildService::do_build`). Instead, it MUST execute the fetch
directly.

The fetch execution MUST:
1. Parse the derivation environment to determine fetch type
   (URL/tarball/git based on `env.unpack` and `env.type`)
2. Download the resource
3. Verify the content hash against the FOD's declared hash
4. Compute NAR hash and size of the output
5. Scan for store path references
6. Persist PathInfo

Steps 4-6 are identical to the post-build processing for regular
derivations.

#### Scenario: Fetcher skips sandbox

- GIVEN a derivation with `builder = "builtin:fetchurl"`
- WHEN the orchestrator processes it
- THEN `do_build` is NOT called
- AND the fetch executes with network access

#### Scenario: Non-FOD builtin rejected

- GIVEN a derivation with `builder = "builtin:fetchurl"` but no
  `fixed_output`
- WHEN the orchestrator processes it
- THEN an error is returned: fetchers must be fixed-output derivations

### Requirement: Hash verification

After a fetch completes, the system MUST verify the output against
the declared hash:

- For `mode = 'flat` (fetchurl): hash the raw file bytes
- For `mode = 'recursive` (fetchTarball, fetchGit): compute the
  NAR hash of the output tree

A mismatch MUST:
1. Delete the fetched output (prevent storing bad content)
2. Report both expected and actual hash in SRI format
3. Report the `.ncl` source file and approximate location of the
   hash to update

#### Scenario: Hash matches

- GIVEN a fetchurl with `hash = "sha256-XXXX..."` and the downloaded
  file hashes to the same value
- WHEN verification runs
- THEN the fetch succeeds

#### Scenario: Hash mismatch

- GIVEN a fetchurl with `hash = "sha256-AAAA..."` but the actual
  content hashes to `sha256-BBBB...`
- WHEN verification runs
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
5. Continue the build (the fixed derivation gets a new output path)

Without `--fix`, the mismatch is reported as an error with the
suggested fix (but the file is not modified).

#### Scenario: Auto-fix updates hash

- GIVEN `hello.ncl` with `hash = "sha256-AAAA..."` and actual is
  `sha256-BBBB...`
- WHEN `crunch build --fix hello.ncl` runs
- THEN `hello.ncl` is updated: `sha256-AAAA...` → `sha256-BBBB...`
- AND the build continues with the new hash

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

Fetcher execution MUST happen outside the build sandbox. The
orchestrator MUST NOT pass fetch derivations to `BuildService`.

Regular (non-fetcher) derivations MUST NOT have network access.
The sandbox MUST continue to block network for all non-builtin
builders.

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
