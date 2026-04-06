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

The build pipeline uses Nickel's JSON export to produce a JSON string,
then deserializes into typed Rust structs via `serde_json::from_value()`.
This JSON round-trip is intentional: `Expr::to_serde()` fails on Nickel
enum tags in nested derivation inputs (e.g., an `Input::Derivation` whose
`system` field is `'x86_64-linux`). Nickel's JSON export pipeline converts
enum variants to plain strings, which `serde_json` handles correctly.

`crunch eval <file>` also outputs this JSON for human-readable debug output.

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
- THEN the exported JSON contains `name`, `builder`, `system`
  (no `_internal`), and the glue layer deserializes it into
  a `CrunchDerivation` Rust struct via `serde_json`

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

The crunch stdlib's `Derivation` contract is an open record contract
(see nickel-stdlib spec). Missing required fields and type mismatches
are Nickel contract violations. Extra fields pass through and are
ignored by the Rust glue layer.

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

### Requirement: Value passing via JSON export

crunch-eval MUST evaluate the Nickel expression in-process and export
the result to a JSON string via Nickel's export pipeline. The JSON
string is then deserialized into typed Rust structs (`CrunchDerivation`)
via `serde_json`. No intermediate files are written — the JSON exists
only as an in-memory `String`.

The glue layer defines `#[derive(serde::Deserialize)]` Rust structs that
mirror the Nickel Derivation contract. Deserialization handles
objects → structs, arrays → Vec, strings → enums (via `NickelString`
wrapper), nullable → Option.

The JSON round-trip exists because Nickel's `Expr::to_serde()` does not
convert enum tags to strings for fields typed as `String` on the Rust
side. The `NickelString` serde wrapper in crunch-glue accepts both plain
strings and enum-tag representations via `deserialize_any`, but nested
derivation inputs (where a full `CrunchDerivation` appears inside an
`Input` array) still fail through `to_serde()`. The JSON export path
handles all cases correctly.

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
