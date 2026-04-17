# Design: Explore compiled evaluation backends

## Context

Today crunch evaluates Nickel through `crunch-eval` and Nickel's existing
interpreter/export APIs. That is a clean fit for the current system because the
rest of crunch cares about evaluated derivation data, not machine code:

- `crunch-glue` converts evaluated records into `nix_compat::Derivation`
- `crunch-build` schedules builds and dispatches `BuildService`
- `crunch-store` persists `PathInfo`, castore content, and attestations

Cranelift and LLVM do not naturally help with store hashing, closure
resolution, sandboxing, substitution, or goal scheduling. They only become
interesting if evaluation itself becomes expensive enough to justify another
backend.

## Goals / Non-Goals

**Goals:**

- define where a future compiled evaluator backend would integrate
- protect current build/store layers from unnecessary compiler coupling
- record backend-selection guidance for future experiments
- require measurement and semantic-equivalence evidence before shipping a
  compiled path

**Non-Goals:**

- replace Nickel semantics with a crunch-specific language runtime
- require code generation for ordinary `crunch build` or `crunch eval`
- decide now between a full JIT and full AOT roadmap
- promise LLVM support on day one of any evaluator experiment

## Decisions

### 1. Keep `crunch-eval` as the evaluator boundary

**Choice:** any future compiled evaluator backend will integrate behind
`crunch-eval` and keep the same outward role: evaluate Nickel input, surface
Nickel-shaped diagnostics, and hand derivation-shaped data to downstream
layers.

**Rationale:** the rest of crunch consumes evaluated values, not evaluator
implementation details. Keeping the boundary narrow prevents codegen concerns
from leaking into `crunch-glue`, `crunch-build`, `crunch-store`, or CLI
orchestration.

**Implementation:** future work may add backend selection or internal traits in
`crunch-eval`, but downstream crates should continue to depend on evaluated
results and existing typed extraction helpers.

### 2. Treat compiled evaluation as an optimization, not a new foundation

**Choice:** compiled evaluation remains optional future work and is gated on
profiling evidence that Nickel evaluation is a material bottleneck.

**Rationale:** crunch's present architecture work is mostly elsewhere.
Introducing compiler infrastructure before measurement would add heavy
complexity without guaranteed payback.

**Implementation:** require benchmarks or real workload profiling from package
sets, self-build, or project commands before starting backend integration.
Documentation must continue to describe the interpreter/export path as the
shipped runtime until a compiled path exists.

### 3. Prefer Cranelift for the first experiment

**Choice:** the first native-code evaluator experiment SHOULD target Cranelift.
LLVM remains an optional later path.

**Rationale:** Cranelift is lighter, has fast compile times, and fits the most
plausible first goal here: speeding up runtime evaluation of derivation-shaped
config code. LLVM only becomes attractive if later work needs deeper
whole-program optimization, broader ahead-of-time tooling, or tighter external
compiler/toolchain integration.

**Implementation:** future experiments should prototype Cranelift first against
an intentionally narrow subset or lowered IR. LLVM reconsideration happens only
if Cranelift proves insufficient for measured goals.

### 4. Preserve interpreter-defined semantics

**Choice:** any compiled backend must preserve today's Nickel semantics for the
supported subset, including contracts, merge behavior, recursive-record
resolution, import handling, and derivation extraction shape.

**Rationale:** faster wrong answers are not useful. crunch's correctness today
comes from Nickel semantics plus typed conversion and downstream build logic.
A compiled backend that weakens those semantics would not be an optimization;
it would be a language fork.

**Implementation:** future work must compare compiled results against the
existing interpreter path on derivation-heavy fixtures, contract failures,
package sets, and nested input shapes.

### 5. Keep compiler backends out of build/store portability claims

**Choice:** compiler backends do not change the existing portability boundary.
Build execution stays behind `BuildService`; store operations stay behind
castore/pathinfo services.

**Rationale:** evaluator backend work and sandbox/backend portability are
separate concerns. Mixing them would blur architecture lines and create false
expectations that LLVM or Cranelift somehow solve runtime build portability.

**Implementation:** future specs and docs should describe compiled evaluators as
an optional eval-layer optimization only.

### 6. Gate evaluator work on checked-in benchmark workloads

**Choice:** until the repo gains a checked-in `crunch project` or self-build
benchmark, compiled-eval planning will gate on two existing suite workloads:
`eval-fetch-git` as the narrow evaluation probe and
`workflow-package-set-eval-build-graph` as the multi-phase representative
workflow. The remaining suite entries are non-eval controls and do not justify
compiler work on their own.

**Rationale:** these workloads already separate honest `evaluation_wall_ns`
from build-graph, substitution, and store work; they use checked-in fixtures;
and they run in ordinary local checks with no network dependency.

**Implementation:** capture a same-host `benchmark_suite` bundle before any
backend experiment, record the bundle under
`openspec/changes/explore-compiled-eval-backends/evidence/`, and keep
project-command or self-build claims out of scope until a checked-in workload
measures those paths directly.

### 7. Require both dominant evaluation cost and a material prototype win

**Choice:** compiled-eval work is justified only when a same-host
`benchmark_suite` run shows `workflow-package-set-eval-build-graph` spending
more than 90% of `total_wall_ns` in `evaluation_wall_ns`, and a future backend
prototype can deliver at least a 2x eval-phase speedup on `eval-fetch-git`
without regressing non-eval workloads past the documented
`benchmark_compare` thresholds.

