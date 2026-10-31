# Baseline behavior record for pool scheduling (2026-10-31)

Scope: source-level facts about the current native-build execution paths and the
prerequisite remote contracts this plan depends on. Tree identity: commit
`7ea0f0969` plus this file. No new build was run for this record, so no runtime
behavior, capacity, or acceptance claim is made.

## Current execution paths

| Path | Location | Observed shape |
| --- | --- | --- |
| Public remote build CLI selection | `src/main.rs:6150 remote_build_selection` | one `--builder`, one endpoint binding verified against the store prefix, one ticket FD, farm profile → `max_concurrency` default `remote_build::DEFAULT_REMOTE_CONCURRENCY = 1` (`src/remote_build.rs:190`) |
| Public remote dispatch | `src/main.rs:7278 run_remote_build_dispatches_async` | iterates `priority_plan.ordered_input_indices` serially, taking one input slot per step, acquiring the coordinator guard and preparing/executing one dispatch at a time |
| Ordinary pipeline remote-first service | `crates/crunch-pipeline/src/lib.rs:940` | `RemoteFirstBuildService::new(remote_realizer, local_bwrap, OnRemoteFailure)` where `remote_realizer` wraps a *local* bubblewrap service (`LocalBuildServiceRealizer`) |
| Watch path | `crates/crunch-pipeline/src/lib.rs create_watch_pipeline_builder` | local sandbox only |

Neither path distributes ready units of one native build across a pool: the CLI
path serializes roots, and the pipeline path's remote side is a local sandbox
realizer with no worker selection.

## Prerequisite contracts that already exist

- Endpoint binding admission: `crunch_remote_core::endpoint` and
  `remote_build::endpoint_config::RemoteEndpointBindingConfig::verify_build`
  (already called by `remote_build_selection`).
- Effect/lease/attempt/identity/resource contracts:
  `crates/crunch-remote-core/src/{effect,lease,attempt,attempt_log,identity,resource,resource_policy,resource_quota}.rs`.
- Worker demand facts (landed by the archived `declare-remote-worker-demand`
  change): `crates/crunch-remote-core/src/demand.rs`.
- Application ports the plan intends to dispatch through:
  `crates/crunch-remote-app/src/lib.rs` —
  `ExecutorPort`, `ExternalBatchPort`, `TransportPort`, `ClientExchangePort`,
  `AttemptPersistencePort`, `AttemptTransitionPort`, `LeasePort`,
  `InputTransferPort`, `OutputTransferPort`, plus store-admission facts.

So the plan's Phase 1 prerequisite contracts are present in the tree; the
change's remaining work is scheduling/dispatch composition rather than new
transport contracts.

## Lazy scheduler surface the plan extends

The ordinary `Worker` already owns dependency readiness, drv-path deduplication,
`MAX_GOALS` bounds, dynamic-plan admission, and waiter completion; the pipeline
comment at `crates/crunch-pipeline/src/lib.rs` ("Keep remote dispatch inside the
lazy scheduler so dedupe, waiter notification, job bounds, fallback, and
terminal propagation stay shared") plus the requirement marker
`r[impl remote_builds.production_scheduler_realization]` record that intent.

## Remaining S1 obligations

- A runtime reproduction of local, single-builder and dynamic-plan behavior with
  exact binary/config identities (requires admitted remote endpoints and a
  ticket, which this workspace does not have in hand).
- Therefore S1 stays unchecked; this record supplies the source-level half.
