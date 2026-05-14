# Castore-Only Store Specification

## Purpose

Defines how mantle uses the snix castore as its primary store, removing
the requirement for a writable filesystem store directory.

## Requirements

### Requirement: Castore-Based Cache Validation

The system MUST validate build cache hits using PathInfo records and
castore content probes, NOT filesystem existence checks.

#### Scenario: Cache hit with castore content, no file on disk

- GIVEN a previous build produced PathInfo for output `foo` in redb
- AND the output's blob exists in the castore blob service
- AND no file exists at `output_dir/nix/store/foo`
- WHEN `check_cache` runs for the derivation
- THEN it returns a cache hit with the PathInfo
- AND `output_nodes` is populated with the PathInfo's node

#### Scenario: Cache miss when castore content missing

- GIVEN PathInfo exists for output `foo` in redb
- AND the blob referenced by PathInfo.node does NOT exist in the blob service
- WHEN `check_cache` runs
- THEN it returns a cache miss
- AND a warning is logged about missing castore content

#### Scenario: Cache miss when no PathInfo

- GIVEN no PathInfo exists for the output
- WHEN `check_cache` runs
- THEN it returns a cache miss

### Requirement: Root-Only Disk Export

The system MUST export build outputs to disk only for top-level
derivations explicitly requested by the user. Intermediate dependency
outputs MUST remain in the castore only.

#### Scenario: Intermediate dep stays in castore

- GIVEN derivation `app` depends on `libfoo`
- AND the user runs `mantle build app.ncl`
- WHEN `libfoo` builds successfully
- THEN `libfoo`'s output is in the castore (blob + PathInfo)
- AND `libfoo`'s output is NOT written to `output_dir`

#### Scenario: Root derivation exported to disk

- GIVEN the user runs `mantle build app.ncl`
- WHEN `app` builds successfully
- THEN `app`'s output is written to `output_dir`
- AND `app`'s output is in the castore

#### Scenario: Root export failure is non-fatal

- GIVEN `output_dir` is read-only
- WHEN a root derivation build completes
- THEN the build reports success
- AND a warning indicates the output was not exported to disk
- AND the output remains available in the castore

### Requirement: Output Nodes Populated from Cache

When a cache hit occurs, the system MUST populate `output_nodes` from
the PathInfo record so that downstream builds can access the dependency
through the castore without disk access.

#### Scenario: Cached dep available to downstream build

- GIVEN `libfoo` was built in a previous session and cached (PathInfo + castore)
- AND `libfoo` was NOT exported to disk
- WHEN `app` (which depends on `libfoo`) is built
- THEN `collect_sandbox_inputs` finds `libfoo` in `output_nodes`
- AND `libfoo` is mounted in `app`'s sandbox from castore content

### Requirement: Source Input Ingestion Unchanged

Source inputs (seed packages from `/nix/store/`) MUST still be read
from the host filesystem and ingested into the castore on first use.
Only read access to `/nix/store/` is required.

#### Scenario: Seed package ingested on first use

- GIVEN seed package `bash` exists at `/nix/store/xxx-bash`
- AND `bash` is not yet in the castore
- WHEN a derivation lists `bash` as an input
- THEN `bash` is ingested into the blob + directory services
- AND subsequent builds find `bash` in `output_nodes` without disk access
