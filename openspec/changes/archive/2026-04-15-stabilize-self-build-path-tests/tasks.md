## Phase 1: Stabilize self-build PATH coverage

- [x] Update `resolve_bwrap_source_falls_back_to_path` so it locks `PATH_MUTEX`, saves/restores `PATH`, and never inspects ambient host `PATH`
- [x] Add explicit deterministic coverage for both host-fallback-present and no-host-bwrap cases using test-owned temp directories, `PATH_MUTEX`, and panic-safe `PATH` restore in every affected test
- [x] Keep assertions concrete: host-fallback case checks the returned fake path, and no-host case checks the missing-bwrap error kind/message fragment expected by the current `resolve_bwrap_source(..., Practical)` contract

## Phase 2: Verify blocker removal

- [x] Inspect the touched `src/self_build.rs` tests and confirm every affected host-fallback / no-host case takes `PATH_MUTEX`, points `PATH` only at test-owned temp directories, and restores `PATH` through the panic-safe helper
- [x] Run `cargo test -p crunch --bin crunch resolve_bwrap_source_ -- --nocapture` under the repo's documented Cargo build environment, confirm the command passes, confirm both controlled cases pass, and confirm output does not contain `should error when no bwrap`; if it fails with `No space left on device`, rerun with disk-backed `TMPDIR`/`CARGO_TARGET_DIR` before using the result as evidence
- [x] Run `cargo test -p crunch -p crunch-pipeline --lib --tests` under the repo's documented Cargo build environment, confirm output does not contain `should error when no bwrap`, and, if it still goes red, record the exact next failing test/assertion outside this change; if it fails with `No space left on device`, rerun with disk-backed `TMPDIR`/`CARGO_TARGET_DIR` before using the result as evidence

## Validation

- [x] Run `openspec validate stabilize-self-build-path-tests`
