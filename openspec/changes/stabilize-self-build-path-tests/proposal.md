# Stabilize self-build PATH-sensitive tests

## Why

`cargo test -p crunch -p crunch-pipeline --lib --tests` is currently blocked by
`self_build::tests::resolve_bwrap_source_falls_back_to_path`.

That test branches on `find_host_bwrap()` without first taking `PATH_MUTEX` or
forcing `PATH` to a test-owned directory. On hosts where some other test or the
ambient machine exposes `bwrap`, the "no bwrap" branch becomes non-deterministic
and the test panics before broader validation can continue.

Observed failure:

```text
WARNING: no crunch-built bwrap in /home/...; using external bwrap at /home/.../bwrap
thread 'self_build::tests::resolve_bwrap_source_falls_back_to_path' panicked at src/self_build.rs:2467:17:
should error when no bwrap
```

Nearby tests in the same file already serialize `PATH` mutation and use
fake executables. This one stale case keeps the broader lib/tests run from
reaching the next real blocker.

The broader `cargo test -p crunch -p crunch-pipeline --lib --tests` command is
still the right acceptance path here because the `crunch` package test run
includes the `src/main.rs` unit-test target that owns the `src/self_build.rs`
module tests.

## What Changes

- make the `resolve_bwrap_source(..., Practical)` host-fallback and no-host unit coverage in `src/self_build.rs` own and serialize `PATH`
- replace ambient host probing in that coverage with deterministic tempdir-controlled fallback and no-fallback assertions
- rerun the broader `cargo test -p crunch -p crunch-pipeline --lib --tests` path after the fix so this specific blocker is no longer the stopping point

## Capabilities

### Modified Capabilities

- `self-build-bwrap-validation-determinism`: the `resolve_bwrap_source(..., Practical)` host-discovery tests stop depending on ambient host tool discovery

## Impact

- **Files**: `src/self_build.rs` test module and nearby test helpers if needed
- **Behavior**: no runtime behavior change; validation becomes deterministic
- **Testing**: rerun targeted self-build tests and the broader lib/tests command under the repo's documented Cargo build environment; if either rerun hits `No space left on device`, redirect `TMPDIR`/`CARGO_TARGET_DIR` to disk-backed paths before treating it as acceptance evidence
- **Out of scope**: the later stale `tests/integration_build.rs::eval_hello_world_with_seed` assertion about `bash` vs `/bin/sh`
