# Binary Cache Substitution Specification

## Purpose

Defines how crunch fetches pre-built outputs from remote Nix binary
caches, avoiding local builds when cached results are available.

## Requirements

### Requirement: Remote Cache Fallback

The build pipeline MUST query a remote binary cache when a derivation's
output is not found in the local PathInfo database.

#### Scenario: Cache hit from remote

- GIVEN a derivation `hello` with output path digest `D`
- AND the local PathInfo database has no entry for `D`
- AND `https://cache.nixos.org/D.narinfo` returns a valid narinfo
- WHEN `check_cache` runs for `hello`
- THEN the NAR is downloaded, decompressed, and ingested into castore
- AND a PathInfo entry is written to the local redb database
- AND the build is skipped (output marked as cached)

#### Scenario: Remote miss falls through to build

- GIVEN a derivation `mypkg` with output path digest `D`
- AND the local PathInfo database has no entry for `D`
- AND the remote cache returns 404 for `D.narinfo`
- WHEN `check_cache` runs for `mypkg`
- THEN the derivation is built locally as normal

#### Scenario: Subsequent build uses local cache

- GIVEN `hello` was previously substituted from a remote cache
- AND its PathInfo exists in the local redb database
- WHEN `check_cache` runs for `hello` again
- THEN the local PathInfo is used (no network request)

### Requirement: FOD Substitution Skip

The system MUST NOT attempt remote substitution for fixed-output
derivations (derivations where any output has a `ca_hash`).

#### Scenario: FOD bypasses remote cache

- GIVEN a fetchurl derivation with `fixed_output.hash = "sha256-..."`
- WHEN the build pipeline processes this derivation
- THEN no `.narinfo` request is made to the remote cache
- AND the derivation is built via the fetcher pipeline

### Requirement: Substitution Disable Flag

The CLI MUST provide a `--no-substitute` flag that disables all
remote cache lookups.

#### Scenario: No-substitute flag

- GIVEN `--no-substitute` is passed on the command line
- WHEN any derivation is processed
- THEN no remote cache requests are made
- AND all derivations are built locally

### Requirement: Substituter Configuration

The CLI MUST accept a `--substituters` flag specifying one or more
cache URLs.

#### Scenario: Custom substituter

- GIVEN `--substituters https://my-cache.example.com`
- WHEN a derivation output is not in local cache
- THEN `https://my-cache.example.com/<digest>.narinfo` is queried

#### Scenario: Default substituter

- GIVEN no `--substituters` flag is provided
- AND `--no-substitute` is not provided
- WHEN a derivation output is not in local cache
- THEN `https://cache.nixos.org/<digest>.narinfo` is queried

### Requirement: Remote Failure Graceful Degradation

Remote cache failures (network errors, malformed narinfo, NAR
download failures) MUST be treated as cache misses, not build
failures.

#### Scenario: Network error during substitution

- GIVEN the remote cache is unreachable
- WHEN `check_cache` queries the remote
- THEN a warning is logged
- AND the derivation is built locally

### Requirement: CA Derivation Substitution Skip

The system MUST NOT attempt remote substitution for content-addressed
derivations whose output paths are not yet known (output.path is None).

#### Scenario: CA derivation not substituted

- GIVEN a CA derivation `ca-hello` with `output.path = None`
- WHEN `check_cache` runs
- THEN no remote cache request is made for this output
- AND the derivation proceeds to build
