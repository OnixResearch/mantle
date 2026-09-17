# Design: Retention interests replace the shared GC roots file

## Goal and scope

Give every retention of a store path an owner, a reason, and its own record.
Keep the GC decision core unchanged. Keep the legacy roots file readable until
an operator migrates it.

## Current behavior

`state_dir/gc-roots.json` holds the retained logical store paths. Root
registration rewrites that document. Release rewrites it again. Nothing in the
document says who retained a path or why. The migration path marks old roots
`legacy-unmanaged`, which records provenance but not ownership.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Keep the JSON document | Rewrite shared state under a lock | Rejected as sufficient: no owner facts, and merge conflicts stay possible | Concurrent-writer fixture |
| One record file per interest | Digest-named canonical records, deterministic merge | Selected direction | Merge and migration fixtures |
| Owner list inside the JSON | Nested owners per path | Rejected: overwrite risk stays inside one document | Lost-update fixture |
| Append-only log | Ordered log with compaction | Deferred: more shell complexity than the record set needs now | Not blocking |

## Contract and component ownership

- Pure core: record validation, canonical bytes, BLAKE3 record identity,
  deterministic merge, owner-scoped release planning, and plan-input
  construction over in-memory values.
- Shell: directory reads and writes, atomic publication of one record,
  legacy-file read for migration, and command rendering.
- Policy: the record schema is versioned. Unknown versions fail closed.

## Decisions

### Decision: One record per (owner, path, reason) interest

**Choice:** Persist a canonical record per interest, named by the BLAKE3
identity of its canonical bytes.

**Rationale:** Add becomes a single create. Release removes one file. Two
writers cannot overwrite each other, and the filename is the integrity check.
This follows the reviewed Synit user-settings shape.

### Decision: Owner-scoped release only

**Choice:** A release command removes records that name the caller's owner. It
does not remove records of another owner.

**Rationale:** An operator action must not silently drop a CI or proof
retention. Cross-owner cleanup becomes an explicit administrative action.

### Decision: GC consumes a merged view, not the record set

**Choice:** The core merges records into the retained root set before planning.

**Rationale:** The GC planner keeps its current inputs and rules. The record
layer changes only how the input set is derived and reported.

## Risks / Trade-offs

- Many small files can accumulate. The merge reports record counts, and a
  compaction path can follow once usage is measured.
- Digest-named files need bounded reads. Record size and count limits fail
  closed before merge.
- `legacy-unmanaged` roots keep their current visibility until migration.

## Non-Claims

- A record proves a declared interest. It does not prove reachability or
  correctness of the retained path.
- The record set is not release evidence.
