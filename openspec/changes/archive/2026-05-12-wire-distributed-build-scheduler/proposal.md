## Why

Provider-neutral distributed build interfaces are now archived and tested, but they are not wired into active scheduling. The next useful step is an opt-in, default-local integration plan that preserves current behavior.

## What Changes

- **Define**: Define how realization keys, resolver dispatch, policy, diagnostics, and scheduler selection connect to `Worker`/orchestration without enabling remote providers by default.
- **Require**: Require default local-only behavior and existing goal dedup/max-jobs semantics to remain unchanged.
- **Keep**: Keep concrete provider implementations out of scope.

## Capabilities

### New Capabilities
- `distributed-scheduler-wiring`: Wire distributed build scheduler seams behind local-only defaults.

## Impact

- **Files**: `crates/crunch-build/src/worker.rs`, `orchestrate.rs`, `distributed.rs`, config/diagnostic surfaces.
- **APIs**: No public API change unless implementation tasks discover a necessary narrow seam.
- **Dependencies**: No new default dependency expected.
- **Testing**: Each task records the smallest relevant command or evidence artifact.
