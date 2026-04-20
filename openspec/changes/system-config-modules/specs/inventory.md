# Inventory Schema

## Overview

The inventory is the primary input to the system-config pipeline. It
declares machines, service instances, settings overrides, and machine
metadata. The inventory is a Nickel file evaluated by `crunch-eval`.

## Requirements

### INV-1: Machine records

The inventory MUST contain a `machines` field: a record keyed by machine
name. Each machine record MUST have:
- `system` — target system string (e.g., `"x86_64-linux"`).
- `class` — assembler backend selector (e.g., `"nixos"`, `"container"`,
  `"s6"`). Defaults to `"nixos"` when absent.

Machine records MAY contain additional metadata fields. The pipeline
passes the full machine record to the assembler; it does not strip
unknown fields.

### INV-2: Service instances

The inventory MUST contain a `services` field: a record keyed by service
name (matching module filenames without `.ncl`). Each service record
MUST contain an `instances` field: an array of instance records.

Each instance record MUST have:
- `machine` — string naming a machine from `machines`.
- `role` — string naming a role from the module's `interface.roles`.

Each instance record MAY have:
- `settings` — record of per-instance overrides merged with module
  interface defaults.
- `tags` — array of strings for grouping/filtering.

### INV-3: Cross-reference validation

The inventory contains references that can only be validated once
modules are loaded. These cross-checks are defined here for
completeness but MUST be executed by the module evaluator (EVAL-12),
which has access to both loaded modules and the inventory.

The following MUST be rejected:
- An instance referencing a `machine` not present in `machines`.
- An instance referencing a `role` not declared in the corresponding
  module's `interface.roles`.
- A service name not matching any loaded module.

The inventory's own Nickel contract (INV-4) validates structural
shape only (machines exist, instances have required fields). Module-
dependent cross-checks are deferred to EVAL-12.

### INV-4: Nickel contract location

The inventory contract MUST ship with crunch's embedded Nickel stdlib
(alongside `lib/lib.ncl`). The contract file defines the structural
shape. User inventories apply the contract via Nickel merge:
`inventory | crunch.Inventory`.

### INV-5: Fixed limits

- Maximum machines: 4096 (configurable).
- Maximum service instances per service: 4096 (configurable).
- Maximum total instances across all services: 65536 (configurable).

Exceeding any limit MUST fail fast with a diagnostic naming the limit.

### INV-6: Inventory is pure data

The inventory MUST be evaluable as a pure Nickel expression with no
side effects. It MUST NOT import module files or call module `impl`
functions. It is data, not code.
