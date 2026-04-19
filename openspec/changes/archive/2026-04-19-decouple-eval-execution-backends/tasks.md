# Tasks: Decouple eval execution backends

## Phase 1: Spec and boundary setup

- [x] Review the existing delta specs for `nickel-eval`, `pipeline`, and
      `portability`, and keep them aligned with the backend seam, inline
      fallback, and host-owned execution policy before coding starts
- [x] Define the intended backend boundary in `crates/crunch-eval` so worker
      input shaping, assignment planning, one-assignment forcing, and ordered
      merge can exist without hardcoding a thread pool in the same path
- [x] Confirm the first iteration keeps daemon-free operation as a hard
      constraint and does not require binary self-spawn from library code

## Phase 2: Portable eval-core refactor

- [x] Extract portable helpers from `crates/crunch-eval/src/session.rs` for:
      immutable worker input construction, bounded assignment planning,
      one-assignment execution, and ordered result merge
- [x] Add a serial inline backend that exercises those helpers without global
      worker-pool state, background threads, subprocesses, daemon lifecycle,
      or binary self-spawn
- [x] Preserve the current root-label order, typed extraction, labeled failure
      semantics, bounded request shaping, and immutable worker-input contract
      through the inline backend
- [x] Add unit and equivalence tests proving the inline backend matches the
      existing same-session semantics for single, array, and record outputs
- [x] Add negative-space proof that eval-core remains usable without mandatory
      global worker-pool state or `current_exe()`-style binary self-spawn

## Phase 3: Host backend adapters

- [x] Move the current thread-backed worker strategy and its existing global
      worker-pool state behind the backend seam so they become one optional host
      execution backend rather than eval-core semantics, and keep cross-thread
      failure transport compatible with the documented `crunch_eval::Error`
      `!Send` constraint
- [x] If local subprocess execution is still desired after the refactor,
      prototype it as an optional command-scoped backend without introducing a
      resident daemon (not shipped in this first iteration)
- [x] If a subprocess backend is prototyped, keep its worker protocol and any
      hidden CLI wiring outside the portable eval core (no subprocess backend
      shipped in this first iteration)
- [x] Add backend-equivalence tests so every shipped non-inline backend matches
      the inline backend for typed results, root order, and labeled failures

## Phase 4: Pipeline integration and validation

- [x] Update `crunch-pipeline` to consume the refactored eval execution
      boundary without changing root -> derivation association semantics,
      without depending on partial success vectors from failed multi-root
      requests, and with a defined fallback to the required inline backend when
      a preferred non-inline backend is unavailable, unsupported, or not
      selected
- [x] Prove pipeline behavior stays stable across any shipped eval execution
      backends for label association, converted derivation semantics, labeled
      failure reporting, and inline-fallback behavior
- [x] Add explicit verification that each root-force request still acts as an
      independent error-handling unit across shipped backends and that failed
      multi-root requests do not leak partial-success dependence into pipeline
      control flow
- [x] Benchmark the inline and any shipped non-inline backends on the checked-in
      wide package-set fixture, while keeping benchmark helpers in example or
      test-only paths and treating semantic equivalence as the primary
      acceptance gate and throughput as secondary evidence
- [x] Validate the change with `openspec validate decouple-eval-execution-backends`
- [x] Run proposal, design, and tasks gates for
      `decouple-eval-execution-backends`
