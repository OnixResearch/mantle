## Overview

The topology rail extends the existing one-edge dependency-chain executor into a target-only closure executor. It remains bounded: every executed unit must already be represented as a ready `unit_derivation_graph` target unit, and Mantle still invokes `rustc` directly from explicit derivation args/env.

## Execution Model

1. Require `unit_derivation_graph.ready=true`.
2. Select supported target units (`execution_kind=target`, `target_kind=lib|bin`) with no consumed host artifacts.
3. For each selected unit, require every dependency artifact to have a matching supported producer `lib` unit in the graph.
4. Topologically order units by dependency package edges.
5. Execute each producer before consumers using the existing per-unit executor.
6. Bind each produced `.rlib` artifact into downstream dependency placeholders, derivation inputs, and `--extern` args before invoking the downstream `rustc`.
7. Emit a topology receipt preserving ordered per-unit execution receipts and a self-reference-safe receipt hash.

## Blockers

The rail blocks deterministically before invoking an affected consumer `rustc` when:

- the graph is not ready,
- supported target units require host/proc-macro/build-script artifacts,
- a dependency producer is absent or not a supported `lib`,
- dependency edges cycle,
- a producer execution fails,
- produced `.rlib` material is absent/unreadable,
- dependency rebinding cannot update the explicit derivation surfaces.

## Receipt Shape

CLI JSON wraps the captured `rust_plan` receipt with `target_topology_execution`. The topology receipt includes schema version, execution status, bounded claim, ordered unit execution receipts, blocker, and receipt hash.
