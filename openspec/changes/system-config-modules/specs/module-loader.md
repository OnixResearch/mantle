# Module Loader

## Overview

Discovers and loads Nickel service module files from a directory into typed
in-memory records. Pure function with filesystem discovery factored out.

## Requirements

### LOADER-1: Directory scanning

The loader MUST accept a directory path and return all `*.ncl` files found
at the top level of that directory. Subdirectories are not recursed.

### LOADER-2: Structural validation

Each loaded module MUST be validated against a structural contract:
- Has an `interface` field (record).
- `interface` has a `roles` field (record, keyed by role name).
- Has an `impl` field (function).
- Optionally has `inputs` (array of strings naming upstream modules).
- Optionally has `consumes_providers` (array of provider-type strings).
- Optionally has `produces_providers` (array of provider-type strings).
- Optionally has `priority` (number for evaluation ordering tiebreaks,
  default 1000 when absent).

The loader does NOT validate the contents of `roles` records or
provider output shapes — that is deferred to module evaluation where
Nickel contracts enforce domain-specific constraints.

Modules failing structural validation MUST produce a diagnostic error
naming the file and the missing/malformed field.

### LOADER-3: Module identity

Each module MUST be assigned a stable identity derived from its filename
(stem without `.ncl` extension). Two modules with the same stem MUST be
rejected with an error.

### LOADER-4: Pure core split

The loader MUST separate filesystem I/O (reading directory entries,
reading file contents) from validation logic. The validation function
MUST accept a module name and an evaluator handle (providing field
access and type introspection via `get_field` and `is_function`) and
return `Result<ValidatedModule, LoaderError>` with no filesystem I/O.
The validation function communicates with the evaluator thread via
`ValueId` handles, not raw `NickelValue` references.

### LOADER-5: Fixed limits

The loader MUST enforce a maximum module count (configurable, default
1024). Exceeding the limit MUST fail fast before evaluation begins.
