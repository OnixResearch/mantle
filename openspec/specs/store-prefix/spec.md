# Configurable Store Prefix Specification

## Purpose

Defines how mantle computes, stores, and references store paths using a
configurable prefix instead of the hardcoded `/nix/store`.

## Requirements

### Requirement: Store prefix is configurable at startup

The system MUST accept a `--store-prefix` CLI flag that sets the logical
store path prefix for all derivation hash computations, path construction,
and sandbox environment variables. Default: `/mantle/store`.

#### Scenario: Default prefix

- GIVEN no `--store-prefix` flag
- WHEN mantle evaluates and builds a derivation
- THEN all store paths use the `/mantle/store` prefix
- THEN the sandbox `NIX_STORE` env var is set to `/mantle/store`
- THEN output paths printed to stdout begin with the `--store` directory

#### Scenario: Nix-compat prefix

- GIVEN `--nix-compat` flag
- WHEN mantle evaluates and builds a derivation
- THEN all store paths use the `/nix/store` prefix
- THEN derivation hashes match prior mantle behavior

#### Scenario: Custom prefix

- GIVEN `--store-prefix /opt/mystore`
- WHEN mantle evaluates and builds a derivation
- THEN all store paths use the `/opt/mystore` prefix

### Requirement: Prefix flows through all hash computations

The store prefix MUST be used consistently in:

1. `StorePath::to_absolute_path()` output
2. Derivation ATerm serialization (output paths, input paths)
3. Hash derivation modulo computation
4. Content-addressed output path computation
5. BuildRequest output path construction
6. Sandbox `NIX_STORE` environment variable (set to the configured prefix)
7. ConversionCache and DerivationRegistry construction
8. PathInfo store path references

#### Scenario: Same derivation, different prefix yields different hash

- GIVEN a derivation spec `{ name = "hello", builder = "/bin/sh", ... }`
- WHEN built with `--store-prefix /crunch/store`
- THEN the output path hash differs from the same spec built with `--store-prefix /nix/store`

### Requirement: Prefix length must be consistent

The store prefix MUST have the same byte length for all derivations within
a single build invocation. The system SHOULD reject prefixes shorter than
8 bytes or longer than 128 bytes.

#### Scenario: Reject empty prefix

- GIVEN `--store-prefix ""`
- WHEN mantle starts
- THEN it exits with an error before any evaluation

### Requirement: nix_compat STORE_DIR patched

The vendored `nix_compat` crate MUST NOT use a compile-time `STORE_DIR`
constant for path computation. All functions that reference the store
directory MUST accept it as a parameter or read it from a thread-local /
context object.

Functions affected:
- `store_path::build_store_path_from_fingerprint_parts`
- `store_path::StorePath::to_absolute_path`
- `derivation::hash_derivation_modulo` (env references)
- `derivation::output_path` computations

#### Scenario: Constant use is rejected in path computation

- GIVEN a vendored `nix_compat` path-computation helper
- WHEN it needs to serialize or hash an absolute store path
- THEN the caller-provided store prefix is used instead of a compile-time
  `STORE_DIR` constant

### Requirement: Backward compatibility flag

The `--nix-compat` flag MUST be a shorthand for `--store-prefix /nix/store`.
It MUST NOT change any other behavior.

#### Scenario: nix-compat flag only changes prefix

- GIVEN `--nix-compat` is passed without any other behavior flag
- WHEN mantle initializes its store-prefix configuration
- THEN the logical prefix is `/nix/store`
- AND every other configuration value matches the ordinary defaults
