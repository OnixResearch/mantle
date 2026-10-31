# Proposal: Schedule native goals across remote workers

## Why

Mantle can execute a concrete remote build, but its public build command
selects one endpoint and awaits selected roots serially
(`src/main.rs::remote_build_selection`, `run_remote_build_dispatches_async`).
The ordinary pipeline's remote-first service instead wraps a local bubblewrap
executor (`crates/crunch-pipeline/src/lib.rs:913-930`). Neither path distributes
ready units of one native build across a worker pool.

The lazy Worker already owns dependency readiness, deduplication, job limits,
dynamic-plan admission and waiter completion. Extending that path avoids a
second compiler scheduler or a new execution protocol.

## What Changes

- Add explicit typed pool selection and per-endpoint credential references;
  admit each worker's binding, capabilities and output trust before effects.
  r[distributed_compilation.pool_selection]
- Dispatch independent ready native goals concurrently through the remote
  application ports, including units admitted after a producer completes.
  r[distributed_compilation.goal_dispatch]
- Carry resolved derivation/action identity, complete declared input closure,
  execution policy and expected named outputs to each worker.
  r[distributed_compilation.concrete_payload]
- Use current durable fenced resource reservations and existing hard placement
  gates without inferring capacity. r[distributed_compilation.resource_ownership]
- Admit signed, content-verified results before normal store finalization and
  dependency notification. r[distributed_compilation.output_admission]
- Preserve compiler failure, cancellation, ownership and bounded fallback
  semantics across local and remote execution.
  r[distributed_compilation.terminal_behavior]
- Preserve existing local/single-builder interfaces and expose bounded actual
  per-goal placement observations. r[distributed_compilation.compatibility_evidence]

## Impact

- **Immediate consumers:** the Rust unit-plan lane and explicit C/C++ plans
  in the two dependent frontend changes; ordinary native dependency graphs
  also use the same execution port.
- **Immediate outcome:** one selected build has independent goals executing
  on two admitted workers at the same time.
- **Durable capability:** production multi-worker execution below generic
  native goal scheduling, without compiler-specific worker/store branches.
- **Maintenance owner:** Mantle build and remote-execution owners, covering
  `crunch-build`, `crunch-pipeline`, remote application adapters and CLI policy.
- **Repeatability evidence:** a real two-worker diamond graph, dependency and
  deduplication controls, resource contention, cancellation and hostile results.
- **Compatibility:** remote execution remains opt-in; existing logical store
  identities and supported local/single-builder report contracts remain valid.

## Dependencies

`metadata.json` requires `separate-remote-build-hexagon` and
`resolve-remote-entry-points-through-one-binding`. Their accepted ports and
endpoint-authority contracts must land before this cutover is accepted.
Worker demand, adaptive preference, REAPI, local compile caches and local slots
are not prerequisites. Architecture and the complete dependent change graph
are recorded in [proposed ADR 0138](../../../adr/0138-distribute-compiler-units-through-native-build-goals.md).

## Non-Goals

New transports, a coordinator service, worker provisioning, remote Nickel
evaluation, transparent splitting of opaque compiler commands, shared-cache
trust, speculative duplicate execution, or distributed watch/early-metadata
sessions. Unsupported backend/transport/mode combinations fail preflight.

## Success Criteria

The ordinary native graph owns all goals; two real worker processes overlap
on independent goals, their verified outputs satisfy a dependent goal, and a
shared dependency executes once. Trust/content/fence failures publish nothing;
resource limits and teardown remain bounded. The final qualification change,
not local loopback or these planning artifacts, owns independent-host claims.
