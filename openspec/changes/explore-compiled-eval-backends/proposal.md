# Explore compiled evaluation backends

## Why

Crunch does not need Cranelift or LLVM in the current build/store path. The
current runtime uses `crunch-eval` plus Nickel's interpreter/export pipeline,
and the main system work still lives in derivation conversion, scheduling,
sandboxing, closure resolution, and store persistence.

That said, compiled evaluation backends may become useful future work if
profiling shows Nickel evaluation is a real bottleneck in large package sets,
self-build, or project-resolution flows. This change records where a compiled
backend would fit, what it must not disturb, and which backend family looks
like the better first experiment.

## What Changes

- add a future-work spec for optional compiled evaluation backends
- define `crunch-eval` as the only integration boundary for any future
  Cranelift or LLVM work
- record that build/store/sandbox layers stay unchanged by evaluator backend
  experiments
- prefer Cranelift for a first native-code experiment while keeping LLVM as an
  optional later path
- require profiling and semantic-equivalence evidence before any compiled
  backend becomes a shipped runtime path

## Capabilities

### New Capabilities

- `compiled-eval-backend-boundary`: future compiled evaluators have a defined
  integration point and do not leak into unrelated crates
- `profiling-gated-codegen`: codegen work starts only after evaluation is shown
  to be a meaningful bottleneck
- `cranelift-first-eval-experiment`: the first native-code backend experiment
  has a default direction without turning LLVM into a required dependency now

## Impact

- **Files**: new change-local spec under
  `openspec/changes/explore-compiled-eval-backends/specs/compiled-eval-backends/spec.md`
- **Architecture**: documents future evaluator extensibility without changing
  current runtime behavior
- **Dependencies**: no new Cargo or bootstrap dependencies in this change
- **Testing**: future implementation work will need profiling evidence and
  interpreter-vs-compiled equivalence tests

## Non-Goals

- ship a JIT or AOT evaluator in this change
- add LLVM or Cranelift to the current runtime dependency set
- rewrite build scheduling, substitution, PathInfo, or sandbox execution around
  compiler infrastructure
- claim evaluator codegen is current priority without measurement
