# Design: Explore compiled evaluation backends

## Context

Crunch evaluates Nickel through `crunch-eval` and Nickel's interpreter/export
APIs. Downstream crates care about evaluated derivation data:

- `crunch-glue` converts evaluated records into `nix_compat::Derivation`
- `crunch-build` schedules builds and dispatches `BuildService`
- `crunch-store` persists `PathInfo`, castore content, and attestations

Cranelift and LLVM only matter if evaluation itself dominates measured runtime.
They do not help store hashing, closure resolution, sandboxing, substitution, or
scheduler design. This change therefore keeps compiler work inside
`crunch-eval`, adds only an opt-in prototype, and records why that prototype is
not a shipped runtime path.

## Goals / Non-Goals

**Goals:**

- define where a future compiled evaluator backend integrates
- protect current build/store/sandbox layers from compiler coupling
- record benchmark thresholds and guardrails before promotion
- keep the first prototype feature-gated and subset-only
- require semantic-equivalence evidence before expanding support

**Non-Goals:**

- replace Nickel semantics with a crunch-specific language runtime
- require code generation for ordinary `crunch build` or `crunch eval`
- make LLVM or Cranelift default workspace dependencies
- promise a full JIT or AOT roadmap in this change

## Decisions

### 1. Keep `crunch-eval` as the evaluator boundary

**Choice:** any future compiled evaluator backend integrates behind
`crunch-eval` and keeps the same outward role: evaluate Nickel input, surface
Nickel-shaped diagnostics, and hand derivation-shaped data to downstream layers.

**Rationale:** the rest of crunch consumes evaluated values, not evaluator
implementation details. Keeping the boundary narrow prevents codegen concerns
from leaking into `crunch-glue`, `crunch-build`, `crunch-store`, or CLI
orchestration.

**Implementation:** `crates/crunch-eval/src/backend.rs` owns the private backend
seam; downstream crates keep using the existing evaluation helpers.

### 2. Treat compiled evaluation as an optimization, not a foundation

**Choice:** compiled evaluation remains optional and is gated on profiling
evidence that Nickel evaluation is a material bottleneck.

**Rationale:** compiler infrastructure adds dependency, bootstrap, correctness,
and maintenance cost. It is only justified if it removes a measured bottleneck.

**Implementation:** checked-in benchmark suite evidence records eval-bearing
workloads separately from non-eval controls. Docs continue to describe the
interpreter/export path as the shipped runtime until a compiled path exists.

### 3. Gate promotion with benchmark evidence and guardrails

**Choice:** a compiled backend cannot move beyond experiment status unless a
same-host benchmark suite shows both an evaluation-dominant workload and a
material eval-phase win without unacceptable non-eval regressions.

**Rationale:** a prototype can be useful even when it is not promotable. The
project needs a recorded reason to keep it feature-gated when guardrails fail.

**Implementation:** use `benchmark_suite` to capture the bundle and
`benchmark_compare` to compare against the saved baseline. The initial gate is
more than 90% eval share on `workflow-package-set-eval-build-graph`, at least a
2x eval-focused win before promotion, and non-eval guardrails within the
checked-in comparison thresholds.

### 4. Prefer Cranelift for the first experiment

**Choice:** the first native-code evaluator experiment targets Cranelift. LLVM
remains optional later work.

**Rationale:** Cranelift is lighter, has fast compile times, and fits the first
goal: test whether runtime evaluation of derivation-shaped config can be sped up
without committing to a full compiler stack.

**Implementation:** LLVM is reconsidered only if Cranelift cannot meet measured
goals or if a future shipped path needs LLVM-specific optimization depth,
ahead-of-time tooling, or external toolchain integration.

### 5. Keep the backend seam private

**Choice:** `crunch-eval` keeps its current free-function API for callers but
routes evaluation through a private `EvalBackend` trait and `EvalRequest`
carrier inside the crate.

**Rationale:** this gives experiments one backend-local insertion point without
forcing a public API commitment.

**Implementation:** `NickelBackend` remains the default. The private seam owns
request preparation and dispatch for `evaluate`, `evaluate_str`,
`evaluate_to_json`, and `evaluate_str_to_json`.

