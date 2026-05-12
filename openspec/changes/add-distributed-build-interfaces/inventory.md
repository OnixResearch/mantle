# Distributed Build Interface Inventory

Task: I1 — inventory current build scheduler, PathInfo lookup, substitution, store-push, and build-finalization seams; record concrete modules that will host each interface without adding provider dependencies.

## Current seams

### Scheduler / goal ownership

- `crates/crunch-build/src/goal.rs`
  - Pure goal state machine: `Goal`, `GoalState`, and `GoalRegistry` own readiness, waiting, waiter notification, terminal states, and deduplication by derivation path.
  - Keep this module provider-free. It should not learn about resolvers, publishers, remote workers, URLs, or network state.
- `crates/crunch-build/src/worker.rs`
  - Imperative scheduler shell. It owns `GoalRegistry`, `ready_queue`, `max_jobs`, `Semaphore`, `JoinSet`, and pending prepared-build metadata.
  - Current dispatch path calls `Builder::prepare_build()`, spawns sandbox builds for `PrepareResult::NeedsBuild`, calls `Builder::finish_build()`, and then notifies waiters.
  - Host future `RealizationPolicy` call sites here only at the point a ready goal has already passed scheduler readiness/deduplication. The policy may select a realizer, but `Worker` must continue to own readiness, deduplication, waiter propagation, terminal states, and `max_jobs` bounds.

### Build pipeline / local realization

- `crates/crunch-build/src/orchestrate.rs`
  - `Builder::prepare_build()` is the current pre-dispatch pipeline: cache check, input-node materialization, source resolution/ingestion, sandbox input collection, build request creation, CA rewrite metadata, and `PrepareResult` production.
  - `PrepareResult::{Done, NeedsBuild}` is the narrow existing seam where resolver hits can continue to return `Done` and misses can proceed toward realizer selection.
  - `PreparedBuild` is the metadata bundle a realizer needs today for the existing local sandbox path plus later finalization.
  - `Builder::finish_build()` is the current post-dispatch finalization path: consume `snix_build::buildservice::BuildResult`, process outputs, rewrite CA outputs, persist signed PathInfo, export roots, and return `BuildOutcome`.
  - Host future distributed-build core types in a new provider-neutral module in this crate, e.g. `crates/crunch-build/src/distributed.rs`, and re-export from `lib.rs`. Keep concrete provider clients out of `crunch-build` default dependencies.

### PathInfo lookup and substitution

- `crates/crunch-build/src/orchestrate.rs::check_cache()`
  - Delegates to `StoreHandle::check_cache()`, converts cache hits into `CacheCheckHit`, records substitution reports, and performs signature validation unless `trust_unsigned` is set.
  - This is the build-pipeline adapter point for `ArtifactResolver`: initially wrap existing `StoreHandle::check_cache()` as the local/substitution resolver without changing default behavior.
- `crates/crunch-store/src/handle.rs`
  - `StoreHandle::check_cache()` checks local PathInfo/castore content and can fall back to remote binary-cache substitution.
  - `StoreHandle::try_substitute_remote()` owns current remote binary-cache lookup/adoption: it reads remote PathInfo, persists accepted PathInfo locally, caches output nodes, and records substitution reports.
  - Keep binary-cache protocol specifics in `crunch-store`; expose them to `crunch-build` through a provider-neutral resolver adapter, not through scheduler-specific remote-cache conditionals.
- `crates/crunch-store/src/pull.rs`
  - Existing explicit pull/import path for binary-cache directories and HTTP cache sources. This is not the scheduler resolver path, but it is an adapter implementation reference for future resolver tests and transport boundaries.

### Store push / publication

- `crates/crunch-store/src/push.rs`
  - `export_paths_to_cache_dir()` exports signed PathInfo as narinfo plus NAR files to a flat binary cache directory.
  - This is the initial local/directory `ArtifactPublisher` implementation boundary. The trait should accept only verified artifacts/PathInfo supplied after finalization; it should not be called on raw build output candidates.
- `crates/crunch-store/src/handle.rs`
  - `persist_and_export_signed_output()` enforces the storage boundary for local builds: sign before PathInfo persistence, persist artifact attestation, export root outputs, and record GC roots.
  - Publisher adapters should sit after this verified boundary unless they are explicitly modeling a verification candidate for future remote realization.

