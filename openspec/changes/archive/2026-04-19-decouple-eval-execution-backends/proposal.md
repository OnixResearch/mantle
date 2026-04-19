# Decouple eval execution backends

## Why

`crunch-eval` now has a useful lazy multi-root boundary, but the current
implementation couples those semantics to one specific host execution strategy:
a process-local global thread pool in `crates/crunch-eval/src/session.rs`.

That makes three things harder than they should be:

- **modularity**: the eval crate owns worker-pool policy instead of only root
  forcing semantics and worker input shaping
- **composability**: library and pipeline callers cannot cleanly swap between
  serial, threaded, or subprocess execution strategies without editing eval-core
  internals
- **portability**: the portable core should not require background worker
  threads, subprocess self-spawn, or a daemon-like lifecycle just to force
  roots

We also want room to explore command-scoped subprocess workers later without
turning `crunch-eval` into a binary-launching library or committing to a daemon.
This change records the boundary before more performance work piles onto the
current thread-pool shape.

## What Changes

- define a backend-neutral execution boundary for bounded multi-root forcing
  semantics in `crunch-eval`
- require a serial inline backend as the canonical portable fallback
- keep shipped inline and threaded root-force backend implementations in
  `crunch-eval` backend-adapter code while moving runtime selection and
  fallback into host-owned policy layers instead of making either strategy a
  semantic requirement of the eval core
- keep daemon-free operation as a non-negotiable constraint: any subprocess
  path must stay optional and command-scoped rather than requiring a resident
  service
- require pipeline behavior to stay semantically equivalent across shipped eval
  execution backends

## Capabilities

### New Capabilities

- `eval-execution-backend-boundary`: root forcing semantics stay separate from
  the concrete thread/process execution strategy
- `portable-inline-root-forcing`: the eval core can force multiple roots with a
  backend that needs no background worker cache, no daemon, and no subprocesses
- `host-owned-local-eval-executors`: CLI or runtime layers can choose local
  threaded or subprocess execution policy without baking that choice into the
  portable core

### Modified Capabilities

- `parallel-root-materialization`: bounded multi-root forcing becomes a backend
  family instead of one hardcoded worker-pool mechanism
- `pipeline-eval-and-convert`: the pipeline keeps lazy root forcing semantics
  while treating execution strategy as host policy rather than core semantics
- `portability`: eval-layer portability now explicitly covers root-force
  execution strategy, not just build sandboxing

## Impact

- **Files**:
  - `openspec/changes/decouple-eval-execution-backends/specs/nickel-eval/spec.md`
  - `openspec/changes/decouple-eval-execution-backends/specs/pipeline/spec.md`
  - `openspec/changes/decouple-eval-execution-backends/specs/portability/spec.md`
- **Code**:
  - `crates/crunch-eval/src/session.rs`
  - optional new backend-local modules under `crates/crunch-eval/src/`
  - `crates/crunch-pipeline/src/lib.rs`
  - optional CLI/runtime wiring if a non-inline backend is selected by default
- **APIs**: first iteration may add a `crunch-eval` execution-policy carrier
  for inline vs preferred threaded forcing, while `crunch-pipeline` owns the
  shipped runtime selection and inline fallback surface; the main change is
  architectural separation, not a new daemon or required CLI flag
- **Testing**: equivalence tests across inline and any shipped host backends,
  plus portability proof that eval-core still works without thread/process
  helpers

## Verification

Before implementation starts, the change MUST stay valid under:

- `openspec validate decouple-eval-execution-backends`
- proposal, design, and tasks gates for
  `decouple-eval-execution-backends`

Implementation acceptance MUST include:

- proof that the serial inline backend returns the same typed derivation values,
  label ordering, and labeled failures as any shipped threaded or subprocess
  backend for the same requested roots
- proof that `crunch-eval` remains usable without global worker-pool state,
  daemon lifecycle, or subprocess self-spawn
- pipeline proof that converted root outputs stay label-stable regardless of
  which shipped eval execution backend is selected
- pipeline proof that any shipped runtime entrypoint can fall back to the
  required inline backend when a preferred non-inline backend is unavailable,
  unsupported, or not selected
- proof that streaming root forcing still treats each root-force request as an
  independent error-handling unit and does not depend on partial success from a
  failed multi-root request
- if a subprocess backend ships, proof that it remains optional and
  command-scoped rather than becoming a required resident service

## Non-Goals

- choose the final fastest eval execution backend in this change
- require a daemon, resident worker service, or cross-command subprocess pool
- promise that subprocess workers will beat threads on current benchmarks
- move build sandbox portability concerns into the eval crate
- force a new public CLI knob in the first iteration unless implementation
  evidence shows backend selection must be user-visible
