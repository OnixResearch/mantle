# ADR 0138: Distribute completed compiler units through native build goals

## Status

Proposed (2026-10-26). This record and the four linked Cairns are an
implementation plan, not implemented distributed compilation or accepted
execution evidence. All implementation and qualification tasks start unchecked.
No spec sync, archive, default change, or release claim is authorized here.

## Context

Mantle has native remote execution, bounded transfer, worker placement,
resource leases, signed output admission, and lazy derivation goals. These
pieces are not yet one multi-worker compilation path:

- `src/main.rs::remote_build_selection` selects one endpoint;
  `run_remote_build_dispatches_async` awaits each selected root in a loop.
- `crates/crunch-pipeline/src/lib.rs::create_pipeline_builder_with_source_policy`
  constructs `LocalBuildServiceRealizer` over a local bubblewrap service even
  at its remote-first dispatch seam. Its name is not remote execution evidence.
- `crates/mantle-cargo-unit-plan` and `lib/cargo_unit_plan.ncl` already lower
  Rust units into ordinary native dynamic plans. Their accepted specification
  keeps Rust semantics out of the generic worker and store.
- `examples/build-from-source.ncl` has independent `greet.o` and `main.o`
  compilations inside one opaque Make derivation. The C/C++ driver classifier
  exists in `crunch-rust-cache-core::cc`, but its local cache/slot daemon is
  neither remote execution authority nor a compilation graph.

The existing stdio loopback proves only its recorded local composition. It
cannot establish independent-machine execution or parallel compiler work.

## Decision Drivers

- Distribute work within one selected build, not just whole roots.
- Reuse native goal, resource, transfer, attempt, and output-admission contracts.
- Keep compiler semantics in frontends and all effects outside pure cores.
- Keep transport credentials, cache reuse, scheduling, and output trust separate.
- Require finished verified artifacts before dependent units can run.
- Preserve ordinary local and single-builder workflows and bounded evidence.

## Decision

Use one local lazy goal graph with an explicitly selected native worker pool.
Ready concrete derivations, including dynamically admitted units, go through
an application-owned remote execution port. Worker placement follows existing
hard admission gates and durable fenced reservations; no resource quantities
are inferred from concurrency or worker presence. The coordinator is not an
output-trust root. Artifact/action identity excludes placement and attempt
metadata, but includes semantic tools, inputs, arguments, environment, platform,
network and sandbox policy.

Rust uses the existing Cargo unit-plan producer/helper and completed dependency
outputs. Proc macros, build scripts, generated files and target linkers retain
their declared host/target and closure dependencies. No live compiler process,
incremental directory or provisional metadata moves between workers.

C/C++ gets an opt-in explicit unit-plan frontend: source/header trees and
compiler/sysroot inputs, generator edges, one object action per translation
unit, and ordinary archive/link/install actions. The first consumer is the
existing `greet` workload from `examples/build-from-source.ncl`, preserving
its ordinary Make entry point as an independent comparison lane. This is not
transparent interception of arbitrary Make/Ninja invocations. Full declared
include roots are inputs; a depfile learned by an earlier compilation is not
sufficient authority for the next execution's input closure.

The existing local compiler cache/slot socket remains local-only. It is not
extended into an execution or store socket inside a sandbox. Native workers
receive concrete inputs, not Nickel source, ambient PATH, secret files, or
permission to discover host toolchains. Existing native framing is reused;
REAPI, distcc, new transports, provisioning and cross-host live pipelines are
not prerequisites for this work.

### Implementation order and ownership

| Change | Outcome | Hard prerequisites | Maintenance owner |
| --- | --- | --- | --- |
| [schedule-native-goals-across-workers](../.cairn/changes/schedule-native-goals-across-workers/proposal.md) | Concurrent native pool dispatch from the ordinary lazy graph | `separate-remote-build-hexagon`, `resolve-remote-entry-points-through-one-binding` | Mantle build/remote owners |
| [distribute-rust-compilation-units](../.cairn/changes/distribute-rust-compilation-units/proposal.md) | One Rust unit-plan build uses multiple workers | Pool dispatch, `build-cargo-units-as-dynamic-plans`, `run-cargo-build-scripts-as-plan-units` | Mantle Rust-planning owner |
| [distribute-cc-compilation-units](../.cairn/changes/distribute-cc-compilation-units/proposal.md) | Explicit C/C++ compile graphs use the same pool | Pool dispatch | Mantle native-build frontend owner |
| [qualify-distributed-compilation](../.cairn/changes/qualify-distributed-compilation/proposal.md) | Repeatable independent-host acceptance and measured limits | Both compiler frontends | Mantle build/evidence owners |

The two compiler frontends can proceed independently after pool dispatch.
The dependency metadata, not this table alone, must enforce these edges.
Worker-demand publication, adaptive resource preference, local object caching,
machine-wide local slots, warm Rust workspaces and early-interface pipelining
remain with their existing Cairns. Their delivery is not required by this
finished-artifact design. Reusing the C classifier does not require accepting
the local cache change's separate performance or protected-bootstrap claims.

## Alternatives Considered

- **Send the whole project to one builder:** retains the current granularity;
  independent units still cannot use different workers.
- **New compiler RPC, daemon or distributed cache:** duplicates native execution,
  fencing and trust boundaries. Cache authorization cannot authorize execution.
- **Ambient distcc/sccache injection:** changes the protected tool inventory and
  cannot establish declared tool or input identity.
- **Live cross-host Rust sessions:** adds provisional result, provider and
  process-state contracts that are unnecessary for finished unit artifacts.
- **Automatically infer C/C++ graphs from arbitrary command logs:** misses
  generated-file ordering and negative header searches. Explicit inputs and
  edges are the admitted frontend contract instead.

## Consequences

Small unit graphs can use multiple admitted workers without compiler-specific
scheduler branches. Opaque Cargo/Make derivations remain whole actions unless
the operator selects a unit-plan frontend. Limits on goals, plans, source
slices, transfer and resources remain enforced; the full Mantle workspace is
not silently admitted by raising them. Worker selection and remote protocol
success cannot substitute for full output-signature and content verification.

Qualification must run real compilers, observe overlapping actions on distinct
workers, execute the final programs, perturb headers/sources/tools, reject
stale or tampered results, and repeat from independent stores on at least two
worker hosts. Local loopback remains useful integration evidence, explicitly
not independent-host acceptance. Timing and transfer measurements describe only
the recorded workloads; this decision makes no general speedup, compiler
correctness, reproducibility, bootstrap, security or release-readiness claim.
