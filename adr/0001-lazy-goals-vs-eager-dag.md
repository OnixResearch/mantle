# ADR 0001: Lazy Goals vs Eager DAG

## Status

Proposed

## Context

crunch aims to replace Nix — not just swap the language (Nickel for Nix)
but own the full stack: evaluation, store, builder, and scheduling. The
current codebase is 9.6k lines of owned code on 42k lines of vendored
snix. The vendored code provides the Nix data model (derivations, store
paths), content-addressed storage (castore), store metadata (PathInfo),
and sandbox execution (bwrap/OCI).

We implemented an eager DAG scheduler (build-dag): full eval → build
explicit graph → Kahn's topological dispatch with concurrent JoinSet.
This works but has structural limits that matter at scale:

1. Eval must complete before any build starts.
2. Can't handle dynamic dependencies (IFD-equivalent).
3. Re-walks the dependency graph that convert() already traversed.

The question: should we replace the eager DAG with a lazy goal-based
scheduler (like Nix's Worker/Goal), and how does this decision interact
with crunch's ambition to own the full stack?

## Decision Drivers

- **snix has no scheduler.** It's recursive async calls with a TODO
  comment. There is no design to adopt or stay compatible with. We're
  building this from scratch regardless.

- **snix's architecture is eval-driven.** `SnixStoreIO` implements
  `EvalIO`, so the evaluator triggers builds via `store_path_to_path_info`.
  crunch doesn't use snix-eval — it uses Nickel. There is no `EvalIO`
  trait to implement. The eval-driven pattern doesn't apply to crunch.

- **Nix's Worker/Goal is the proven architecture** for build scheduling
  in this domain. 15+ years of production use. The goal system handles
  diamond dedup, concurrent dispatch, substitution, IFD, and failure
  propagation.

- **crunch's vendored dependencies are data-layer, not scheduling.**
  nix-compat (derivation format), snix-castore (content-addressed
  storage), snix-store (PathInfo), snix-build (BuildService trait) —
  these are all below the scheduler. A goal-based scheduler sits on top
  of them, same as the eager DAG does. The vendor layer doesn't
  constrain the scheduling architecture.

- **The full-stack ambition means crunch's scheduler is permanent.**
  This isn't a stopgap until snix ships their scheduler. crunch IS the
  scheduler. Getting the architecture right matters more than shipping
  fast.

## Decision

Replace the eager DAG with a lazy goal-based scheduler. Specifically:

- **Goal state machine** with explicit states (Pending → Waiting →
  Ready → Building → Done/Failed). Not coroutines — tokio provides
  async suspension.

- **Worker event loop** that drives goals: dispatch Ready to JoinSet,
  wait for completions, notify waiters, repeat.

- **Lazy creation** via `want(drv_path)`: goals created on demand,
  deduped by store path. Dependencies discovered by inspecting
  `input_derivations`, not by pre-computing a graph.

- **Streaming eval→build** via mpsc channel: evaluation produces
  derivations incrementally, Worker processes them as they arrive.

- **Builder as service container**: Builder owns store services
  (blob, directory, pathinfo, build). Worker owns scheduling. Worker
  borrows &mut Builder for prepare/finish, spawns do_build via
  Arc<BuildService>.

## Alternatives Considered

### Keep the eager DAG

Pro: Already implemented, tests pass, simpler code.
Con: Can't overlap eval+build, can't handle dynamic deps, fundamentally
different from how Nix/snix approach the problem. If crunch replaces
Nix, its scheduler should handle the same workloads.

### Adopt snix's recursive pattern

Pro: Less code, matches snix's current design.
Con: snix's design is explicitly a placeholder ("should be replaced with
a proper scheduler"). No concurrency. No scheduling. Would need to be
replaced anyway.

### Full Nix Worker clone (coroutines, fork+select, goal hierarchy)

Pro: Proven at scale, handles every edge case.
Con: Massive complexity (6 goal types, C++20 coroutines, process-level
FD monitoring). crunch uses tokio, not fork+select. Most of this
complexity solves problems crunch doesn't have yet (substitution goals,
build hooks, per-build timeout monitoring).

## Consequences

- `dag.rs` removed. Goal registry replaces DAG functionality.
- Existing prepare_build/finish_build, Arc<BuildService>, JoinSet,
  Semaphore, --jobs all carry forward.
- The goal system is extensible: substitution goals, remote build
  goals, and IFD-triggered goals can be added later as new GoalState
  variants or goal types.
- **Dynamic derivations are not blocked.** The eager DAG required all
  derivations to be known before building — this structurally
  prevented dynamic derivations (where a build output is a .drv that
  must itself be built). The lazy goal system allows `want()` to be
  called mid-run, KnownPaths to grow during builds, and goals to be
  created after other goals complete. Nix handles this via
  `DerivationTrampolineGoal` with `SingleDerivedPath`; crunch can
  add an `ObtainingDrv` goal state when the feature is needed.
- crunch's scheduler is its own design, not a port of Nix or snix.
  It uses the same conceptual model (lazy goals, waiter notification,
  slot-limited dispatch) but with a Rust/tokio implementation.

## Follow-up: deterministic ready ordering

The original implementation used FIFO insertion order for goals already in
`Ready`. [ADR 0014](0014-deterministic-lazy-goal-priority.md) replaces only that
ready ordering with Mantle's pure bounded priority kernel over explicit
known-graph, policy, age, resource, locality, and stable-identity facts. It does
not revisit this ADR's lazy graph, streaming evaluation, dynamic insertion,
waiter notification, deduplication, or slot-limited dispatch decision.
