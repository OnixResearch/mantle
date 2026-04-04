# Nickel Evaluation Specification

## Purpose

Defines how crunch evaluates Nickel source files and extracts derivation
descriptions from the result. The evaluation relies on Nickel's contract
system, merge semantics, and export pipeline — not custom parsing.

## Requirements

### Requirement: Nickel evaluation via eval_full_for_export

The system MUST use `nickel-lang-core` to parse, typecheck, and evaluate
`.ncl` files. Evaluation MUST use the export path (`eval_full_for_export`)
which:

1. Fully reduces the expression (forces all thunks)
2. Strips `not_exported` fields
3. Validates all contracts
4. Produces a fully-resolved `NickelValue`

The resulting `NickelValue` is passed directly to the glue layer. No JSON
serialization step. `NickelValue` implements `serde::Deserializer`, so the
glue layer deserializes directly into typed Rust structs via `#[derive(Deserialize)]`.

`crunch eval` MAY serialize to JSON for human-readable debug output, but
the build pipeline MUST NOT use JSON as an intermediate format.

#### Scenario: Evaluate and deserialize

- GIVEN a `.ncl` file containing:
  ```nickel
  {
    name = "hello",
    builder = "/bin/sh",
    system = 'x86_64-linux,
    _internal | not_exported = "stripped",
  }
  ```
- WHEN crunch-eval evaluates it
- THEN the returned `NickelValue` contains `name`, `builder`, `system`
  (no `_internal`), and the glue layer can deserialize it directly into
  a `CrunchDerivation` Rust struct

#### Scenario: Evaluation error with source location

- GIVEN a `.ncl` file with a type error or unresolved variable
- WHEN crunch-eval evaluates it
- THEN a structured error is returned including the source file, line, and
  column, using Nickel's error formatting

### Requirement: Contract validation at eval time

Nickel's contract system MUST serve as the primary validation layer. Errors
in derivation descriptions (missing fields, wrong types, invalid enum
variants) MUST be caught by Nickel contracts during evaluation, before the
Rust glue layer runs.

The crunch stdlib's `Derivation` contract is a closed record contract.
Missing required fields, extra unknown fields, and type mismatches are all
Nickel contract violations.

#### Scenario: Missing required field

- GIVEN `{ builder = "/bin/sh" } | crunch.Derivation`
- WHEN evaluated
- THEN Nickel reports a contract violation for missing `name`

#### Scenario: Wrong field type

- GIVEN `{ name = 42, builder = "/bin/sh" } | crunch.Derivation`
- WHEN evaluated
- THEN Nickel reports a contract violation: `name` expected `String`, got `Number`

#### Scenario: Invalid enum variant

- GIVEN `{ name = "foo", builder = "/bin/sh", system = 'PDP-11 }`
- WHEN the `System` enum contract runs
- THEN Nickel rejects it as a non-matching variant

### Requirement: Merge-based composition preserved through eval

The evaluation MUST correctly handle records produced by merging multiple
partial records. Nickel's merge semantics (recursive merge, priority
resolution, default handling) MUST be fully resolved before the result
reaches the glue layer.

#### Scenario: Merged derivation

- GIVEN:
  ```nickel
  let base = { system | default = 'x86_64-linux, outputs | default = ["out"] } in
  let specifics = { name = "hello", builder = "/bin/sh" } in
  base & specifics | crunch.Derivation
  ```
- WHEN evaluated
- THEN the result is a single flat JSON object with all fields resolved

#### Scenario: Priority override

- GIVEN:
  ```nickel
  let template = { args | default = [] } in
  template & { args | force = ["-c", "echo hi"] }
  ```
- WHEN evaluated
- THEN `args` is `["-c", "echo hi"]` (force wins)

### Requirement: Multi-derivation output

The system MUST support `.ncl` files that evaluate to either:

1. A single record matching the `Derivation` contract
2. A record of named derivation records (each value matches `Derivation`)

Detection: if the top-level record has a `name` field (String), treat it as
a single derivation. Otherwise, treat each field as a named derivation.

#### Scenario: Single derivation

- GIVEN a `.ncl` file evaluating to `{ name = "hello", builder = "...", ... }`
- WHEN processed by crunch
- THEN one derivation is built

#### Scenario: Package set

- GIVEN a `.ncl` file evaluating to:
  ```nickel
  {
    hello = { name = "hello", builder = "...", ... },
    world = { name = "world", builder = "...", inputs = [hello], ... },
  }
  ```
- WHEN processed by crunch
- THEN both derivations are built in dependency order

### Requirement: Recursive records for self-referencing derivations

Nickel records are recursive by default. The evaluation MUST support
derivations where fields reference each other:

```nickel
{
  name = "myapp",
  version = "1.0",
  env.APP_NAME = name,
  env.APP_VERSION = version,
  ...
}
```

This is evaluated before export — the glue layer sees `env.APP_NAME = "myapp"`.

#### Scenario: Self-referencing env

- GIVEN a derivation with `env.NAME = name` where `name = "hello"`
- WHEN evaluated
- THEN the exported JSON has `env.NAME = "hello"` (fully resolved)

### Requirement: Import resolution

crunch-eval MUST configure Nickel's import resolution to find:

1. The crunch stdlib (shipped with the binary or via `--import-path`)
2. Files relative to the evaluated `.ncl` file's directory
3. JSON/YAML/TOML files (Nickel auto-converts these on import)

#### Scenario: Import seed

- GIVEN `hello.ncl` containing `let seed = import "seed.ncl" in ...`
  and `seed.ncl` in the same directory
- WHEN `crunch build hello.ncl` is run
- THEN the import resolves relative to `hello.ncl`'s directory

### Requirement: Value passing is in-process, no serialization

crunch-eval MUST return `NickelValue` directly to crunch-glue in-process.
No intermediate files, no JSON serialization step. The `NickelValue` type
(which implements `serde::Deserializer`) is the contract boundary between
evaluation and derivation construction.

The glue layer defines `#[derive(serde::Deserialize)]` Rust structs that
mirror the Nickel Derivation contract. Deserialization from `NickelValue`
handles records → structs, arrays → Vec, enum variants → Rust enums,
nullable → Option automatically.

### Requirement: Explicit input declarations

Derivation records MUST declare their build inputs explicitly via an `inputs`
field. There is no implicit string context tracking. Nickel's string
interpolation (`"%{expr}"`) produces plain strings.

#### Scenario: Input declared

- GIVEN a derivation with `inputs = [dep_a, dep_b]`
- WHEN the glue layer processes it
- THEN both appear in the Derivation's `input_derivations`

#### Scenario: Undeclared reference

- GIVEN a derivation that interpolates `"%{some_drv.out}/bin/foo"` into
  `builder` but does not list `some_drv` in `inputs`
- WHEN the build runs
- THEN the sandbox blocks access to that path and the build fails
  (correct behavior — enforces hermeticity)
