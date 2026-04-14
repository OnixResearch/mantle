# Pipeline Crate Specification

## Purpose

Defines the `crunch-pipeline` crate: the integration layer that wires
eval, convert, and build into a single callable function.

## MODIFIED Requirements

### Requirement: Pipeline owns eval and convert

The pipeline MUST evaluate Nickel in-process and extract derivation data
through direct typed deserialization for build execution. It MUST NOT
require a whole-program JSON export merely to obtain `CrunchDerivation`
values for `build()`.

`evaluate_to_json()` remains available for `crunch eval` and other debug or
reporting paths, but the build path MUST use the direct typed representation
that `crunch-eval` exposes.

The pipeline MUST still run `crunch_glue::convert()` for each derivation and
bridge the `ConversionCache` to a `DerivationRegistry`.

#### Scenario: Direct single-derivation extraction

- GIVEN a `.ncl` file describing one derivation
- WHEN `build()` is called
- THEN the pipeline deserializes the evaluated Nickel expression directly into
  a `CrunchDerivation`
- AND no JSON export string is required before `convert()` runs

#### Scenario: Direct package-set extraction with nested enum tags

- GIVEN a `.ncl` file exporting a record of derivations whose nested inputs
  contain Nickel enum tags
- WHEN `build()` is called
- THEN the pipeline deserializes the package set directly into typed Rust
  derivation values
- AND `convert()` receives the same derivation graph without a JSON round-trip
