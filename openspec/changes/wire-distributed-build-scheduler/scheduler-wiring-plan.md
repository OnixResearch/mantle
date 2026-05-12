# Distributed scheduler wiring plan

## Current insertion-point trace

### Existing provider-neutral pieces

- `crates/crunch-build/src/distributed.rs` already defines pure distributed seams:
  - `RealizationKeyRequest` and `DefaultRealizationKeyDeriver` for deterministic keys.
  - `ArtifactResolver`, `ArtifactPublisher`, `ResolveOutcome`, and `VerifiedRealizationArtifact` for provider-neutral artifact adapters.
  - `DistributedDiagnostic` receipts for cache hit/miss, local realization, remote candidate/fallback, publish skip, and verification rejection.
- These types are not yet part of `Worker::new`, `Worker::run`, `Builder::build_all`, or `Builder::prepare_build`.

### Scheduler/orchestration flow today

1. `Builder::build_all(roots, known_paths, max_jobs)` constructs `Worker::new(max_jobs)`.
2. Roots enter the scheduler with `Worker::want(root, known_paths, true)`, which deduplicates goals by drv-path key and wires dependency waiters.
3. `Worker::run` and `Worker::run_streaming` repeatedly call `dispatch_ready`.
4. Ready goals call `Builder::prepare_build(drv_path, derivation, known_paths, is_root)`.
5. `prepare_build` first checks existing local/remote PathInfo substitution via `check_cache`.
6. If no existing cache hit is available, `prepare_build` resolves inputs, builds a sandbox `BuildRequest`, and returns `PrepareResult::NeedsBuild`.
7. `Worker` spawns the sandbox build under the existing `max_jobs` semaphore and later calls `Builder::finish_build`.
8. `finish_build` hashes/persists/exports outputs and returns `BuildOutcome`.

### Safe realization-key boundary

The first distributed scheduler seam belongs after existing local/remote PathInfo substitution misses and after the sandbox request facts are known, but before sandbox dispatch:

- too early: `Worker::want` has only derivation graph facts and must keep goal dedup behavior unchanged;
- too late: after spawning the sandbox build, a resolver hit can no longer avoid local execution;
- best first boundary: `Builder::prepare_build` after `derivation_to_build_request(...)`, because sandbox/environment/store facts are normalized enough to derive a key and because `PrepareResult` can still complete synchronously.

### Completion/publish boundary

The optional publish seam belongs after `finish_build` has produced verified `PathInfo` outputs and before returning the root/non-root `BuildOutcome`. Publishing must be best-effort/diagnostic-only until a later provider change defines retry, auth, and remote consistency policy.
