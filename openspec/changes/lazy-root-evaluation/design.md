# Design: Lazy root evaluation

## Context

Today crunch typically goes through `crunch_eval::evaluate_and_extract_named_roots()`.
That path evaluates a `.ncl` file deeply enough for export, then extracts all
root derivations as typed Rust values. It keeps semantics simple, but it means
root discovery and root forcing happen together.

That is a poor fit for Nix/snix-style laziness. The first high-value lazy step
is to split:

1. top-level root discovery
2. selected-root forcing

The recent Cranelift experiment proved that `crunch-eval` can host alternate
backends, but it did not change the more important issue: crunch still forces
more than it needs too early.

## Goals / Non-Goals

**Goals:**

- expose a lazy root-evaluation boundary in `crunch-eval`
- let build-planning and pipeline code discover root labels before forcing root
  values
- define honest metrics for lazy evaluation optimization and autoresearch
- keep full-root workflows and semantic equivalence visible as guardrails

**Non-Goals:**

- implement incremental Nickel caches in this change
- turn `cranelift_proto.rs` into a larger parser/compiler
- replace Nickel imports, contracts, or merge semantics with custom logic
- optimize whole-program export time at the expense of selected-root latency
  visibility

## Decisions

### 1. `crunch-eval` owns the lazy session boundary

**Choice:** `crunch-eval` will expose a session-style API that keeps one Nickel
context and one top-level evaluation result alive across root discovery and
selected-root forcing.

**Rationale:** laziness and sharing only help if crunch keeps evaluation state
alive. If every helper collapses immediately to owned Rust data, sibling reuse
and per-root forcing disappear.

**Implementation:** add an `EvaluationSession`-style type in `crunch-eval` that
can:
- open a file or source string
- list root labels without deep-exporting whole roots
- force one selected root into an export/deserialize-ready value on demand

The exact helper names can evolve, but the ownership stays in `crunch-eval`,
not `crunch-pipeline`.

### 2. Root discovery is shallow; forcing is per-root

**Choice:** root discovery will use a shallow top-level evaluation boundary,
while forcing remains explicit and per-root.

**Rationale:** top-level package-set inspection is where crunch can avoid a lot
of unnecessary work. The important optimization is not “compile everything
faster”; it is “avoid forcing siblings before they are demanded.”

**Implementation:** `crunch-eval` should keep the whole-program export path for
operator-visible `crunch eval` output, but build/planning code should first:
- evaluate the top-level result only enough to determine whether it is a single
  derivation, array of derivations, or record of derivations
- enumerate labels or positions
- force one root only when the caller actually needs its `CrunchDerivation`

This change intentionally counts explicit top-level root forcing by crunch. It
DOES NOT try to infer every internal Nickel thunk that fires while evaluating a
selected root.

### 3. Implementation order starts with planning, then pipeline builds

**Choice:** the first consumer of the lazy session API should be
`crunch build --plan`, followed by `crunch-pipeline::build()`.

**Rationale:** planning is the cheapest place to prove the new boundary. It
needs root labels early and often does not need the same degree of eager work
as the full build path. That gives us a lower-risk proving ground before the
main pipeline switches over.

**Implementation:** thread the lazy session into `src/build_plan.rs` first,
then into `crates/crunch-pipeline/src/lib.rs`. The build path may still force
all roots eventually if all roots are selected for build, but it should not
require a whole-program deep export before label discovery.

### 4. Autoresearch optimizes selected-root latency, not whole-program export

**Choice:** the future autoresearch loop should target one checked-in, wide
package-set workload and optimize the latency to obtain one selected root from
that workload.

**Rationale:** selected-root latency is the user-visible behavior most likely to
benefit from laziness. Whole-program export time can hide the real win by
forcing siblings that a lazy consumer would never touch.

