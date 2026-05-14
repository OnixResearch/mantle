# Store Directory Threading Specification

## Purpose

Defines the requirements for threading a configurable store directory
through the full pipeline so that `--store <path>` actually works.

## Requirements

### Requirement: convert() accepts store_dir

`crunch_glue::convert()` MUST accept a `store_dir: &str` parameter.
All internal calls that produce absolute store path strings MUST use
this parameter instead of the `STORE_DIR` constant.

Affected call sites:
- `nix_compat::Derivation::calculate_output_paths` — needs a
  variant or wrapper that embeds `store_dir` in the fingerprint
- `nix_compat::Derivation::calculate_derivation_path` — same
- `StorePath::to_absolute_path()` calls — replaced with
  `to_absolute_path_with_prefix(store_dir)`
- `build_text_path` / `build_ca_path` — these call
  `build_store_path_from_fingerprint_parts` which already
  delegates to `_with_store_dir`; the default must be overridable

#### Scenario: Non-default store dir produces different paths

- GIVEN a derivation with name "hello", builder "/bin/sh"
- WHEN converted with `store_dir = "/opt/mantle"`
- THEN the drv path starts with `/opt/mantle/` and differs from
  the `/nix/store/` path for the same derivation

#### Scenario: Default store dir matches current behavior

- GIVEN the same derivation
- WHEN converted with `store_dir = "/nix/store"`
- THEN the paths are identical to v0 output

### Requirement: KnownPaths is store-dir-aware

`KnownPaths` MUST store the configured `store_dir` and use it when
serializing paths for lookup keys. `get_by_drv_path` and
`get_hdm_by_drv_path` MUST accept paths in the configured prefix.

#### Scenario: Lookup with custom prefix

- GIVEN a KnownPaths with `store_dir = "/opt/mantle"`
- WHEN a derivation is inserted and looked up by its absolute
  drv path
- THEN the lookup succeeds using the `/opt/crunch/...` path

### Requirement: Builder uses store_dir for filesystem checks

Builder filesystem checks MUST resolve absolute paths using the configured
`store_dir`, not the constant. This includes `Builder::all_outputs_exist()`,
`path_exists_on_disk()`, `ensure_input_nodes()`, and `load_cached_outputs()`.

#### Scenario: Cache check in custom store

- GIVEN `--store /tmp/test-store` and an output that exists at
  `/tmp/test-store/<hash>-hello`
- WHEN the builder checks for a cache hit
- THEN it finds the output and skips the build

#### Scenario: Cache check does not look in /nix/store

- GIVEN `--store /tmp/test-store` and an output that exists only
  at `/nix/store/<hash>-hello`
- WHEN the builder checks for a cache hit
- THEN it does NOT find the output (wrong store dir)

### Requirement: Sandbox env uses store_dir

`derivation_to_build_request()` MUST accept a `store_dir` parameter.
The `NIX_STORE` environment variable inside the sandbox MUST be set
to the configured store dir. The `inputs_dir` field of the
`BuildRequest` MUST be derived from the configured store dir (strip
the leading `/`).

#### Scenario: NIX_STORE in sandbox

- GIVEN `store_dir = "/opt/mantle"`
- WHEN a BuildRequest is constructed
- THEN `NIX_STORE` is `/opt/mantle` in the sandbox env
- AND `inputs_dir` is `opt/mantle`

### Requirement: Output path display uses store_dir

The CLI MUST print output paths using the configured store dir.
`to_absolute_path()` calls in `main.rs` output formatting MUST
use `to_absolute_path_with_prefix(store_dir)`.

#### Scenario: Build output with custom store

- GIVEN `mantle build --store /opt/mantle hello.ncl`
- WHEN the build succeeds
- THEN stdout shows `/opt/mantle/<hash>-hello`

### Requirement: nix-compat path computation with store_dir

The vendored `nix-compat` MUST support store-dir-aware path computation
at the `Derivation` level. Either:

(a) `calculate_derivation_path` and `calculate_output_paths` gain a
    `store_dir` parameter, or
(b) a parallel set of `_with_store_dir` methods is added, or
(c) the existing methods are changed to accept an optional store dir

The store dir MUST be embedded in the fingerprint hash (this is already
the case in `build_store_path_from_fingerprint_parts_with_store_dir`).

#### Scenario: Derivation path depends on store dir

- GIVEN the same derivation ATerm
- WHEN `calculate_derivation_path` is called with two different
  store dirs
- THEN the resulting paths differ (different fingerprint)

### Requirement: Backward compatibility of StorePath struct

`StorePath` itself MUST NOT store the prefix — it remains a
(digest, name) pair. The prefix is applied at serialization time
via `to_absolute_path_with_prefix()`. This keeps `StorePath`
lightweight and avoids changing its layout.

#### Scenario: StorePath serializes with caller prefix

- GIVEN a `StorePath` value and two different store-dir prefixes
- WHEN each prefix is passed to `to_absolute_path_with_prefix()`
- THEN the serialized paths differ only by prefix while the digest and name
  remain unchanged

### Requirement: Tests use tempdir stores

Test coverage MUST avoid requiring write access to `/nix/store`. Unit and
integration tests that check filesystem behavior SHOULD use a temporary
directory as the store dir.
Tests that assert exact path strings MUST be parameterized or duplicated for
the default store dir.

#### Scenario: Cache test without /nix/store

- GIVEN a Builder configured with `store_dir = <tempdir>`
- WHEN an output file is created in `<tempdir>/<hash>-name`
- THEN the cache check succeeds without needing /nix/store access