### Remote realization candidate verification

- `crates/crunch-build/src/orchestrate.rs::finish_build()` and `process_output()` remain the canonical local verification/finalization path for local sandbox outputs.
- Future remote realization should return a provider-neutral candidate result shape into `crunch-build` and then pass through an equivalent verifier/finalizer before PathInfo persistence.
- Remote candidate structs should live in `crates/crunch-build/src/distributed.rs` first. Transport-specific adapters should live outside core defaults or behind optional feature-gated crates/modules.

## Interface hosting plan

| Interface / model | Initial host | Existing seam it wraps | Provider dependency rule |
| --- | --- | --- | --- |
| `RealizationKeyDeriver`, `RealizationKeyRequest`, `RealizationKey` | `crates/crunch-build/src/distributed.rs` | pure facts from `Derivation`, input closure, store prefix, sandbox/hermeticity, platform, toolchain/profile data | no provider deps; deterministic serde/hash only |
| `ArtifactResolver` | `crates/crunch-build/src/distributed.rs` trait; first adapter in `crates/crunch-build/src/distributed.rs` or `crates/crunch-store` wrapper | `Builder::check_cache()` → `StoreHandle::check_cache()` | no remote SDK in core; existing binary-cache support remains in `crunch-store` |
| `ArtifactPublisher` | `crates/crunch-build/src/distributed.rs` trait; directory-cache adapter wraps `crates/crunch-store/src/push.rs` | post-`persist_and_export_signed_output()` verified PathInfo | publishers accept verified artifacts only |
| `DerivationRealizer` | `crates/crunch-build/src/distributed.rs` trait | current `build_service.do_build(build_request)` local sandbox call from `Worker` | local sandbox realizer required; remote realizers optional adapters |
| `RealizationPolicy` | `crates/crunch-build/src/distributed.rs` | ready-goal dispatch in `Worker::dispatch_ready()` | data-driven, no provider-specific branches in scheduler |
| `RealizationDiagnostic` / receipt events | `crates/crunch-build/src/distributed.rs`; surfaced through `BuildOutcome` later | existing tracing/cache/substitution messages and `BuildOutcome` | provider-neutral event kinds only |
| Remote candidate result / verifier input | `crates/crunch-build/src/distributed.rs` | `finish_build()`/`process_output()` verification/finalization equivalence | candidate is untrusted until verified |

## Default behavior guardrails

- Default builds remain local-only: no resolver, publisher, or remote realizer profile is active unless configured.
- Existing `StoreHandle::check_cache()` substitution remains the only default cache lookup path used by builds.
- No scheduler code may depend on concrete provider crates, endpoint formats, authentication, retry policy, Cargo wrappers, Buck/Bazel semantics, or remote worker protocols.
- Remote or external adapters may be registered later, but policy selection must fail closed when requested capabilities are unavailable.

## Evidence

Inspected on 2026-05-12:

- `crates/crunch-build/src/worker.rs`: worker comments and structure show ready-goal dispatch through `Builder::prepare_build()`, sandbox spawning on `JoinSet`/`Semaphore`, `max_jobs`, and waiter completion ownership.
- `crates/crunch-build/src/goal.rs`: pure `Goal`/`GoalRegistry` state transitions with no I/O or builder references.
- `crates/crunch-build/src/orchestrate.rs`: `BuildOutcome`, `PreparedBuild`, `PrepareResult`, `Builder::prepare_build()`, `Builder::finish_build()`, and `Builder::check_cache()` are the current resolver/realizer/finalizer seams.
- `crates/crunch-store/src/handle.rs`: `StoreHandle::check_cache()`, `try_substitute_remote()`, and `persist_and_export_signed_output()` define current PathInfo lookup/substitution/persistence boundaries.
- `crates/crunch-store/src/push.rs`: `export_paths_to_cache_dir()` is the existing verified PathInfo publication/export path.
- `crates/crunch-store/src/pull.rs`: cache-dir/HTTP import path documents transport-specific pull behavior that must stay adapter-owned.