**Implementation:** the checked-in lazy benchmark workload names should be:
- `lazy-root-discovery-wide-package-set`
- `lazy-selected-root-wide-package-set`
- `eager-all-roots-wide-package-set`

The planned autoresearch session should use:
- **Primary metric:** `selected_root_total_wall_ns` (lower is better)
  - definition: end-to-end wall time from opening the lazy session to obtaining
    one fully materialized selected root from the fixed benchmark fixture
- **Secondary metrics:**
  - `root_discovery_wall_ns`
  - `selected_root_force_wall_ns`
  - `explicit_top_level_root_force_count`
  - `explicit_nonselected_root_force_count`
  - `all_roots_total_wall_ns`

Metric semantics:
- `selected_root_total_wall_ns` is the primary autoresearch target because it
  captures discovery overhead, force overhead, and any session setup trade-off
  together
- `root_discovery_wall_ns` and `selected_root_force_wall_ns` are split only at
  the `crunch-eval` API boundary, not by guessed internal Nickel VM events
- `explicit_top_level_root_force_count` counts only top-level root values that
  crunch explicitly forces through the lazy API
- `explicit_nonselected_root_force_count` counts only top-level roots other
  than the requested label that crunch explicitly forces; target value is `0`
- `all_roots_total_wall_ns` is a guardrail workload on the same fixture so the
  single-root path does not regress the full-root path catastrophically

Because current control workloads are sub-millisecond and noisy, the
autoresearch harness should either:
- use a wide enough checked-in fixture that the primary metric is comfortably
  above the noise floor on the reference host, or
- run repeated samples inside `autoresearch.sh` and report the median

If the new wide fixture depends on checked-in seed-backed conversion inputs, it
must use `/nix/store`-compatible paths for those fixtures instead of the default
`/crunch/store` benchmark prefix.

### 5. Compiled-eval work is tabled until the lazy path has data

**Choice:** after the current Cranelift experiment, further compiled-eval work
is tabled until lazy root discovery + selected-root forcing exist and are
benchmarked.

**Rationale:** it is too early to decide whether codegen matters if crunch is
still forcing too much too early. Laziness may remove enough work that a
compiled backend becomes marginal, or it may reveal a smaller, better-defined
hot path worth compiling later.

**Implementation:** keep the current compiled-eval change as evidence, but make
this lazy-eval change the main optimization direction. Revisit compiled backends
only after the lazy metrics are available.

## Verification Strategy

- unit-test root label discovery for single derivations, arrays, and records of
  derivations without forcing unrelated top-level roots through the crunch API
- equivalence-test selected-root forcing against the current eager path for:
  - flat derivations
  - package sets
  - nested derivation inputs
  - import-heavy fixtures
  - recursive-record references
- switch `src/build_plan.rs` to the lazy session first and verify the resulting
  plan entries match the eager path for the same checked-in fixtures
- add benchmark workloads for lazy root discovery, lazy selected-root forcing,
  and eager/all-roots guardrails on the same fixture
- record a baseline bundle and `benchmark_compare` transcript before beginning
  autoresearch

## Risks / Trade-offs

**[Hidden eager fallback]**
A “lazy” API could still deep-force everything internally and only move the
boundary on paper.

**Mitigation:** track `explicit_top_level_root_force_count` and
`explicit_nonselected_root_force_count` in the benchmark/autoresearch harness.

**[Nickel API mismatch]**
Nickel’s public API may not expose exactly the force boundary crunch wants.

**Mitigation:** keep the forcing mechanism internal to `crunch-eval` and start
with planning-first integration before changing the main pipeline.

**[Benchmark noise]**
Selected-root workloads can be too small and noisy to optimize honestly.

**Mitigation:** use a wider checked-in fixture and median sampling in the
autoresearch harness.

**[Scope creep back into codegen]**
The team could keep adding compiled-eval experiments before the lazy path is
measured.

**Mitigation:** make lazy metrics the optimization gate and keep codegen work
explicitly tabled in this change.
