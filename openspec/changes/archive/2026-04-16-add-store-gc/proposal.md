# Add store garbage collection

## Why

crunch still has no garbage collection. `PathInfo`, exported outputs, and
castore content accumulate until the operator deletes state by hand. That makes
long-lived local use risky and makes performance or self-hosting experiments
consume disk until the machine intervenes.

The repo already documents managed GC as a desired default. The immediate gap is
smaller and more urgent: crunch needs a safe manual reachability-based collector
with durable roots and clear operator reporting.

## What Changes

- add a durable GC root registry for retained outputs
- auto-root successful top-level outputs from `crunch build`, `crunch
  self-build`, and `crunch bootstrap --fetch`
- add manual mark-and-sweep collection over `PathInfo`, exported outputs, and
  castore content
- make GC fail closed when retained-root reachability metadata is missing,
  unreadable, or incomplete
- add operator commands for root inspection, pinning, unpinning, and dry-run GC
  reporting

## Capabilities

### New Capabilities
- `manual-store-gc`: reclaim unreachable local store state without deleting live
  roots
- `gc-root-registry`: retain top-level outputs and explicit pins across restarts
- `gc-sweep-report`: preview and audit what a collection run would remove

## Impact

- **Files**: `crates/crunch-store/**`, `src/store_cmd.rs`, build finalization
  paths that auto-root top-level outputs, docs for store operations
- **APIs**: new store-facing GC and root-management entry points
- **Dependencies**: none required
- **Testing**: reachability retention, unreachable deletion, dry-run no-mutate,
  auto-rooting of top-level results, and castore cleanup safety

## Non-Goals

- background or low-space-triggered automatic GC in this change
- generational GC policies or age-based heuristics
- remote cache pruning
- deleting content that is still reachable from any retained root
