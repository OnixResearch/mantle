## Context

crunch already owns package evaluation, derivation conversion, and build/store
execution, but it still has no native equivalent to the NixOS-style module
pipeline. The active proposal adds that missing layer: load Nickel service
modules, evaluate them against an inventory, merge their fragments per machine,
and assemble buildable derivations without a Nix shim.

The main design risk is not raw implementation complexity. It is contract
drift at the seam between async orchestration, the non-`Send` Nickel runtime,
CLI result rendering, and pluggable assembler selection. This design fixes that
boundary before implementation starts.

## Goals / Non-Goals

**Goals**
- Keep module and inventory schema in Nickel contracts.
- Keep graph logic, merge logic, and build orchestration in Rust.
- Keep raw Nickel values on one dedicated thread while allowing async CLI
  orchestration.
- Define a partial-success result model that can report successful machines plus
  warnings/errors coherently.
- Ship one phase-1 NixOS backend that proves the pipeline end to end.

**Non-Goals**
- Full NixOS closure assembly.
- A parallel or distributed Nickel runtime.
- Importing nixpkgs or existing NixOS modules.
- Deployment or activation commands.

## Decisions

### 1. One crate owns the pipeline

**Choice:** create `crates/crunch-system/` with submodules for loader,
evaluator, fragment collector, assembler, inventory, errors, eval trait, and
threading.

**Rationale:** the layers share types and evolve together. Splitting them into
multiple crates now would add dependency churn without giving a real versioning
boundary.

### 2. Keep the functional core separate from the imperative shell

**Choice:** filesystem discovery, CLI parsing, evaluator-thread lifecycle, and
calls into `crunch-pipeline::build()` live in the shell. Dependency ordering,
fragment merging, backend lookup, and result aggregation stay pure Rust.

**Rationale:** this keeps most correctness logic testable without a filesystem,
Nickel runtime, or build store.

### 3. `NickelValue` is on-thread only; async code uses `ValueId`

**Choice:** `NickelValue` remains a type alias for `crunch_eval::Expr`, but it
never crosses the evaluator-thread boundary. Async orchestration stores
`ValueId` handles, and only the evaluator thread dereferences those handles to
raw `NickelValue` values.

**Rationale:** this resolves the current drift between the proposal/spec/task
story. The type alias is still useful for the on-thread `NickelEvaluator`
contract, but the async pipeline cannot pretend that `NickelValue` is a shared
cross-thread representation.

### 4. The evaluator boundary has two faces

**Choice:** define an on-thread `NickelEvaluator` trait that works with
`NickelValue`, and define an async `EvalThreadHandle` protocol that works with
`ValueId`, `serde_json::Value`, and `EvalError`.

**Rationale:** this gives pure-core code a clean trait boundary while still
acknowledging the real thread boundary. Loader and evaluator logic can stay
expressive without violating `!Send` constraints.

### 5. `ValidatedModule` stores metadata plus handle references

**Choice:** `ValidatedModule` lives on the async side and stores plain Rust
metadata together with `ValueId` handles for fields such as `interface` and
`impl`.

**Rationale:** the async pipeline needs durable references into the evaluator
thread's value table, but it must not hold raw Expr values.

### 6. Result aggregation carries warnings separately from errors

**Choice:** keep `SystemConfigError` for hard failures, add
`SystemConfigWarning` for non-fatal issues, and make
`SystemPipelineResult { machines, errors, warnings }` the shared non-fatal
result envelope.

**Rationale:** orphan-provider consumption and similar conditions are true
warnings, not malformed errors. The explicit split makes CLI rendering and
partial-success semantics unambiguous.

### 7. Partial success is first-class

**Choice:** independent modules and machines continue after non-fatal failures.
Successful machine outcomes stay in `SystemPipelineResult.machines`, while
non-fatal problems accumulate in `errors` and `warnings`. Fatal inventory
validation still short-circuits with a top-level error result.

**Rationale:** multi-machine evaluation is not useful if one bad module erases
all successful work.

### 8. Stderr carries diagnostics; stdout carries the primary result

**Choice:** both `crunch system eval` and `crunch system build` reserve stdout
for the command's primary result envelope. Stderr carries warnings/errors. When
crunch's global `--json` flag is active, stderr switches to one JSON
diagnostic object per line and stdout stays machine-readable.

**Rationale:** this matches existing crunch CLI expectations and fixes the
current ambiguity around mixed stdout/stderr behavior.

### 9. Build mode embeds existing build-report payloads

**Choice:** `crunch system build --json` returns a `SystemPipelineResult`
envelope whose successful machine outcomes embed the existing
`crunch-build-report-v1` payloads produced by `crunch-pipeline::build()`.

**Rationale:** system builds need a machine-level envelope for partial success,
but they should still reuse the established build-report schema rather than
inventing a second build-report format.

### 10. Assembler override precedence is explicit

**Choice:** machine selection happens first, then backend override resolution.
Without `--assembler`, each selected machine uses `machine.class` (defaulting to
`nixos`). With `--assembler <name>`, every selected machine uses that backend
for the current invocation.

**Rationale:** this removes ambiguity when multi-machine builds mix machine
classes and CLI overrides.

### 11. Phase-1 backend proves the pipeline, not the whole system story

**Choice:** the first `nixos` backend only turns merged `output.nixos` data into
a derivation that writes `$out/system-config.json`.

**Rationale:** this is enough to prove module loading, evaluation, merge,
assembly, and build execution together, without over-claiming a full NixOS
replacement in one change.

## Data Flow

```text
inventory.ncl
  -> crunch_eval::evaluate_and_deserialize::<Inventory>()
  -> Inventory

modules/*.ncl
  -> discover_module_files()
  -> EvalThreadHandle::evaluate_file() -> ValueId
  -> validate_module(name, value_id, handle) -> ValidatedModule
  -> build dependency graph
  -> per-instance merge/call/to_json through EvalThreadHandle
  -> EvaluatedFragment { data: serde_json::Value }
  -> group + merge per machine
  -> MergedConfig
  -> assembler registry selects backend
  -> dry-run derivations or build reports
  -> SystemPipelineResult { machines, errors, warnings }
```

## Verification Strategy

- **Loader**: top-level discovery, duplicate stems, and count-limit tests.
- **Evaluator seam**: `ValueId`-based field access, timeout, import sandboxing,
  and handle cleanup tests.
- **Module evaluator**: dependency ordering, provider threading, partial
  success, and determinism tests.
- **Fragment collector**: deep merge, precedence conflicts, filtering, and
  provenance tests.
- **Assembler**: backend selection, override precedence, dry-run, and NixOS
  passthrough tests.
- **CLI**: eval/build envelope shape, stderr diagnostic behavior, and partial-
  success exit-status tests.

## Risks / Trade-offs

**Dedicated thread limits parallelism.** This is acceptable for phase 1 because
correct boundaries matter more than speculative concurrency. The handle-based
protocol leaves room for future parallel evaluators if needed.

**Two-layer evaluator API is more explicit than the original sketch.** That is
deliberate. The earlier single-type story blurred the `NickelValue`/`ValueId`
seam and made both the specs and tasks internally inconsistent.

**System build JSON becomes an envelope instead of a bare build report.** This
is the right trade-off for partial success because multi-machine execution needs
a place to record warnings, errors, and per-machine outcomes together.
