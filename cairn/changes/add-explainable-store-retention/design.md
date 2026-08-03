# Design: Explainable store retention

## Context

`crates/crunch-store/src/roots.rs` persists one record per logical path with `GcRootSource` and `created_unix_s`. The build and bootstrap paths can register output roots automatically. `crates/crunch-store/src/gc.rs` computes live closure state and reports candidates plus reclaimable bytes. Project input retention separately records current or recent source generations.

These primitives protect data, but they do not provide a complete lifetime model. A permanent build root and a temporary development shell look similar to GC. An operator can see what GC may delete, but not why another closure remains or which owner and policy retain it.

## Decisions

### Decision: Make retention policy typed and explicit

**Choice:** Add a typed Nickel policy with named limits and rules for root classes. The initial classes are explicit pin, project output generation, project source generation, active shell lease, bootstrap, self-build, remote result, and legacy unmanaged.

Each class declares durability, maximum retained generations when applicable, lease behavior when applicable, eligible owner scopes, and removal requirements. Runtime Rust consumes checked-in generated policy data.

**Rationale:** Retention choices are operator policy. Named policy values prevent hidden time windows and generation counts.

### Decision: Plan retention in a pure core

**Choice:** A pure core receives normalized root records, project generations, lease observations, PathInfo closure summaries, storage byte observations, supplied current time, and policy. It returns ordered keep, expire, migrate, quarantine, and remove decisions with stable reason codes.

The shell owns clocks, file reads, database snapshots, closure traversal, locks, writes, and deletions. The core does not inspect timestamps or paths outside supplied observations.

**Rationale:** Operators must be able to replay why one root remains and another expires.

### Decision: Version root provenance

**Choice:** A new root record binds logical path, root class, owner scope, project identity when present, selector when present, lock or generation identity when present, lease identity and expiry when present, policy BLAKE3, creation observation, and last accepted transition.

Host checkout paths remain locator metadata. Project identity derives from canonical project and lock facts, not from an absolute checkout path. Unknown legacy records migrate to `legacy-unmanaged` and remain protected until explicit classification or removal.

**Rationale:** Invented ownership would make migration unsafe. Path-only ownership would make equivalent checkouts unrelated.

### Decision: Separate usage observation from retention policy

**Choice:** `mantle store usage` reports observed bytes and object counts by retained, reclaimable, quarantined, and unclassified state. It also groups retained bytes by root class and owner scope when facts exist.

Shared closure bytes are counted once in totals. Per-root inclusive bytes and unique bytes remain separate fields. Unknown or unreadable bytes remain explicit instead of becoming zero.

**Rationale:** Disk observations do not decide policy, but operators need both total pressure and the roots that contribute to it.

### Decision: Explain every keep and remove decision

**Choice:** Explained root listing shows the direct root reason, owner, policy, generation or lease state, and bounded closure summary. Explained GC planning shows each candidate reason and every retaining root for paths that remain live.

Human output can summarize. JSON retains stable reason codes and bounded parent links. Full unbounded closure paths require a separate explicit query.

**Rationale:** A GC report that lists only deletion candidates cannot answer why disk space remains occupied.

### Decision: Make GC execution plan-bound

**Choice:** Ordinary `mantle store gc` becomes a non-mutating plan. Mutation requires explicit execution and the accepted plan identity. Before deletion, the shell re-observes root registry, policy, PathInfo, and relevant storage facts. Drift rejects the plan before mutation.

The mutation shell holds the existing store mutation lock, stages rewritten registries and databases, deletes only plan-authorized unreferenced files, and records partial cleanup failures without claiming complete success.

**Rationale:** A reviewed plan must not authorize deletion after root or closure state changes.

### Decision: Treat shell leases as revocable retention, not build identity

**Choice:** A shell lease has an explicit owner, lease identity, expiry observation, and renewal policy. Lease timing never enters package action identity. Expiry only changes retention eligibility.

A crashed shell cannot renew. GC can expire its lease only after policy and supplied clock facts permit it. Clock rollback, malformed expiry, and unknown lease state fail closed.

**Rationale:** Development convenience must not alter build hashes, but abandoned shells must not retain closures forever.

## Validation

Positive fixtures cover explicit pins, project output generations, source generations, active and renewed leases, bootstrap roots, usage totals, shared closures, explained keep decisions, dry-run plans, and unchanged-plan execution.

Negative fixtures cover corrupt roots, unknown policy, duplicate owners, generation overflow, expired leases, clock rollback, stale plans, changed PathInfo, missing closure facts, symlinks, interrupted registry replacement, deletion failure, and output that is retained by another root.

## Risks / Trade-offs

- Legacy roots remain conservative and can delay reclaimed space until operators classify them.
- Unique-byte accounting requires bounded closure and castore traversal.
- Plan-bound GC adds one explicit operator step.
- Lease policy depends on trustworthy supplied clock observations but does not make the clock part of build identity.
