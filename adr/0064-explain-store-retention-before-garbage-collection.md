# ADR 0064: Explain store retention before garbage collection

## Status

Accepted

## Context

Mantle's legacy GC-root registry records a logical path, a broad source label, and creation time. It does not record the owner, policy, project generation, lease, or transition that keeps a path live. The legacy GC command also plans and deletes in one operation.

This design makes routine cleanup hard to review. It also makes policy changes and stale observations hard to detect before deletion.

## Decision Drivers

- Make each retention decision explainable from bounded recorded facts.
- Keep filesystem, clock, and store service access outside the pure planning core.
- Preserve legacy roots until an explicit migration records their unknown ownership.
- Require an exact accepted plan identity before deletion.
- Keep project identity, selector, generation, lease, policy, and transition facts visible.
- Continue independent deletion attempts after one deletion fails.

## Decision

Mantle will use a versioned root record with typed root classes. The classes cover explicit pins, project output generations, source generations, active shell leases, bootstrap roots, self-build roots, remote results, and protected legacy roots.

A typed Nickel policy will set bounded generation, lease, root, decision, usage, and explanation limits. The generated JSON bytes have a BLAKE3 policy identity. Each root transition also has a BLAKE3 identity.

`crunch-gc-core` will remain a `no_std + alloc` functional core. It will accept root, time, closure, and size facts. It will return deterministic retention decisions, usage totals, and a plan identity without I/O.

The `crunch-store` shell will load root and store observations, call the core, and render explanations. A normal GC command will only produce a plan. Execution will require `--execute --plan-id <id>`. The shell will replan before deletion and reject a stale identity.

Project builds will register selected output and source generations. Shell builds will register time-bounded leases with bounded renewals. Remote substitutions without managed project provenance will register as remote-result roots.

Legacy records will migrate to the protected `legacy-unmanaged` class. Migration does not infer an owner or authorize deletion.

## Alternatives Considered

### Keep source labels and add more CLI text

Rejected because prose cannot recover missing owner, policy, generation, lease, or transition facts.

### Delete immediately after displaying a plan

Rejected because display does not prove that the operator accepted the exact facts used for deletion.

### Put store access in the planning crate

Rejected because ambient I/O would make retention decisions harder to test and review.

### Delete only after every candidate is known to succeed

Rejected because filesystem deletion cannot be made atomic across independent store objects. Mantle commits authoritative metadata first. It then continues across independent safe cleanup attempts and reports candidate paths, completed operation classes, and observed failures.

## Consequences

- GC is a two-step operation.
- Existing root files need explicit migration.
- Unknown size or closure facts remain visible and do not become zero-byte claims.
- A plan identity proves agreement with the recorded planning inputs. It does not prove deletion, release eligibility, or whole-store correctness.
