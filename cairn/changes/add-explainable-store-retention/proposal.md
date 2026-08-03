# Add explainable store retention

## Why

Mantle already records GC roots, supports explicit pin and unpin operations, and reports dry-run candidates with reclaimable bytes. Project inputs also support bounded retention generations.

The store-level root record still contains only a logical path, a broad source class, and a creation time. Build roots can therefore accumulate without a project, selector, generation, lease, or policy reason that explains their lifetime. GC reports list deletion candidates but do not show why retained closures remain live or which policy change would make space reclaimable.

Mantle needs a bounded retention policy and an operator-visible explanation before storage growth becomes another hidden-state problem.

## What Changes

- Add a typed Nickel store-retention policy for explicit pins, project outputs, source generations, development-shell leases, bootstrap roots, self-build roots, and unmanaged compatibility roots.
- Add a pure retention planner over root, project, lease, closure, and current-time observations supplied by the shell.
- Version root records with owner scope, selector, generation, lease, policy identity, and stable reason codes.
- Add deterministic `mantle store usage`, explained root listing, and explained GC planning.
- Make ordinary GC planning non-mutating and require explicit execution against an unchanged plan identity.
- Migrate current root records without inventing project ownership or lease facts.
- Add positive, negative, interruption, stale-plan, overflow, and corruption fixtures.

## Non-Goals

- Garbage-collecting a read-only base store or implementing overlay composition in this change.
- Selecting organization-wide storage quotas or billing policy.
- Treating filesystem access time as retention authority.
- Proving that retained outputs remain semantically correct or rebuildable.

## Impact

- **Affected specs:** new `store-lifecycle` capability
- **Planned files:** typed Nickel policy, generated policy data, pure retention and GC planning core, versioned root registry, store CLI, reports, migration, docs, and fixtures
- **Compatibility:** legacy roots remain protected until classified or explicitly migrated; current mutating GC behavior gains a reviewed explicit-execution transition
- **Testing:** root migration, project generations, leases, usage accounting, explain output, stale plans, interrupted mutation, corruption, and Cairn gates
