# Avoid tmpfs validation failures

## Why

The checked-in self-hosting proof helper can fail with `No space left on
device` for environment reasons instead of code regressions. On this host,
`/tmp` filled completely while the main filesystem still had hundreds of GiB
free, so fresh proof-oriented validation failed only because large compiler
artifacts and temp files were routed through tmpfs-backed scratch paths.

Crunch needs the checked-in proof helper to choose disk-backed scratch by
default, name a single override interface, and fail fast with actionable
diagnostics when the selected scratch filesystem is too small.

## What Changes

- make `scripts/prove-self-hosting.sh` resolve scratch from `CRUNCH_PROOF_SCRATCH_DIR` first, else a repo-local disk-backed path under `target/self-hosting-proof/work/`
- route both `TMPDIR` and `CARGO_TARGET_DIR` through subdirectories of the selected scratch root
- fail before the long proof run starts when the selected scratch filesystem has less than 4 GiB free
- document direct heavyweight `cargo` guidance for disk-backed scratch without replacing the proof helper as the canonical self-hosting entry point

## Capabilities

### New Capabilities

- `proof-scratch-root-selection`: the checked-in proof helper avoids ambient tmpfs defaults
- `proof-scratch-capacity-preflight`: the helper reports selected scratch paths and blocks clearly undersized filesystems before the long proof run

## Impact

- **Files**: `scripts/prove-self-hosting.sh`, self-hosting proof docs, and related regression tests
- **Behavior**: the checked-in proof helper stops relying on ambient `/tmp`
- **Testing**: add coverage for default scratch selection, `CRUNCH_PROOF_SCRATCH_DIR`, `TMPDIR`/`CARGO_TARGET_DIR` placement, and low-space diagnostics