**Rationale:** a compiled backend adds dependency, maintenance, bootstrap, and
correctness cost. The project needs proof that evaluation dominates a real
workflow and proof that a prototype buys a materially large win.

**Implementation:** use `benchmark_compare` against a saved baseline bundle,
treat non-eval workloads as guardrails rather than targets, and do not advance
past a narrow prototype until Phase 3 interpreter-vs-compiled equivalence tests
pass.

### 8. Add a private backend seam before any compiled prototype

**Choice:** `crunch-eval` keeps its current free-function API for callers, but
routes evaluation through a private `EvalBackend` trait and `EvalRequest`
carrier owned inside the crate.

**Rationale:** this gives Phase 3 experiments one backend-local insertion point
without changing downstream crates or forcing an early public API commitment.
The shipped path stays the Nickel interpreter, while a future prototype can add
another backend implementation behind the same internal boundary.

**Implementation:** `crates/crunch-eval/src/backend.rs` owns
`EvalBackend`, `EvalRequest`, and `NickelBackend`. `crates/crunch-eval/src/lib.rs`
now prepares file or inline requests and delegates `evaluate`,
`evaluate_str`, `evaluate_to_json`, and `evaluate_str_to_json` through
`default_backend()`.

### 9. Keep the first Cranelift prototype feature-gated and subset-only

**Choice:** the first Cranelift experiment stays behind an optional
`cranelift-proto` Cargo feature and supports only flat derivation literals:
required `name` and `builder`, plus optional `system`, `addressing_mode`,
`args`, and `outputs`.

**Rationale:** this proves the backend seam with real code generation while
keeping the semantic surface small enough to test honestly. The default shipped
runtime path stays the Nickel interpreter, and the prototype rejects imports,
merges, nested inputs, env maps, fixed-output metadata, and package-set shapes
instead of pretending to support them.

**Implementation:** `crates/crunch-eval/src/cranelift_proto.rs` parses the
supported literal subset, JIT-compiles pointer/length writes for scalar string
fields and their defaults, and returns JSON through
`evaluate_str_to_json_with_cranelift_prototype(...)`,
`evaluate_str_and_deserialize_with_cranelift_prototype(...)`, and
`evaluate_str_and_extract_named_roots_with_cranelift_prototype(...)`. The
prototype backend implementation lives in `crates/crunch-eval/src/backend.rs`.

## Current benchmark gate evidence

- Baseline bundle: `target/benchmarks/compiled-eval-gate.json`
- Checked-in evidence copy:
  `openspec/changes/explore-compiled-eval-backends/evidence/compiled-eval-gate.json`
- Command:
  `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/compiled-eval-gate.json --repeat-count 2`
- `eval-fetch-git`: `evaluation_wall_ns = 89535453`,
  `total_wall_ns = 89549279` → evaluation share `99.98%` → interpreter-bound
- `workflow-package-set-eval-build-graph`:
  `evaluation_wall_ns = 142402615`, `build_graph_wall_ns = 1158384`,
  `total_wall_ns = 143675183` → evaluation share `99.11%`,
  evaluation/build-graph ratio `122.93x` → interpreter-bound
- `convert-multi-output`, `build-graph-package-set`,
  `substitution-plan-delta-suite`, and `store-persist-lookup-blob` expose
  non-eval control phases only and are not compiled-eval targets by
  themselves
- The current checked-in suite therefore shows two interpreter-bound
  eval-bearing workloads and no checked-in I/O-bound eval-bearing workload yet
- No checked-in benchmark yet measures `crunch project` flows or self-build
  evaluation directly, so this change keeps project/self-build codegen claims
  out of scope for now

## Current prototype evidence

- `crates/crunch-eval/Cargo.toml` adds optional `cranelift-proto`
  dependencies only behind the feature flag; the default runtime build path is
  unchanged
- `crates/crunch-eval/src/cranelift_proto.rs` now implements a feature-gated
  Cranelift prototype for flat derivation literals only
- `openspec/changes/explore-compiled-eval-backends/evidence/cranelift-prototype-tests.txt`
  captures:
  - `cargo test -p crunch-eval --lib --features cranelift-proto` →
    `test result: ok. 41 passed`
  - `cargo test -p crunch-eval --lib` → `test result: ok. 35 passed`
  - `cargo test -p crunch --test examples_eval` → `test result: ok. 4 passed`
- `openspec/changes/explore-compiled-eval-backends/evidence/post-prototype-benchmark-compare.txt`
  currently shows non-eval drift above the documented thresholds for
  `build-graph-package-set`, so the regression-guard task remains open

## Risks / Trade-offs

**[Wrong bottleneck]**
Evaluation might not be the dominant cost on real workloads.

**Mitigation:** require measured evidence before implementation.

**[Semantic drift]**
A lowered IR or compiled subset may disagree with Nickel interpreter behavior.

**Mitigation:** keep interpreter equivalence tests and preserve the interpreter
path as reference behavior until the compiled path is proven.

**[Dependency weight]**
LLVM in particular can add large build and bootstrap cost.

**Mitigation:** prefer Cranelift first and keep LLVM optional, not mandatory.

**[Scope creep]**
A performance experiment could turn into a language/runtime rewrite.

**Mitigation:** keep the first experiment narrow: derivation-oriented
workloads, backend-local changes, no store/build redesign.
