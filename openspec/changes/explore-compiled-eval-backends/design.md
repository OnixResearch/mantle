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
