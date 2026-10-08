# ADR 0100: Store retention interests as per-owner records

- Status: Proposed
- Date: 2026-10-01

## Context

The writable store's `gc-roots.json` is a single path-keyed document. Although the operator CLI serializes mutations under the store mutation guard, a path has only one provenance slot and independent callers cannot express or release separate interests in the same path. Legacy path-only roots have no recoverable owner identity.

## Decision Drivers

- Two independent owners retaining one path must both remain visible.
- Removing an interest must not silently remove another owner's declaration.
- The existing bounded retention and GC planners must retain their path-level rules and candidate ordering.
- Corrupt, duplicate, oversized, and unknown-version input must fail closed before GC plans a deletion.
- Existing roots must stay readable until explicit migration without invented ownership.

## Decision

Store each canonical version-1 interest under `<state-dir>/retention-interests/<blake3-hex>.json`. The canonical JSON bytes contain owner label, logical path, reason, and the existing versioned root declaration facts; the BLAKE3 filename checks those bytes on every read. Maximum record size is 4,096 bytes and count is 65,536. A dedicated admission lock serializes the bounded count check and single-record publication: sync a fresh pending file, hard-link it to its content-addressed final name, sync the directory, then remove the pending name. Release removes only the validated identity matching the requested owner, path, and (when ambiguous) exact reason; it never rewrites another interest. Identical publication is idempotent. Operator re-declaration replaces one same-owner/path/reason record. Automatic re-registration of non-explicit-pin roots replaces earlier declarations for its owner/path even when the transition reason changes; separate operator pin reasons remain independent.

Lease renewal and project generation resolution read the owner's own declaration, not the one-path merged representative that might belong to another owner. For interrupted transitions with multiple reason revisions, the most recent persisted owner/path declaration supplies renewal and generation facts.

The store reads the legacy registry without mutating it during unrelated operations. `store roots --migrate` explicitly publishes legacy declarations with the existing `legacy-unmanaged` provenance, then archives the old registry as `gc-roots.migrated.json`. Until migration, the old entries remain visible and protective. The deterministic merge presents one root per logical path to the existing retention and GC plan inputs, while reporting all owner and reason facts plus each path's physical record count. GC classification, candidate ordering, and mutation rules are not rewritten by the ledger.

An explicit operator `--owner alice` maps to the bounded local scope `operator:alice`; the default `--owner operator` retains the legacy `operator` scope. These labels are not cryptographic credentials. The previous unsigned legacy registry cannot establish authority over a named owner; migration must not imply it does. A caller with filesystem write access to the state directory can manipulate unsigned record files. External multi-tenant authority requires an independent authenticated capability boundary and is **not** provided by this decision.

## Alternatives

- Continue rewriting one JSON map under a global lock: rejected because one slot per path cannot represent independent interests.
- Store owner arrays in that map: rejected because release still rewrites shared state.
- Append-only log and compaction: deferred; adds durability and coordination complexity without a present need.

## Consequences

- Two writers publish separate identity-named files; single-record failure cannot replace another owner's file.
- Invalid, tampered, duplicate, or over-limit records block root listing and GC rather than being ignored.
- Record replacement is not an atomic multi-file transaction. A reader overlapping publication and deletion can briefly see both old and new transition reasons; duplicate records with the same owner/path/reason fail closed. No whole-store database atomicity is claimed.
- Legacy roots and retained paths are declarations, not proof of content reachability, correctness, trust, or release eligibility.
