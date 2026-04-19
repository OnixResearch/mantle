# Design: Decouple eval execution backends

## Context

The current lazy multi-root path in `crunch-eval` already has a clean semantic
core:

- one coordinator session discovers root shape and labels
- immutable `IsolatedWorkerInput` captures source text, import paths,
  source name, shape, and selected labels
- each worker assignment opens its own `EvaluationSession` and forces one chunk
  of labels

But `crates/crunch-eval/src/session.rs` also owns the current execution policy:
`force_selected_roots_with_workers(...)` talks directly to a global
`OnceLock<Mutex<BTreeMap<u32, Arc<BoundedWorkerPool>>>>`, which owns named
thread workers and queueing policy.

That makes the portable eval core responsible for host scheduling details.
It also blurs two different concerns:

1. **root-force semantics**: label order, typed extraction, labeled failures,
   bounded assignment shaping
2. **local execution strategy**: serial inline calls, local threads,
   command-scoped subprocesses, or some future host-specific adapter

For a project that values modularity, composability, and portability, those two
concerns need different homes.

## Goals / Non-Goals

**Goals:**

- separate root-force semantics from execution mechanism
- require a portable inline backend that works without thread or process help
- leave room for local threaded and subprocess strategies without daemon
  requirements
- keep `crunch-eval` as the semantic boundary and keep host execution policy
  outside that portable core
- preserve existing lazy-root semantics and pipeline label stability

**Non-Goals:**

- commit today to threads or subprocesses as the permanent winner
- introduce a resident daemon or mandatory cross-command worker service
- redesign Nickel evaluation semantics
- change build sandbox portability boundaries
- require immediate public backend-selection CLI flags

## Decisions

### 1. `crunch-eval` owns forcing semantics, not host scheduling policy

**Choice:** `crunch-eval` keeps ownership of worker input shaping, bounded
assignment planning, typed extraction, and labeled error semantics, but it no
longer hardcodes one host worker mechanism as part of those semantics.

**Rationale:** the eval crate is the right place for label order, root-shape,
Nickel error mapping, and worker-input construction. It is the wrong place to
force one host execution primitive onto every caller.

**Implementation:** keep helpers such as worker-input construction,
assignment building, one-assignment execution, and ordered-result merge near
`EvaluationSession`. The backend seam is a caller-provided parameter to the
multi-root forcing entry point, not a global or module-level default. Backends
may return per-assignment results in completion order; ordered-result merge
stays in eval-core so label-order semantics do not become backend-specific.
Move thread-pool or subprocess orchestration behind a backend seam that
consumes those helpers.

### 2. Serial inline execution is the required baseline backend

**Choice:** the architecture requires a serial inline backend that executes the
same worker assignments in-process without background threads, subprocesses, or
global worker caches.

**Rationale:** this is the portability floor. If the semantics only work when a
host can spawn threads or processes, the eval core is not really portable or
composable.

**Implementation:** the serial backend runs assignment execution directly in the
caller thread. It may still reuse immutable worker input and assignment shaping,
but it must need no global state beyond ordinary function-local data.

### 3. Threaded and subprocess execution are optional host backends

**Choice:** local threaded and local subprocess execution stay optional backend
implementations layered above the portable forcing core.

**Rationale:** those strategies may be useful optimizations on ordinary host
builds, but they are not universal requirements for every embedding target or
library caller.

**Implementation:** the host runtime may choose a default backend, but every
shipped backend must consume the same immutable worker input and preserve the
same typed results and labeled failures as the inline path.

### 4. No daemon becomes a hard architectural constraint

**Choice:** no eval execution backend may require a resident daemon or
cross-command worker service.

**Rationale:** a daemon would hurt composability, complicate operator mental
models, and make the eval layer feel heavier than the problem warrants.

**Implementation:** if a subprocess backend ships, it must stay optional and
command-scoped. Acceptable shapes include one-shot helper children or a pool of
children owned only for the lifetime of the current `crunch` command. A
resident background service is out of scope.

### 5. `crunch-eval` must not require binary self-spawn in library mode

**Choice:** the portable eval crate cannot rely on `current_exe()` or an
assumption that another `crunch` binary is available.

**Rationale:** library callers, tests, embedded environments, and non-process
platforms may not have a meaningful self-spawn path. Making that mandatory
would collapse portability.

**Implementation:** subprocess execution, if present, belongs in a host layer
that can explicitly opt into a hidden worker protocol. The core backend seam
must still function without any executable-launch path.

### 6. Pipeline treats eval execution as host policy, not semantic drift

**Choice:** `crunch-pipeline` may choose or receive a local eval execution
backend, but it must keep the same root-label and conversion semantics across
all shipped backends.

**Rationale:** pipeline callers care about correct root -> derivation -> output
association, not about whether roots were forced inline, on threads, or through
local helper processes.

**Implementation:** the pipeline continues to rely on `crunch-eval` for lazy
root discovery and forcing semantics. Any backend-specific choice happens at a
policy boundary and must not leak new label-order or partial-result semantics
into conversion or worker dispatch.

### 7. Verification is equivalence-first, benchmark-second

**Choice:** first acceptance proof is semantic equivalence against the inline
backend; throughput comparisons come second.

**Rationale:** backend modularity only helps if alternate strategies remain
behaviorally identical. Fast wrong answers or backend-specific error/reporting
shapes would damage composability.

**Implementation:** every shipped backend must pass the same root-order,
typed-derivation, and labeled-failure tests. Performance work may compare
threaded and subprocess strategies later, but the spec does not assume either
one wins.

## Verification Strategy

- unit-test assignment shaping and ordered-result merge without threads or
  processes
- equivalence-test inline backend against each shipped non-inline backend on:
  - single-root forcing
  - multi-root package-set forcing
  - nested derivation inputs
  - labeled root failures
- integration-test pipeline output association so root labels and converted
  outputs stay stable across backend selection
- if a subprocess backend ships, add proof that command-scoped workers shut
  down with the command and do not require a resident service
- keep benchmark evidence separate from semantic acceptance so backend modular
  changes do not overfit to one fixture

## Risks / Trade-offs

**[Extra abstraction with no speed win]**
A backend seam could add complexity before it shows throughput benefit.

**Mitigation:** keep the seam narrow and semantics-focused, with the inline
backend as the simplest reference path.

**[Accidental binary coupling]**
A subprocess experiment could leak `current_exe()` assumptions into library
code.

**Mitigation:** keep subprocess orchestration outside the portable core and
require inline-only operation as the baseline acceptance path.

**[Backend-specific semantic drift]**
Different execution strategies could reorder labels or surface different failure
shapes.

**Mitigation:** make inline-backend equivalence mandatory for all shipped
backends.

**[Portability regression through hidden globals]**
Even without threads, a new backend seam could still leave mandatory global
state in the eval crate.

**Mitigation:** require the serial backend to work without global worker caches
or daemon lifecycle assumptions.
