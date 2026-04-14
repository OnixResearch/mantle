# Nickel Evaluation Specification

## Purpose

Defines how crunch evaluates Nickel source files and extracts derivation
descriptions from the result. The evaluation relies on Nickel's contract
system, merge semantics, and export pipeline — not custom parsing.

## MODIFIED Requirements

### Requirement: Nickel evaluation via eval_full_for_export

The system MUST use `nickel-lang-core` to parse, typecheck, and evaluate
`.ncl` files. Evaluation MUST use the export-ready deep evaluation path
(`eval_full_for_export` or an equivalent API) which:

1. fully reduces the expression,
2. strips `not_exported` fields,
3. validates contracts,
4. leaves the evaluated value available for direct typed extraction or JSON
   rendering, depending on the caller.

Build execution MUST consume derivation-shaped results through a direct typed
extraction path. `crunch eval <file>` MAY still render the evaluated result
through Nickel's JSON export for operator-visible output.

#### Scenario: Evaluate and deserialize derivation directly

- GIVEN a `.ncl` file containing:
  ```nickel
  {
    name = "hello",
    builder = "/bin/sh",
    system = 'x86_64-linux,
    _internal | not_exported = "stripped",
  }
  ```
- WHEN crunch-eval evaluates it for build execution
- THEN the evaluated expression deserializes directly into a
  `CrunchDerivation`
- AND `_internal` is absent from the resulting Rust value

## ADDED Requirements

### Requirement: Direct derivation extraction supports nested enum-tag fields

The system MUST provide a direct typed extraction path for
`CrunchDerivation`-shaped Rust structs that accepts Nickel enum tags in both
top-level and nested derivation fields without first converting the whole
evaluation result to JSON.

#### Scenario: Nested derivation input uses enum tags

- GIVEN a package set where an `Input::Derivation` record contains
  `system = 'x86_64-linux` and `addressing_mode = 'content-addressed`
- WHEN crunch-eval deserializes the evaluated expression for the build
  pipeline
- THEN the nested derivation is accepted as a typed Rust value
- AND the pipeline does not need a JSON export string to normalize those tags

## REMOVED Requirements

### Requirement: Value passing via JSON export

The build pipeline MUST NOT require a whole-program JSON export as the only
supported way to obtain derivation records from evaluated Nickel.
