# Design: Stabilize self-build PATH-sensitive tests

## Context

`resolve_bwrap_source()` intentionally consults host `PATH` in practical mode
when no crunch-built `bwrap` exists yet. The runtime behavior is correct, but
its unit coverage is not fully isolated from ambient host state.

Most PATH-sensitive tests in `src/self_build.rs` already take `PATH_MUTEX`, set
`PATH` to temp directories they control, and restore the prior value. The
remaining `resolve_bwrap_source_falls_back_to_path` test still branches on the
real host environment. That makes the broader validation result depend on which
`bwrap` happens to be discoverable on the machine and on what neighboring tests
have done with process-global `PATH`.

## Goals / Non-Goals

**Goals:**

- make self-build PATH-sensitive test outcomes deterministic
- keep positive and negative coverage for `resolve_bwrap_source(..., Practical)`
- unblock broader lib/tests validation so later failures become visible

**Non-Goals:**

- change runtime `resolve_bwrap_source()` behavior
- redesign self-build bootstrap tool selection
- fix unrelated later test failures such as the stale hello-world builder assertion

## Decisions

### 1. Stop branching on ambient host discovery inside the test

**Choice:** replace the current host-probing branch with deterministic setup
that controls `PATH` explicitly.

**Rationale:** a unit test should decide whether host fallback exists. It should
not ask the machine and then try to adapt.

**Implementation:** use test-owned temp directories for both cases: one empty
`PATH` for the no-host-bwrap assertion and one fake `bwrap` executable for the
host-fallback assertion.

### 2. Reuse the existing PATH serialization pattern, with panic-safe restore

**Choice:** PATH-sensitive coverage keeps using the shared `PATH_MUTEX`, and it
must restore `PATH` through a panic-safe guard/helper owned by the test code.

**Rationale:** `PATH` is process-global. A manual single-exit cleanup path is
not strong enough for this bug class because the observed failure mode is a
panic inside the test body.

**Implementation:** every test that mutates or relies on `PATH` for this code
path takes `PATH_MUTEX`, snapshots the old value, sets a controlled `PATH`, and
restores it through the panic-safe helper before the test scope unwinds.

### 3. Map each spec scenario to a concrete test case

**Choice:** the delta-spec scenarios map directly to concrete coverage in
`src/self_build.rs`.

**Rationale:** proposal/spec/design drift is less likely when each scenario has
a named test target.

**Implementation:**

- `No-host-bwrap case uses empty controlled PATH` -> a deterministic negative
  test for `resolve_bwrap_source(..., Practical)` with `PATH` set to an empty
  temp directory.
- `Host-fallback case uses fake controlled PATH` -> a deterministic positive
  test for `resolve_bwrap_source(..., Practical)` with a fake `bwrap`
  executable in a test-owned temp directory.
- `Broader lib/tests rerun advances past the self-build PATH flake` -> the
  acceptance rerun of `cargo test -p crunch -p crunch-pipeline --lib --tests`.

### 4. Keep scope on the current blocker only

**Choice:** this change stops after the self-build PATH-sensitive blocker is
removed and the next broader test result is exposed.

**Rationale:** the next observed failure (`eval_hello_world_with_seed`) has a
separate root cause and should not be folded into this change.

**Implementation:** rerun the broad lib/tests command after the fix and, if it
still goes red, record the exact next failing test/assertion outside this
change.

## Verification

- Structural check: inspect the touched `src/self_build.rs` tests and confirm
  each affected host-fallback / no-host case (1) takes `PATH_MUTEX`, (2) points
  `PATH` only at test-owned temp directories, and (3) restores `PATH` through
  the panic-safe helper.
- Targeted command: `cargo test -p crunch --bin crunch resolve_bwrap_source_ -- --nocapture`
  - run it under the repo's documented Cargo build environment.
  - expected result: the controlled host-fallback and no-host cases pass and
    output does not contain `should error when no bwrap`.
  - triage note: if the rerun fails with `No space left on device`, treat that
    as environment noise and rerun with disk-backed `TMPDIR` and
    `CARGO_TARGET_DIR` before using the result as acceptance evidence.
- Broader command: `cargo test -p crunch -p crunch-pipeline --lib --tests`
  - run it under the repo's documented Cargo build environment.
  - expected result: the run does not stop on
    `resolve_bwrap_source_falls_back_to_path`; if a later failure remains, record
    that exact next failing test/assertion explicitly.
  - triage note: if the rerun fails with `No space left on device`, treat that
    as environment noise and rerun with disk-backed `TMPDIR` and
    `CARGO_TARGET_DIR` before using the result as acceptance evidence.

## Risks / Trade-offs

**Another PATH-sensitive test may still leak ambient state**

That risk already exists, but fixing the currently failing case removes the
known blocker first. The broader rerun will show whether more cleanup is needed.
