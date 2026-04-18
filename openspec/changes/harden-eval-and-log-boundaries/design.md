# Design: harden eval and log boundaries

## Context

The current lazy session API already distinguishes discovery errors from
successful typed extraction, but parts of the implementation still use
`expect(...)` for shape assumptions in public call paths.

Separately, build-log persistence is treated as best-effort in places where the
operator experience depends on knowing whether a saved log exists. If the write
fails silently, later output can only guess.

## Goals / Non-Goals

**Goals**
- convert user-reachable lazy-eval panic sites into typed errors
- surface build-log persistence failures explicitly
- add regression tests that lock those behaviors in place

**Non-Goals**
- remove every internal assertion in the repo
- change successful log formatting or storage layout
- redesign unrelated operator diagnostics

## Decisions

### 1. Public lazy-eval boundaries return typed errors

**Choice:** public lazy session and root-extraction helpers must return
`crunch-eval::Error` for malformed or inconsistent evaluated shapes instead of
relying on `expect(...)` / `unwrap(...)`.

**Rationale:** callers can recover from a typed error. They cannot recover from
an unexpected process panic triggered by evaluated input or stale session state.

**Implementation:** harden `crates/crunch-eval/src/lib.rs` and
`crates/crunch-eval/src/session.rs` first, keeping `debug_assert!` only for
truly internal impossible-state checks after user-reachable failures have been
translated into typed errors.

### 2. Diagnostic persistence failures become explicit operator facts

**Choice:** when crunch cannot persist a build log, operator-facing output must
report that failure explicitly instead of dropping the error on the floor.

**Rationale:** a missing log path and a failed log write are different facts.
The operator needs to know which one happened.

**Implementation:** plumb write failures from `src/build_cmd.rs` into human and
structured reporting, and ensure saved-log references are emitted only when the
file really exists.

### 3. Regression tests cover malformed shapes and write failures directly

**Choice:** add targeted tests for both classes of failure instead of relying
only on broad end-to-end coverage.

**Rationale:** these bugs live in error paths. Broad happy-path tests are less
likely to catch regressions there.

**Implementation:** add direct unit/integration coverage for malformed lazy
shape handling and for unwritable log directories.

## Verification Strategy

- Prove the panic-free lazy-eval boundary with targeted malformed-shape
  regression tests that return `crunch-eval::Error` and do not panic.
- Prove explicit log-persistence failure handling with targeted tests for
  unwritable or failing log directories, including evidence that no fake
  saved-log path is emitted.
- Prove no broad regression with `cargo test --workspace --lib --tests` under
  the documented build environment.

## Risks / Trade-offs

**[Internal error text may change]** -> Replacing panics with typed errors can
change exact strings in tests or operator output. Mitigation: assert the public
fact pattern, not fragile internal wording, unless wording itself is the
contract.

**[Log persistence can stay non-fatal]** -> Some log-write failures should stay
non-fatal to preserve build result reporting. Mitigation: require explicit
warning/audit output and forbid fake saved-log paths.
