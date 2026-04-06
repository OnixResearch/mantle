# Architecture Specification

## Purpose

Defines the crate structure, data flow, and integration boundaries for crunch.

## Requirements

### Requirement: Pipeline stages

The system MUST implement the following pipeline:

1. Nickel source → evaluated record (via nickel-lang-core)
2. Evaluated record → `nix_compat::Derivation` struct (via glue)
3. Derivation → `BuildRequest` (via vendored snix-build builder logic)
4. BuildRequest → sandboxed execution → `BuildResult` (via snix-build)
5. BuildResult → `PathInfo` persisted in store (via snix-store)

Each stage MUST be a separate module/crate with a defined interface between
them.

#### Scenario: Trivial build

- GIVEN a `.ncl` file describing a derivation with name "hello", builder
  "/bin/sh", and a simple build script
- WHEN `crunch build hello.ncl` is invoked
- THEN the Nickel file is evaluated, a Derivation is constructed, the build
  runs in a sandbox, and the output is persisted in the store

### Requirement: Crate layout

The workspace MUST contain the following crates:

| Crate | Role |
|---|---|
| `crunch` (binary) | CLI entry point, wires everything together |
| `crunch-eval` | Nickel evaluation wrapper — takes a file path, returns structured JSON |
| `crunch-glue` | Converts evaluated Nickel records to Derivation structs, manages KnownPaths |
| `crunch-build` | Goal scheduler (Worker), build dispatch, output processing, CA rewriting |
| `nix-compat` (vendored) | Derivation struct, store path calculation, ATerm, NAR, nixbase32 |
| `snix-build` (vendored) | BuildService trait, BuildRequest, sandbox execution |
| `snix-castore` (vendored) | Content-addressed blob and directory storage |
| `snix-store` (vendored) | PathInfoService, NAR calculation, store construction |

Additional vendored support crates (snix-tracing, snix-serde, nix-compat-derive)
MAY be included as needed.

#### Scenario: Workspace compiles

- GIVEN all crates are present in `crunch/`
- WHEN `cargo build` is run from `crunch/`
- THEN the workspace compiles without errors

### Requirement: No Nix evaluation

The system MUST NOT include or invoke a Nix language evaluator at any point
in the build pipeline. The `snix-eval` crate MUST NOT be a dependency.

#### Scenario: No snix-eval dependency

- GIVEN the crunch workspace
- WHEN `cargo tree` is inspected
- THEN `snix-eval` does not appear in the dependency tree

### Requirement: Nickel as sole input language

The system MUST accept `.ncl` files as the sole input format for describing
derivations. No `.nix` files are processed.

#### Scenario: Evaluate a Nickel file

- GIVEN a valid `.ncl` file exporting a record matching the Derivation contract
- WHEN passed to crunch-eval
- THEN a JSON representation of the derivation is returned

### Requirement: Store path determinism (not Nix compatibility)

Derivations constructed by crunch MUST produce deterministic store paths—
the same derivation parameters MUST always yield the same path. However,
crunch store paths intentionally DIFFER from Nix/snix store paths because
crunch uses BLAKE3 for derivation hashing instead of SHA-256.

Pre-existing store paths from a Nix installation (seed toolchain) are
referenced as-is via `Input::Source`. crunch does not re-hash them.

#### Scenario: Deterministic paths

- GIVEN the same derivation parameters
- WHEN crunch computes the output path twice
- THEN both paths are identical

#### Scenario: Differs from Nix

- GIVEN the same derivation parameters
- WHEN crunch and Nix each compute the output path
- THEN the paths differ (BLAKE3 vs SHA-256)

## Requirements: Vendoring

### Requirement: Vendor from snix

Code MUST be vendored from `../snix/snix/` into `crunch/vendor/` (or as
workspace members). Vendored crates MAY be modified — they are not kept in
sync with upstream snix.

### Requirement: Minimal vendoring

Only crates required for the build pipeline MUST be vendored. `snix-eval`,
`snix-glue`, and snix CLI crates MUST NOT be vendored — their functionality
is replaced by crunch-eval and crunch-glue.

#### Scenario: Required crates vendored

- GIVEN the crunch workspace
- WHEN the dependency graph is inspected
- THEN it includes nix-compat, snix-build, snix-castore, snix-store and their
  necessary support crates, but not snix-eval or snix-glue

## Requirements: Modularity

### Requirement: Core is a build engine, not a build framework

crunch MUST be a minimal build engine: evaluate Nickel, construct derivations,
execute builds, persist results. It MUST NOT bundle:

- Builder templates or convenience wrappers (bash builder, mkDerivation)
- Build phase abstractions (unpack/configure/build/install)
- Input set helpers or stdenv equivalents
- A module system (NixOS-style or otherwise)
- Package set definitions or override/overlay mechanisms

These are separate concerns that belong in independent Nickel packages
layered on top of crunch. crunch ships the derivation schema; external
packages ship the opinions.

#### Scenario: No builder logic in core

- GIVEN the crunch Nickel stdlib
- WHEN inspected
- THEN it contains only the Derivation contract, enum types, validators,
  and conversion helpers — no shell script templates, no build phase names,
  no references to specific tools

#### Scenario: External builder package

- GIVEN a hypothetical `crunch-builders` Nickel package that imports
  crunch's stdlib
- WHEN it defines a `bash_builder` partial record with merge defaults
- THEN it works without any changes to crunch itself

### Requirement: Crate boundaries are stable interfaces

Each crate MUST expose a minimal, documented public API. Crate boundaries
are the integration points where external tools or alternative
implementations can plug in:

- `crunch-eval` can be swapped for a different evaluator (e.g., a future
  Nickel version, or a different config language entirely)
- `crunch-glue` can be reused by tools that produce derivation JSON from
  sources other than Nickel
- Vendored snix crates can be replaced with upstream snix if APIs converge

#### Scenario: Alternative evaluator

- GIVEN a tool that produces JSON matching the Derivation schema
- WHEN fed to crunch-glue
- THEN it works identically to Nickel-produced JSON
