# Parallel root evaluation

## Why

Crunch now has a lazy root-discovery boundary, but multi-root evaluation still
serializes root forcing in the places that matter most:

- `crunch_eval::session::EvaluationSession` exposes `force_root()` and
  `force_all_roots()`, but `force_all_roots()` walks roots one by one.
- `crates/crunch-pipeline/src/lib.rs` still calls
  `session.force_all_roots::<CrunchDerivation>()` before conversion and build
  dispatch.
- Build execution already runs independent derivations in parallel, so a wide
  package set can spend time in a single-threaded eval/convert prefix while the
  worker has nothing to do yet.

That means answer to "do we have parallel eval yet?" is still no. This change
records how to add bounded parallel multi-root evaluation without undoing the
new lazy selected-root boundary.

## What Changes

- define a bounded multi-root evaluation path in `crunch-eval`
- require root-materialization results to stay label-stable and semantically
  equivalent to the current eager path
- require `crunch-pipeline` to stream root forcing/conversion incrementally
  instead of serially forcing every root up front
- add checked-in benchmark evidence for serial-vs-parallel all-root forcing on
  `tests/fixtures/wide_package_set.ncl` while keeping selected-root latency as
  a guardrail

## Capabilities

### New Capabilities

- `parallel-root-materialization`: multi-root callers can materialize more than
  one selected root concurrently under a bounded cap
- `streaming-eval-into-pipeline`: the pipeline can overlap root
  force/convert work with downstream scheduling instead of waiting for a full
  serial eval pass
- `parallel-eval-benchmark-evidence`: checked-in evidence can show whether
  bounded parallel root forcing improves wide package-set throughput

### Modified Capabilities

- `lazy-root-evaluation`: the lazy session boundary now grows from selected-root
  forcing into bounded multi-root forcing for build-oriented callers
- `pipeline-eval-and-convert`: eval/convert no longer has to be one serial
  prefix before build scheduling starts

## Impact

- **Files**:
  - `openspec/changes/parallel-root-evaluation/specs/nickel-eval/spec.md`
  - `openspec/changes/parallel-root-evaluation/specs/pipeline/spec.md`
  - `openspec/changes/parallel-root-evaluation/specs/performance/spec.md`
- **Code**:
  - `crates/crunch-eval/src/session.rs`
  - `crates/crunch-pipeline/src/lib.rs`
  - lazy-eval benchmark/example support under `examples/` and tests
- **APIs**: `crunch-eval` gains a multi-root forcing API; pipeline switches from
  `force_all_roots()` as one serial prefix to a bounded streaming path
- **Evaluator constraint**: the first implementation must treat current Nickel
  evaluator state as likely `!Send` until proven otherwise, so the isolation
  mechanism must be chosen around safe worker boundaries rather than assuming
  `tokio::spawn` over one shared session
- **Testing**: equivalence tests, pipeline integration tests, and wide-fixture
  benchmark evidence for serial vs parallel forcing

## Verification

Before implementation starts, the change MUST stay valid under:

- `openspec validate parallel-root-evaluation`
- proposal, design, and tasks gates for `parallel-root-evaluation`

Implementation acceptance MUST include:

- `crunch-eval` equivalence proof that isolated-worker multi-root forcing
  returns the same typed derivation values as the serial same-session path for
  the same labels
- pipeline integration proof that incremental streaming preserves label ->
  derivation association and stops later dispatch on labeled eval failure
- machine-readable benchmark evidence from `tests/fixtures/wide_package_set.ncl`
  with serial all-root metrics, bounded-parallel all-root metrics, selected-root
  guardrail metrics, and the concurrency value used for the run
- isolated benchmark execution so unrelated Cargo work does not contaminate the
  recorded comparison

## Non-Goals

- replace Nickel with a compiled evaluator or merge this with
  `explore-compiled-eval-backends`
- promise parallel selected-root forcing for every caller; selected-root latency
  remains a guardrail, not the main throughput target here
- require a new user-facing CLI flag in the first iteration
- weaken root-label determinism or typed error reporting in exchange for speed