### 6. Keep the Cranelift prototype feature-gated and subset-only

**Choice:** the prototype stays behind `cranelift-proto` and supports only flat
derivation literals with required `name` and `builder`, plus optional `system`,
`addressing_mode`, `args`, and `outputs`.

**Rationale:** this proves the backend seam with real code generation while
keeping the semantic surface small enough to test honestly.

**Implementation:** `crates/crunch-eval/src/cranelift_proto.rs` parses the
supported literal subset, JIT-compiles pointer/length writes for scalar string
fields and their defaults, and returns JSON through feature-gated helper
functions. Unsupported imports, merges, nested inputs, env maps, fixed-output
metadata, package sets, contracts, and recursive records are outside this
prototype.

### 7. Preserve interpreter-defined semantics before expansion

**Choice:** every compiled backend must preserve today's Nickel semantics for
its supported subset, and broader support requires interpreter-vs-compiled
equivalence tests for contracts, merges, imports, package sets, recursive
records, and nested derivation inputs.

**Rationale:** faster wrong answers are not useful. A compiled backend that
weakens semantics would be a language fork, not an optimization.

**Implementation:** the current prototype has subset parity tests only. Broader
semantic coverage is a prerequisite for any later expansion beyond the flat
literal subset.

### 8. Table further compiled-eval work after lazy-root data

**Choice:** after this prototype, further compiled-eval work is tabled until
lazy-root evaluation data shows a remaining codegen-sized bottleneck.

**Rationale:** lazy root discovery and selected-root forcing removed unnecessary
work from the evaluation path and became the main optimization direction. That
can make compiled evaluation less valuable or expose a narrower future target.

**Implementation:** the archived `lazy-root-evaluation` change records this
reassessment and keeps compiled-eval work tabled until lazy metrics justify
reopening it.

## Current benchmark gate evidence

- Baseline bundle: `openspec/changes/explore-compiled-eval-backends/evidence/compiled-eval-gate.json`
- Baseline command: `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/compiled-eval-gate.json --repeat-count 2`
- `eval-fetch-git`: `evaluation_wall_ns = 89535453`, `total_wall_ns = 89549279`
  → evaluation share `99.98%`
- `workflow-package-set-eval-build-graph`: `evaluation_wall_ns = 142402615`,
  `build_graph_wall_ns = 1158384`, `total_wall_ns = 143675183` → evaluation
  share `99.11%`
- The checked-in suite showed two interpreter-bound eval-bearing workloads and
  no checked-in I/O-bound eval-bearing workload at this point
- No checked-in benchmark in this change measured `crunch project` flows or
  self-build evaluation directly

## Current prototype evidence

- `crates/crunch-eval/Cargo.toml` adds optional `cranelift-proto` dependencies
  only behind the feature flag
- `crates/crunch-eval/src/backend.rs` contains the private backend seam
- `crates/crunch-eval/src/cranelift_proto.rs` implements the flat-derivation
  Cranelift prototype
- `openspec/changes/explore-compiled-eval-backends/evidence/cranelift-prototype-tests.txt`
  captures:
  - `cargo test -p crunch-eval --lib --features cranelift-proto` →
    `test result: ok. 41 passed`
  - `cargo test -p crunch-eval --lib` → `test result: ok. 35 passed`
  - `cargo test -p crunch --test examples_eval` → `test result: ok. 4 passed`
- `openspec/changes/explore-compiled-eval-backends/evidence/post-prototype-benchmark-compare.txt`
  records non-eval guardrail drift in the old prototype comparison, so the
  prototype remains non-shipping and tabled rather than promoted

## Risks / Trade-offs

**[Wrong bottleneck]**
Evaluation might not remain the dominant cost after lazy root evaluation.

**Mitigation:** keep compiled-eval work tabled until lazy metrics show a
remaining codegen-sized target.

**[Semantic drift]**
A lowered IR or compiled subset may disagree with Nickel interpreter behavior.

**Mitigation:** preserve the interpreter path as reference behavior and require
broad equivalence tests before expanding the prototype.

**[Dependency weight]**
Compiler crates add cost even when optional.

**Mitigation:** keep Cranelift behind `cranelift-proto` and keep LLVM optional
until a later measured need exists.
