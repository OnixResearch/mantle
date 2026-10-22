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

- Pure core: bounded record fact validation, BLAKE3 identity over canonical
  bytes, deterministic merge, owner-scoped release planning, and GC plan-input
  selection over in-memory values.
- Shell: versioned record validation, canonical serde JSON encoding, directory
  reads and atomic single-record writes, explicit legacy-file migration, and
  command rendering.
- Policy: unknown record versions and invalid identities fail closed.

## Decisions

### Decision: One record per (owner, path, reason) interest

**Choice:** Persist a canonical record per interest, named by the BLAKE3
identity of its canonical bytes.

**Rationale:** Add becomes a single create. Release removes one file. Two
writers cannot overwrite each other, and the filename is the integrity check.
This follows the reviewed Synit user-settings shape.

Automatic re-registration of a non-explicit-pin root replaces the same
owner's prior declaration for that path even when renewal or generation
advancement changes its transition reason. Operator pins under distinct
explicit reasons remain independent.

Renewal and generation decisions consult the owner's own persisted declaration
before the single-path merged reporting view; another owner cannot reset its
renewal count or change its transition classification.

### Decision: Owner-scoped release only

**Choice:** A release command removes the one validated interest matching its
requested owner, path, and (when ambiguous) exact reason. It does not delete a
different owner's record.

**Rationale:** Independently declared interests must survive other owners'
releases. Operator `--owner` is a local label, not authenticated identity;
filesystem write access grants control of unsigned records. This is not a
multi-tenant authorization boundary.

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
