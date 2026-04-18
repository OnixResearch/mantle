# Design: Parallel root evaluation

## Context

Crunch already split root discovery from selected-root forcing, but the
multi-root path is still mostly serial.

Current shape:

- `EvaluationSession::open_file()` does shallow discovery and keeps one Nickel
  `Context` alive.
- `EvaluationSession::force_root()` deep-forces one selected root.
- `EvaluationSession::force_all_roots()` loops over labels and calls
  `force_root()` one label at a time.
- `crunch-pipeline::build()` calls `force_all_roots()` before conversion and
  before the worker sees any root.

That means a wide package set still pays a serial eval/convert prefix even
though downstream build dispatch is already parallel.

## Goals / Non-Goals

**Goals:**

- add bounded multi-root materialization for build-oriented callers
- preserve lazy discovery and selected-root semantics
- let pipeline overlap root forcing/conversion with downstream scheduling
- define honest benchmark evidence for throughput wins and selected-root
  guardrails

**Non-Goals:**

- prove Nickel `Context` is thread-safe or share one mutable `Context` across
  threads without evidence
- redesign build-worker scheduling itself
- introduce compiled eval backends as part of this change
- optimize single selected-root latency at the expense of wide-package-set
  throughput measurement clarity

## Decisions

### 1. `crunch-eval` owns the batch forcing boundary

**Choice:** bounded multi-root forcing lives in `crunch-eval`, not in
`crunch-pipeline`.

**Rationale:** root shape, label ordering, Nickel error handling, and selected
root semantics already belong to `crunch-eval`. Putting the parallel boundary
in the pipeline would duplicate label selection and couple downstream crates to
Nickel-specific forcing details.

**Implementation:** extend the session-facing API with a batch forcing entry
point for selected labels. The API must preserve request order in its returned
results and must include label context when one requested root fails.

### 2. Parallelism uses isolated forcing workers, not one shared mutable `Context`

**Choice:** first implementation must assume the current Nickel session state is
not safe to drive from multiple threads at once.

**Rationale:** current lazy path depends on one live `Context`, but this change
needs throughput, not speculative thread-safety. A wrong shared-context design
would risk races, hidden nondeterminism, or unsound aliasing.

**Implementation:** the parallel path may reuse shallow discovery results from a
coordinator session, but each concurrent root-force job should run through an
isolated worker session or other independently safe evaluator state. The
coordinator should capture the source text and import-path set once, then hand
that immutable input to worker sessions so the first implementation does not
re-read the `.ncl` file from disk per root-force job. The public spec stays
about semantics and bounded concurrency, not about a specific Rust threading
primitive.

### 3. Concurrency is bounded and derived from existing build parallelism

**Choice:** first iteration reuses existing build parallelism configuration as
its upper bound instead of adding a new CLI knob.

**Rationale:** users already control total build parallelism through `max_jobs`.
A second public knob would complicate planning and oversubscription before we
have evidence that separate tuning is needed.

**Implementation:** pipeline chooses an internal eval parallelism no greater
than `max_jobs`, the number of requested roots, and a fixed safety cap if
needed. The implementation may still choose `1` when only one root is selected.

### 4. Pipeline streams eval/convert output incrementally

**Choice:** `crunch-pipeline` should stop building one full `Vec<(String,
CrunchDerivation)>` before conversion and scheduling.

**Rationale:** serial full-materialization recreates the current bottleneck even
if root forcing itself becomes parallel. The worker should start seeing roots as
soon as each label is forced and converted.

**Implementation:** after lazy discovery, pipeline issues bounded independent
root-force requests, potentially one label per request, converts each root as
it lands, drains pending conversion cache entries, and sends `EvalMessage`s to
the worker incrementally. This streaming path does not rely on partial success
results from a failed multi-root request.

### 5. Determinism beats maximum throughput

**Choice:** returned root labels, root-to-derivation association, and typed
error labeling stay deterministic even when forcing runs concurrently.

**Rationale:** a faster pipeline that can swap labels, reorder externally
visible outcomes unpredictably, or surface unlabeled eval failures is not an
acceptable trade.

**Implementation:** root request order is stable, result association stays by
label, and tests must assert label-to-output pairing rather than only comparing
unordered sets.

### 6. Verification needs both throughput evidence and selected-root guardrails

**Choice:** benchmark evidence must cover wide multi-root throughput and confirm
selected-root lazy metrics do not regress silently.

**Rationale:** this change is motivated by multi-root throughput, but the repo
just added lazy selected-root evaluation and should not give that win back.

**Implementation:** add checked-in serial-vs-parallel all-root benchmark
workloads on the wide fixture and compare them with existing selected-root lazy
metrics as guardrails. The fixed fixture should stay `/nix/store`-compatible if
it depends on checked-in seed-backed source paths, and the benchmark workflow
should run in isolation from unrelated Cargo builds so shared-target-dir noise
does not fake wins or regressions.

## Verification Strategy

- unit-test batch forcing on single, array, and record outputs
- equivalence-test parallel batch forcing against the current serial path for:
  - flat derivations
  - package sets
  - nested derivation inputs
  - import-heavy fixtures
- integration-test pipeline streaming so root label -> output association stays
  correct under concurrency
- add benchmark evidence on the checked-in wide fixture for serial vs parallel
  all-root forcing
- compare selected-root lazy metrics before/after the change so single-root
  latency stays explicit

## Risks / Trade-offs

**[Nickel worker overhead]**
Opening isolated forcing workers may duplicate some setup and reduce the gross
parallelism win.

**Mitigation:** benchmark the fixed wide fixture and keep the first cap bounded.

**[Determinism drift]**
Concurrent forcing could scramble externally visible root ordering or label
association.

**Mitigation:** preserve request order at the API boundary and assert label ->
result association in tests.

**[Pipeline oversubscription]**
Eval and build work could compete for the same CPU budget and regress some
hosts.

**Mitigation:** derive eval concurrency from `max_jobs` and keep selected-root
metrics plus serial-vs-parallel benchmark evidence visible.

**[Wrong abstraction split]**
Too much concurrency logic in `crunch-pipeline` could recreate Nickel-specific
knowledge outside `crunch-eval`.

**Mitigation:** keep label discovery, batch forcing semantics, and eval errors
owned by `crunch-eval`; pipeline only orchestrates bounded requests and
conversion streaming.
