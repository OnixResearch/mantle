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

## Default-local wiring shape

### Configuration object

Add a small `DistributedSchedulerConfig` owned by `crunch-build` with these defaults:

```rust
pub struct DistributedSchedulerConfig {
    pub mode: DistributedSchedulerMode,
    pub resolver_name: Option<String>,
    pub publisher_name: Option<String>,
}

pub enum DistributedSchedulerMode {
    LocalOnly,
    ResolveThenLocalFallback,
}
```

`Default::default()` MUST be `LocalOnly` with no resolver or publisher. `Builder::build_all` and existing `Worker::new(max_jobs)` callers should continue using that default until CLI/config plumbing explicitly opts in.

### Builder-held integration point

Keep provider adapters on `Builder`, not `Worker`:

- `Worker` remains the goal scheduler and keeps drv-path goal dedup/max-jobs semantics unchanged.
- `Builder::prepare_build` has the normalized facts needed to derive a `RealizationKey` and can return a synchronous `Done(BuildOutcome)` on resolver hit.
- `Builder::finish_build` already owns verified output `PathInfo` values and can attempt optional publish without changing worker concurrency accounting.

### Resolver decision contract

For `LocalOnly`:

1. Do not derive a distributed realization key.
2. Do not call an artifact resolver.
3. Do not emit distributed diagnostics unless an explicit diagnostics sink asks for `local-only-disabled` receipts in a later change.
4. Continue the existing local cache/fetcher/sandbox path exactly as today.

For `ResolveThenLocalFallback`:

1. Derive `RealizationKey` after `derivation_to_build_request(...)` succeeds.
2. Call the configured `ArtifactResolver` once before sandbox dispatch.
3. On `ResolveOutcome::Hit(artifact)`, verify/adapt outputs into the same `BuildOutcome` shape used by cache hits, record `DistributedDiagnostic::CacheHit`, and complete the goal synchronously.
4. On `Miss`, `Unavailable`, adapter error, or verification rejection, record `CacheMiss`, `RemoteFallback`, or `VerificationRejected` and continue to local sandbox dispatch.
5. Never consume a `max_jobs` semaphore permit for a resolver hit; only local sandbox execution remains semaphore-gated in the first wiring slice.

### Diagnostic receipts

Reuse `DistributedDiagnostic::receipt()` as the stable operator-facing format. Required first-slice events:

- resolver hit: `CacheHit { key, resolver }`;
- resolver miss: `CacheMiss { key, resolver }`;
- resolver unavailable or rejected: `RemoteFallback` or `VerificationRejected`;
- local fallback after an opt-in decision: `LocalRealization { key, realizer: "local-sandbox" }`;
- publisher skipped by default: no event in `LocalOnly`; explicit opt-in with no publisher may use `PublishSkipped`.

### Out of scope for implementation slice

- provider-specific auth, discovery, network clients, retry policy, and remote execution;
- scheduling remote jobs concurrently with local jobs;
- changing root/dependency goal identity away from drv path;
- enabling resolver/publisher behavior by default.

## Follow-up implementation task cut

A later implementation drain should keep the first code slice narrow and testable:

1. Add `DistributedSchedulerConfig`/`DistributedSchedulerMode` to `distributed.rs` with `Default = LocalOnly` and unit tests that prove no resolver/publisher names are set by default.
2. Add optional distributed fields to `Builder` plus constructor/builder methods that preserve existing `Builder::new*` defaults.
3. Add a pure helper that converts a `BuildRequest`/derivation/store facts into `RealizationKeyRequest`; test deterministic ordering and rejection of incomplete facts.
4. In `Builder::prepare_build`, after `derivation_to_build_request(...)`, call the resolver only when `mode == ResolveThenLocalFallback`; add tests with a fake resolver for hit, miss, unavailable, and rejected-artifact fallback.
5. Adapt `VerifiedRealizationArtifact` into `BuildOutcome` on resolver hit and record output paths in `DerivationRegistry` the same way cache hits do.
6. Add a diagnostic sink/receipt collector on `Builder`; assert `LocalOnly` emits no distributed receipts and opt-in miss/fallback emits the expected redacted `DistributedDiagnostic::receipt()` lines.
7. Keep `Worker::new`, `Worker::want`, goal keys, and semaphore `max_jobs` behavior unchanged; add a regression that resolver hit completes synchronously without spawning a sandbox build.
8. Run `cargo test -p crunch-build distributed`, the focused worker/orchestration tests touched by the seam, `cargo fmt`, and `openspec validate --all --strict`.
