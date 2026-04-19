# Nickel Evaluation Specification

## Purpose

Defines how crunch evaluates Nickel source files and extracts derivation
descriptions from the result. The evaluation relies on Nickel's contract
system, merge semantics, and export pipeline — not custom parsing.
## Requirements
### Requirement: Nickel evaluation via eval_full_for_export

The system MUST provide a stateful lazy session boundary that retains Nickel
program state across at least two operations:

1. a lazy root-discovery path that evaluates the top-level Nickel result only
   enough to determine whether it is a single derivation, an array of
   derivations, or a record of derivations,
2. a selected-root forcing path that fully evaluates only the requested root
   value for typed extraction or export, and
3. a bounded multi-root forcing path for callers that need more than one root
   without serially deep-exporting every root up front.

Build and planning execution MUST use the lazy discovery path before any
per-root forcing. Whole-program export-ready deep evaluation MAY still be used
for operator-visible `crunch eval` output or other callers that explicitly need
whole-program export semantics.

The multi-root forcing path MUST preserve the caller's requested label order in
its returned results.

If the caller omits an explicit concurrency cap, the multi-root forcing path
MUST default to a cap of `1`.

For any given requested label, the multi-root forcing path MUST return the same
typed derivation value as the serial same-session forcing path for that label.

If one requested root fails during forcing, the surfaced `crunch-eval` error
MUST identify the failed label. The batch API MUST fail the whole request on
that first labeled error rather than returning a partial success vector. That
atomicity applies per batch request. Callers that need incremental delivery
MUST issue smaller requests, including single-label requests.

#### Scenario: Batch forcing preserves requested root order

- GIVEN a `.ncl` file exporting a record of named derivations
- WHEN crunch-eval forces a selected batch of root labels through the multi-root
  lazy session API
- THEN the returned typed derivations preserve the caller's requested label
  order
- AND each returned derivation remains associated with the same label that was
  requested

#### Scenario: Failed root reports its label during batch forcing

- GIVEN a `.ncl` file exporting named derivations
- AND one requested root fails during forcing
- WHEN crunch-eval runs the bounded multi-root forcing path
- THEN it returns a `crunch-eval` error
- AND the error identifies the failed root label
- AND it does not return a partial success vector for the same batch request

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

### Requirement: Root discovery supports multi-derivation outputs lazily

The system MUST discover top-level root identities for single-derivation,
array, and record outputs before eagerly deserializing every root value.

The discovery result MUST preserve the current root-label semantics:
- a single derivation uses its `name` field as the label
- an array of derivations uses each derivation's `name` field as the label
- a record of derivations uses each field name as the label

#### Scenario: Array labels come from derivation names without full sibling extraction

- GIVEN a `.ncl` file evaluating to an array of derivation records
- WHEN crunch-eval lists root labels lazily
- THEN each label comes from the corresponding derivation `name`
- AND accessing those `name` fields is the maximum acceptable partial forcing
  for discovery in that array case
- AND crunch-eval does not need to deserialize every sibling into a final Rust
  value before returning the label list

#### Scenario: Record labels come from top-level field names

- GIVEN a `.ncl` file evaluating to a record of derivations
- WHEN crunch-eval lists root labels lazily
- THEN each label is the corresponding top-level field name
- AND individual root values may still be forced later on demand

### Requirement: Lazy session APIs return typed errors instead of panicking

The `crunch-eval` lazy session API MUST return `crunch-eval::Error` values for
malformed or inconsistent top-level and selected-root shapes that can arise
from evaluated input or stale session state.

Public lazy-eval methods MUST NOT rely on `expect(...)` or `unwrap(...)` for
user-reachable shape checks.

Internal impossible-state assertions MAY remain as private invariants only
after user-reachable failures have already been converted into typed errors.

#### Scenario: Selected-root forcing rejects inconsistent shape without panic

- GIVEN a lazy-eval session whose selected-root extraction encounters a shape
  inconsistent with the discovered top-level structure
- WHEN a caller forces that root through the public lazy session API
- THEN the method returns a `crunch-eval::Error`
- AND the process does not panic

#### Scenario: Root discovery rejects malformed record or array shape without panic

- GIVEN a lazy-eval session whose discovery path encounters a malformed record
  or array structure for the current root shape
- WHEN a caller asks for root labels through the public API
- THEN the method returns a `crunch-eval::Error`
- AND the process does not panic

### Requirement: Parallel multi-root forcing uses only bounded safe concurrency

The system MUST separate multi-root forcing semantics from the concrete local
execution backend used to run root-force assignments.

The `crunch-eval` core MUST provide one backend-neutral bounded multi-root
forcing interface whose semantics stay the same across inline and any shipped
non-inline backends.

The `crunch-eval` core MUST provide a serial inline backend that can execute a
bounded multi-root request without background worker threads, subprocesses,
daemon lifecycle, or binary self-spawn.

Any shipped threaded or subprocess backend MUST remain optional and MUST
preserve the same typed derivation values, requested-label ordering, and
labeled failure semantics as the serial inline backend for the same requested
roots.

Any backend that evaluates more than one root concurrently MUST still use
bounded concurrency and MUST drive only immutable worker input across the
concurrency boundary unless stronger compile-time safety evidence exists for the
actual concurrently used evaluator state.

#### Scenario: Portable inline backend forces multiple roots without host worker helpers

- GIVEN a multi-root forcing request on a host or embedding target that does
  not want background worker threads or subprocesses
- WHEN `crunch-eval` materializes that request through its required inline
  backend
- THEN it returns the requested typed derivation values
- AND it preserves the requested label order
- AND it does not require a daemon, global worker pool, or binary self-spawn

#### Scenario: Optional non-inline backend matches inline semantics

- GIVEN the same source input and requested root labels
- WHEN `crunch-eval` forces them through the serial inline backend and through
  any shipped threaded or subprocess backend
- THEN both paths produce the same typed derivation values for each label
- AND both paths preserve the same requested-label ordering
- AND a failure in either path identifies the same failed label

