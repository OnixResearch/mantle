# Proposal: Retention interests replace the shared GC roots file

## Why

`state_dir/gc-roots.json` keeps the retained store paths of the writable store
in one mutable document. The document names no owner for a path, so two writers
can lose an update, and releasing one root means editing shared state.

The Synit manual stores each user setting as a file named by the canonical
digest of its content and watches the directory
(`~/.local/share/mantle-references/synit-book/pages/19-operation__synit-config.md`,
reviewed in `docs/synit-application-notes.md`). The same shape fits retention:
one record per interest, add is a new record, release is the removal of one
record, and unrelated owners stay intact.

Retention is already a declared-interest model in operator practice. `store
roots`, `store usage`, and `store gc` describe retained, reclaimable, and
`legacy-unmanaged` state (ADR 0064). The record layer makes ownership explicit
without changing the GC decision rules.

ADR 0080 keeps retention records, merge, and GC in the building plane. The
coordination daemon may add interests for its own consumers.

## What Changes

- Define a versioned retention-interest record with owner identity, logical
  store path, reason, and declaration facts.
  r[mantle.store_lifecycle.retention_interest_records]
- Persist one canonical record per interest under the state directory, named by
  the BLAKE3 identity of its canonical bytes, and merge records
  deterministically into the retained root set.
  r[mantle.store_lifecycle.retention_interest_records]
- Release retention by removing exactly the caller's record. An owner MUST NOT
  modify or remove another owner's record.
  r[mantle.store_lifecycle.retention_owner_scope]
- Report per-path owner and reason facts in `store roots` and `store usage`.
  Keep `legacy-unmanaged` entries visible and keep migration explicit.
  r[mantle.store_lifecycle.retention_owner_scope]
- Keep GC planning identical in rules. The merged retained set is the plan
  input, and GC still removes only unretained paths.

## Impact

- **Immediate consumer**: local build root registration and the operator
  `store roots` / `store usage` / `store gc` workflow.
- **Immediate outcome**: concurrent writers stop losing retention updates, and
  every retained path has an owner and a reason.
- **Durable capability**: an auditable retention ledger that later carries CI,
  remote-layer, and proof retention.
- **Maintenance owner**: Mantle store lifecycle owner.
- **Repeatability evidence**: concurrent-owner fixtures, retraction fixtures,
  malformed-record rejection, and GC plan equivalence between the legacy file
  and the merged record set.
- **Compatibility**: `store roots --migrate` converts existing JSON roots into
  owner records. Single-owner stores keep working without migration.

## Scope

The change covers the record schema, persistence, merge, release, reporting,
migration, and the GC input boundary.

## Out of Scope

- Changing GC decision rules, ordering, or candidate classification.
- Deleting or rewriting the legacy roots file during unrelated commands.
- Retention for a store path that does not exist in the writable layer.
- Any claim that a retained path is reachable, correct, or trusted.

## Success Criteria

- Two owners add retention concurrently and both records survive.
- Removing one owner's record keeps every other record and path.
- A malformed, duplicate, or foreign-owned record fails closed or is rejected.
- A GC plan computed from merged records equals the plan computed from the
  equivalent legacy root set.
