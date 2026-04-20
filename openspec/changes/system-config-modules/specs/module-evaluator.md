# Module Evaluator

## Overview

Resolves module dependency order, validates settings, threads
exports/providers, and calls each module's `impl` function. Pure
deterministic core — no I/O, no store access.

## Requirements

### EVAL-1: Topological sort

The evaluator MUST sort modules by their declared `inputs` dependencies.
Modules with no inputs are evaluated first. The sort MUST be stable
(tiebreak by priority, then by name).

### EVAL-2: Cycle detection

Circular dependencies MUST be detected before evaluation begins. The
error MUST name all modules in the cycle.

### EVAL-3: Settings validation

For each module instance in the inventory, the evaluator MUST merge
user-provided settings with the module's interface defaults using Nickel
contract merge. Contract violations MUST produce Nickel blame errors
with the module name, role name, and field path.

### EVAL-4: Impl invocation

The evaluator MUST call each module's `impl` function with a record
containing at minimum:
- `settings` — validated settings for this instance.
- `machine_name` — string.
- `role_name` — string.
- `upstream` — record of outputs from declared input modules.
- `providers` — record keyed by provider type, each value an array
  of provider outputs from modules that produce that type (EVAL-6d).
  Present only when the module declares `consumes_providers`. For
  orphan provider types (EVAL-6e), the value is an empty array.

The return value MUST be a record. The evaluator does not interpret
the record's contents — that is the fragment collector's job.

### EVAL-5: Export threading

If a module's impl returns an `output.exports` field, those exports
MUST be made available to downstream modules that declare this module
as an input, under the `upstream.<module_name>` key.

### EVAL-6: Provider threading

If a module's impl returns an `output.providers` field, those providers
MUST be collected and made available to modules that consume that
provider type.

#### EVAL-6a: Provider consumption declaration

A module declares provider consumption via an optional
`consumes_providers` field in its interface: an array of provider-type
strings (e.g., `["firewall", "reverse-proxy"]`). Modules without this
field receive no providers.

#### EVAL-6b: Provider ordering

Provider production and consumption are declared statically in the
module structural contract (`produces_providers` and
`consumes_providers` in LOADER-2). The evaluator computes ordering
edges from these declarations before evaluation begins: a module that
consumes provider type `P` MUST be evaluated after all modules that
declare `produces_providers` containing `P`. These edges are merged
into the topological sort alongside explicit `inputs` edges.

#### EVAL-6c: Provider cycle detection

EVAL-2 cycle detection MUST operate on the combined graph of explicit
`inputs` edges and implicit provider edges. Cycles involving provider
edges MUST name both the modules and the provider types involved.

#### EVAL-6d: Provider merging

When multiple modules produce the same provider type, their provider
outputs are collected into an array ordered by evaluation order. The
consuming module receives `providers.<type>` as this array, not a
merged record.

#### EVAL-6e: Orphan provider consumption

If a module declares `consumes_providers` containing a provider type
that no loaded module produces, the evaluator MUST emit a warning
(not an error). The consuming module receives `providers.<type>` as
an empty array. This allows optional provider dependencies.

### EVAL-7: Determinism

Given the same modules, inventory, and overrides, the evaluator MUST
produce identical output fragments. No randomness, no timestamps, no
ambient state.

### EVAL-8: Fixed limits

The evaluator MUST enforce:
- Maximum dependency chain depth: 256 (configurable).
- Maximum provider types per module: 32.
- Maximum total provider edges across all modules: 4096.

Exceeding any limit MUST fail fast before evaluation begins.

### EVAL-9: Error collection

Evaluation failures in one module MUST NOT prevent evaluation of
independent modules. The evaluator MUST collect all errors and return
them alongside any successful results (fail-open per independent
subgraph, fail-closed per dependency chain).

### EVAL-10: Nickel evaluation resource bounds

Nickel evaluation of each module MUST be bounded by a configurable
wallclock timeout (default 60 seconds per module `impl` call).
Exceeding the timeout MUST abort that module's evaluation with a
diagnostic naming the module and the elapsed time. Other independent
modules continue evaluating per EVAL-9.

Memory bounding SHOULD be enforced if the Nickel runtime supports it.
See TRAIT-3 for the `EvalOptions` contract.

### EVAL-11: NickelEvaluator trait

The evaluator MUST receive Nickel evaluation capability through an
injected `NickelEvaluator` trait. The full trait contract including
all methods, value types, error types, and resource bounds is
specified in `evaluator-trait.md` (TRAIT-1 through TRAIT-6). This
requirement defers to that spec as authoritative.

The trait MUST be object-safe. `crunch-eval` provides the production
implementation. Tests inject a mock evaluator that returns
predetermined values without a Nickel runtime.

### EVAL-12: Cross-reference validation

Before calling any module's `impl`, the evaluator MUST validate
inventory-module cross-references:
- Each service name in the inventory matches a loaded module.
- Each instance's `role` matches a role in the module's `interface.roles`.
- Each instance's `machine` matches a machine in the inventory.

All violations MUST be collected and reported together (same semantics
as INV-3, but executed here because validation requires loaded modules).
