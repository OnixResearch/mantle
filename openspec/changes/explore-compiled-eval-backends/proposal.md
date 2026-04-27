# Explore compiled evaluation backends

## Why

Crunch still ships Nickel's interpreter/export pipeline as the default runtime.
That path is correct and keeps build, store, sandbox, substitution, and
PathInfo concerns independent from compiler infrastructure.

Benchmark evidence later showed two checked-in eval-bearing workloads where
`evaluation_wall_ns` dominates the measured runtime:
`openspec/changes/explore-compiled-eval-backends/evidence/compiled-eval-gate.json`
plus `evidence/benchmark-suite-baseline.txt` record `eval-fetch-git` and
`workflow-package-set-eval-build-graph`;
`evidence/post-prototype-benchmark-compare.txt` records the old prototype
guardrail comparison. That makes a narrow
compiled-evaluator experiment useful as evidence, but not as a shipped runtime
path. The change therefore records both the architecture boundary and the
feature-gated Cranelift prototype that was used to test the boundary.

## What Changes

- add a future-work spec for optional compiled evaluation backends
- keep `crunch-eval` as the only integration boundary for any Cranelift or LLVM
  work
- add a private `crunch-eval` backend seam while preserving existing public
  evaluation helpers for callers
- add an optional `cranelift-proto` Cargo feature in `crunch-eval` for a narrow
  flat-derivation prototype
- record benchmark-gate and regression-guard evidence before any compiled path
  can be promoted
- keep the interpreter/export path as the default shipped runtime

## Capabilities

### New Capabilities

- `compiled-eval-backend-boundary`: compiled evaluators have a defined
  integration point and do not leak into unrelated crates
- `profiling-gated-codegen`: codegen work starts only after evaluation is shown
  to be a meaningful bottleneck
- `cranelift-first-eval-experiment`: the first native-code backend experiment
  has a default direction without turning LLVM into a required dependency now
- `feature-gated-cranelift-prototype`: prototype dependencies stay opt-in and do
  not affect the default runtime path

## Impact

- **Files**: `crates/crunch-eval/src/backend.rs`,
  `crates/crunch-eval/src/cranelift_proto.rs`, `crates/crunch-eval/Cargo.toml`,
  benchmark evidence under this change, and the compiled-eval spec.
- **Architecture**: documents future evaluator extensibility without changing
  build/store/sandbox boundaries.
- **Dependencies**: Cranelift dependencies are allowed only behind the optional
  `cranelift-proto` feature in `crunch-eval`; no downstream crate or default
  runtime feature may require them.
- **Testing**: benchmark compare evidence, default interpreter tests,
  feature-gated prototype tests, and interpreter-vs-prototype subset checks.

## Non-Goals

- ship a JIT or AOT evaluator as the default runtime path
- make LLVM or Cranelift a default workspace dependency
- expand the prototype beyond the documented flat-derivation subset here
- rewrite build scheduling, substitution, PathInfo, or sandbox execution around
  compiler infrastructure
- claim compiled evaluation is the current optimization priority after lazy-root
  evaluation became the main measured optimization path
