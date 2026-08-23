# ADR 0083: Bind receipt source through an aggregate closure root

## Status

Accepted (2026-08-23)

## Context

The source-built fixed-point plan computes one source-authority BLAKE3 over eight required source roles. The v2 deterministic receipt uses this aggregate identity as `source_blake3`.

The rebuild descriptor originally listed only the eight source leaves. V47 completed both strict Cargo-free stages, but generic receipt classification rejected the descriptor. No leaf digest matched the aggregate receipt source identity.

## Decision Drivers

- Keep the receipt source bound to the complete source authority.
- Retain each named source role as reviewable rebuild input.
- Satisfy the generic v2 source-closure contract without adding product rules to the release core.
- Reject leaf-only descriptors before receipt publication.

## Decision

The content-bound rebuild descriptor contains one aggregate `source-authority-closure` root and all eight validated source leaves.

The root digest equals the plan-owned receipt source BLAKE3. Its bounded size is the checked sum of all leaf sizes. The approved-read identity set contains the same aggregate root.

The fixed-point plan remains the owner of leaf-role completeness. Receipt construction consumes that contract, adds the aggregate root, and validates the final source binding.

Positive tests require successful v2 classification with the root. Negative tests require rejection when only the leaves remain.

## Alternatives Considered

### Use only the Mantle source leaf as `source_blake3`

Rejected. This would narrow the receipt source identity and stop binding the complete StageX, native, Rust, Mantle, and vendor source authority.

### Teach the generic release core to recompute Mantle's aggregate

Rejected. The release core does not own Mantle's source-authority framing or role set.

### Replace the leaves with only the aggregate root

Rejected. This would hide the named source inputs from the rebuild descriptor.

## Consequences

- The receipt source identity binds the complete source-authority closure.
- Reviewers can still inspect every source role and digest.
- The descriptor has one more source identity than the plan's leaf count.
- A missing or changed aggregate root fails before receipt publication.
- This decision does not prove source correctness, compiler correctness, or release eligibility.
