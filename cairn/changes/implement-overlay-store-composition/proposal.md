# Implement overlay store composition

## Why

Mantle currently opens one writable local store per invocation. Remote substituters can backfill missing paths, but they cannot present a trusted read-only local base beneath a thin per-user writable store.

ADR 0012 selects overlay-first read-through, overlay-only writes, no read backfill, one logical prefix, per-layer trust, and overlay-only GC. The design is still proposed and has no Cairn implementation contract.

Shared hosts, CI workers, and unprivileged users need a bounded way to reuse one local base without copying its complete closure into every user store or granting write authority over shared state.

## What Changes

- Add ordered read-only base stores beneath one writable overlay under the same logical store prefix.
- Add no-backfill read-through combinators for PathInfo, directory, and blob services.
- Route every mutation, substitution result, root update, and attestation write to the overlay only.
- Bind layer identity, trust policy, precedence, and base-state generation into plans and reports.
- Preserve explicit shadow semantics while failing closed on invalid higher-precedence facts.
- Make overlay GC retain overlay content referenced by overlay roots and preserve base references without mutating base state.
- Present one merged castore view to sandbox and store inspection paths.
- Add positive, negative, fault, race, and multi-layer fixtures.

## Dependencies

- `add-explainable-store-retention` owns versioned root reasons and plan-bound GC semantics used by overlay collection.
- ADR 0012 remains the architectural basis. Any implementation deviation requires an ADR update before code changes.

## Non-Goals

- Combining different logical store prefixes.
- Supporting multiple writable layers or distributed multi-writer coherence.
- Copying base content into the overlay on read.
- Garbage-collecting, repairing, signing, or rewriting a declared read-only base.
- Treating base presence as trust without the configured layer policy.

## Impact

- **Affected specs:** `store-lifecycle`
- **Planned files:** `crunch-store`, vendored generic service combinators, build orchestration, attestation and GC adapters, global CLI, typed Nickel policy, reports, fixtures, and docs
- **Compatibility:** single-store mode remains the default; overlay mode requires explicit ordered base declarations with the same logical prefix
- **Testing:** service composition, no-backfill, write isolation, trust shadowing, base generation drift, cross-layer GC, sandbox reads, and Cairn gates
