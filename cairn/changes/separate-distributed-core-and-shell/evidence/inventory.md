# Separate distributed core and shell: inventory and baseline

Change: `separate-distributed-core-and-shell`
Tasks: I1, I2

## Scope

This evidence maps the distributed realization and external-batch decision
surface to its concrete owners before and after the boundary work. It records
the behavioral baseline that the change must not disturb.

## Decision surface inventory (I1)

Each row names a deterministic decision, its pre-change location, and its
post-change owner.

| Decision | Pre-change owner | Post-change owner |
|---|---|---|
| Remote build-service request admission | `distributed.rs::validate_remote_build_service_request` (reads vendor `snix_build::buildservice::BuildRequest`) | `distributed::core::validate_remote_build_service_request` over `RemoteBuildServiceRequestFacts`; shell projects the vendor request |
| Remote failure-to-fallback classification | `distributed.rs::classify_remote_build_service_failure` | `distributed::core::classify_failure` over `FallbackPolicy`; shell projects policy and maps the disposition |
| Remote goal-attachment planning (dedup + ownership) | `distributed.rs::plan_remote_goal_attachments` | `distributed::core::plan_goal_attachments` over plain string keys; shell keeps the `RealizationKeyDeriver` port call |
| Route eligibility normalization | `distributed.rs::normalize_remote_route_scheduling_facts` (pure, deferred) | unchanged; pure logic already delegates to `scheduling::normalize_eligible_preference` |
| Ready-realization selection | `distributed.rs::select_ready_realizations` (pure, deferred) | unchanged; pure logic already owns the decision |
| External batch planning/reconciliation | `external_batch.rs` pure functions returning `Result<_, String>` (deferred) | unchanged; typed error migration is a listed remaining task |
| Resolver/publisher/realizer capability traits | `distributed.rs` trait declarations alongside adapters (deferred) | unchanged; trait-to-port relocation is a listed remaining task |

## Boundary contract (guarded)

`scripts/check-distributed-core-boundary.rs` enforces the declared pure
boundary over `distributed/core.rs`:

- no vendor types (`snix`, `PathInfo`, `StoreHandle`, `nix_compat`);
- no async runtimes or provider ports (`async fn`, `async_trait`, `tokio`,
  `futures`);
- no host effects (`std::fs`, `std::env`, `std::process`, `std::thread`,
  clocks, `std::net`, printing);
- the `r[impl ...]` and `r[verify ...]` markers for
  `realization_routing.distributed_core_boundary` are present.

The guard strips comments before token matching so prose that names a
forbidden concept does not count as a reference. It carries positive and
negative fixtures and a `--self-test` rail.

## Behavioral baseline (I2)

The change must preserve these accepted behaviors:

- `validate_remote_build_service_request` still rejects raw eval requests and
  output-less requests with `RemoteBuildServiceDispatchError::Phase` and the
  exact reason strings `remote-build-service-raw-eval-request` /
  `remote-build-service-outputs-empty`.
- `classify_remote_build_service_failure` still maps `Never` to
  `ReturnFailure` and `OnRemoteFailure` to `FallbackToLocal`.
- `plan_remote_goal_attachments` still suppresses duplicate realization keys
  behind the first owner, rejects unbounded ready sets with
  `RemoteScheduleError::TooManyReadyGoals`, and preserves input priority
  order.
- Public function signatures and public error types remain byte-compatible so
  `worker.rs`, `orchestrate.rs`, and the root binary compile unchanged.

## Baseline checks

- `cargo test -p crunch-build --lib distributed:: -- --test-threads=1` must
  stay green before and after the split.
- `cargo test -p crunch-build --lib distributed::core:: -- --test-threads=1`
  covers the new pure core positive and negative fixtures.
- `cargo -Zscript scripts/check-distributed-core-boundary.rs --self-test`
  must pass (positive + three negative fixtures).

## Claims

This inventory proves the decision-to-owner mapping and the guarded pure
boundary for the completed slice. It does not claim that every distributed
decision is relocated, that ports are fully separated, or that external batch
errors are fully typed. Those remaining tasks stay open in `tasks.md`.
