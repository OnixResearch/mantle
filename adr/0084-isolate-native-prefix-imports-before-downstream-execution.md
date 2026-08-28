# ADR 0084: Isolate native-prefix imports before downstream execution

## Status

Accepted (2026-08-28)

## Context

A promoted proof can reuse a validated native-provider prefix from a stopped attempt. The reuse path revalidated the imported provider, but it returned the origin path to Rust construction.

Rust bootstrap can retain shared inodes for selected musl inputs while it prepares a sysroot. V78 cleanup applied a recursive permission change to its failed working root. Six shared V61 files became owner-writable. The provider bytes stayed the same, but its tree identity changed because file modes are identity inputs. V79 then rejected the changed origin.

The source attempt is evidence. Downstream execution must not mutate it.

## Decision Drivers

- Keep checkpoint-origin bytes and modes unchanged.
- Preserve the independently authenticated provider identity.
- Keep source admission and no-follow path policy unchanged.
- Do not use hard links for mutable working inputs.
- Detect changes to the origin during materialization.
- Record the origin and working-copy validation paths.

## Decision

Mantle must treat a native-prefix import as origin evidence, not as a working tree.

Mantle first revalidates the origin provider and source closure. It then copies the provider into the current proof staging root with the bounded no-follow tree copier. The copy operation creates independent regular files and preserves the provider identity.

Mantle revalidates the copied provider before downstream execution. It binds Rust construction to the copied provider and its current revalidation report. It then rehashes the origin. The import fails if the origin changed during materialization.

The materialization report records both paths, both revalidation reports, the shared output identity, and the bounded copy semantics.

A stopped origin that already changed is not repaired in place. An operator can derive a separate candidate by ordinary byte copying and a narrowly reviewed normalization. That candidate is eligible only if full admission restores the previously authenticated identity.

## Alternatives Considered

### Execute directly from the imported provider

Rejected. Downstream mode changes can alter historical evidence and invalidate later imports.

### Copy with hard links

Rejected. Mode and content changes affect all links to the same inode.

### Accept the changed provider identity

Rejected. A new observed digest does not replace the independently authenticated expected digest.

### Rewrite the historical origin

Rejected. Historical evidence must remain distinguishable from a derived repair candidate.

## Consequences

- Native-prefix reuse copies about one provider tree before Rust construction.
- The copy adds bounded disk and hashing work.
- Downstream stages can change the working copy without changing origin evidence.
- A failed proof can remove its staging root without changing the imported attempt.
- This decision does not prove provider construction, compiler correctness, or fixed-point equality.
