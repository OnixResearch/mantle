## Why

Multi-output derivations landed but consuming derivations can't reference
individual outputs. Every dependency mounts *all* outputs in the sandbox.
A package that splits into `out`, `dev`, `lib` forces every consumer to
carry all three — no way to say "I only need headers."

This blocks real stdenv-style packaging. In Nix, `pkg.dev` is the
fundamental mechanism for splitting build-time from runtime deps. Without
it, multi-output is decoration.

## What Changes

- **Nickel `Input` contract**: add a structured variant for output
  selection — `{ drv = some_pkg, output = "dev" }`. A bare derivation
  record in `inputs` keeps the current behavior (all outputs).
- **Rust `Input` enum**: new `OutputSelection` variant carrying the
  derivation plus the selected output name(s).
- **Glue layer `resolve_inputs`**: route the new variant into
  `input_derivations` with only the selected output names instead of
  all of them.
- **Nickel helper**: `select_output` function on derivations or in the
  stdlib — `crunch.select dep_pkg "dev"` or `dep_pkg |> crunch.select "dev"`.

The Rust build layer (`collect_input_paths`, `collect_sandbox_inputs`,
bwrap mounting) already filters by `output_names` in `input_derivations`.
No changes needed there — the data just needs to flow through correctly
from the Nickel/glue layer.

## Capabilities

### New Capabilities
- `output-selection`: consuming derivations can depend on specific
  outputs of multi-output dependencies.
- `select-helper`: stdlib function for ergonomic output selection.

### Modified Capabilities
- `input-handling`: `Input` contract and Rust enum gain a third variant.

## Impact

- **Files**: `lib/contracts.ncl` or `lib/derivation.ncl` (Input contract),
  `lib/lib.ncl` (select helper), `crates/crunch-glue/src/types.rs` (Input
  enum), `crates/crunch-glue/src/convert.rs` (resolve_inputs).
- **APIs**: `Input` enum is extended (non-breaking — existing `Source`
  and `Derivation` variants unchanged).
- **Dependencies**: none.
- **Testing**: unit tests for the new Input variant deserialization,
  convert tests verifying input_derivations carry the right output
  names, integration test building a package that uses only one output
  of a multi-output dep.
