# Error Model

## Overview

Shared error taxonomy and propagation policy across all pipeline layers.
Errors are structured, carry provenance (which layer, which module),
and follow consistent propagation rules.

## Requirements

### ERR-1: Error type hierarchy

All pipeline errors MUST implement a common `SystemConfigError` enum
with variants per layer:

- `Loader { module_name, detail }` — structural validation failures.
- `Inventory { field_path, detail }` — inventory schema violations.
- `CrossRef { module_name, instance, detail }` — inventory-module
  cross-reference mismatches (EVAL-12).
- `Eval { module_name, detail }` — Nickel evaluation failures including
  contract blame, timeout, and memory limit.
- `Fragment { machine_name, path, detail }` — merge conflicts.
- `Assembler { machine_name, backend, detail }` — derivation assembly
  failures.

Each variant MUST carry enough context for a human-readable diagnostic
without re-reading source files.

### ERR-2: Propagation policy

- **Loader:** fail-open. A module that fails structural validation is
  excluded from evaluation. All failures are collected and reported.
  Modules that depend on a failed module also fail (with a chained
  diagnostic).
- **Inventory:** fail-closed. If the inventory fails schema validation,
  the pipeline stops. No modules are evaluated.
- **Evaluator:** fail-open per independent subgraph (EVAL-9). A failed
  module does not prevent evaluation of modules that do not depend on it.
- **Fragment collector:** fail-open per machine. A merge conflict on
  machine A does not prevent collection for machine B.
- **Assembler:** fail-open per machine. A failed assembly for machine A
  does not prevent assembly of machine B.

### ERR-3: Structured output

When `--json` is active, errors MUST be emitted as JSON objects on
stderr with fields: `layer`, `module` (if applicable), `machine`
(if applicable), `message`, and `detail`. Without `--json`, errors
are human-readable text on stderr.

### ERR-4: Error collection type

The pipeline's top-level return type MUST be a `SystemPipelineResult`
struct carrying both per-machine outcomes and collected non-fatal
errors:

```
struct SystemPipelineResult {
    machines: BTreeMap<String, MachineOutcome>,
    errors: Vec<SystemConfigError>,
}
```

Partial success (some machines built, others failed) is represented
by successful entries in `machines` alongside errors in `errors`.
Inventory-level failures (ERR-2 fail-closed) are returned as
`Err(Vec<SystemConfigError>)` wrapping the result in a top-level
`Result`.
