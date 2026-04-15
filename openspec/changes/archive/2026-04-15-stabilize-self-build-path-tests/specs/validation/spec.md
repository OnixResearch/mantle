## ADDED Requirements

### Requirement: Self-build PATH-sensitive validation owns host PATH

Self-build validation that exercises host `bwrap` discovery MUST not depend on
the ambient machine `PATH`.

Any test that expects a host-fallback or no-host-bwrap outcome MUST serialize
`PATH` mutation, set `PATH` to test-owned directories, and restore the prior
environment through a panic-safe scope-exit mechanism before the test scope
unwinds.

#### Scenario: No-host-bwrap case uses empty controlled PATH

- GIVEN a self-build validation case that expects `resolve_bwrap_source(..., Practical)` to fail because no host `bwrap` is available
- WHEN the test runs
- THEN it holds the shared PATH serialization guard
- AND it sets `PATH` to an empty temp directory under test control
- AND the assertion does not depend on whichever `bwrap` binaries happen to exist on the contributor host

#### Scenario: Targeted regression rerun passes the controlled cases

- GIVEN a contributor runs `cargo test -p crunch --bin crunch resolve_bwrap_source_ -- --nocapture` under the repo's documented Cargo build environment
- WHEN the targeted self-build regression tests execute
- THEN the controlled host-fallback and no-host cases pass
- AND the output does not contain the panic text `should error when no bwrap`
- AND if the rerun fails with `No space left on device`, the contributor reruns with disk-backed `TMPDIR` and `CARGO_TARGET_DIR` before treating the result as acceptance evidence

#### Scenario: Host-fallback case uses fake controlled PATH

- GIVEN a self-build validation case that expects host fallback tool discovery
- WHEN the test runs
- THEN it holds the shared PATH serialization guard
- AND it places a fake `bwrap` executable in a test-owned temp directory
- AND it sets `PATH` so the assertion exercises only that fake binary
- AND it restores the prior `PATH` when the test finishes

### Requirement: Broader lib/tests rerun is no longer blocked by ambient PATH discovery

The repo MUST keep the broader `cargo test -p crunch -p crunch-pipeline --lib --tests`
validation path from failing because `resolve_bwrap_source_falls_back_to_path`
observed an unexpected host `bwrap`.

#### Scenario: Broader lib/tests rerun advances past the self-build PATH flake

- GIVEN the self-build PATH-sensitive validation fix is in place
- AND the broader command includes the `crunch` package unit-test target that owns `src/self_build.rs`
- AND a contributor reruns `cargo test -p crunch -p crunch-pipeline --lib --tests` under the repo's documented Cargo build environment
- WHEN the broader validation command executes
- THEN that command does not fail because `resolve_bwrap_source_falls_back_to_path` saw an ambient host `bwrap`
- AND the output does not contain the panic text `should error when no bwrap`
- AND if a later failure remains, the contributor records the exact next failing test/assertion outside that PATH-control bug
- AND if the rerun fails with `No space left on device`, the contributor reruns with disk-backed `TMPDIR` and `CARGO_TARGET_DIR` before treating the result as acceptance evidence
