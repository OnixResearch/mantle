# Design: Check GC planning against a Terrane kernel

## Context

The kernel works over indices. Mantle works over path identifiers. The
projection between them is the unverified part of the rail, so it stays small
and is exercised with unsorted and duplicated input.

## Decisions

### Decision: Follow the ADR 0060 import pattern

**Choice:** Copy exact Git objects from a pinned Terrane revision, check
BLAKE3 identities offline, and compile the kernel only under `cfg(test)`.

**Rationale:** The production dependency graph does not change, and the kernel
cannot drift silently. `crunch-gc-core` is `no_std`, so the kernel imports
`Vec` and `vec!` explicitly instead of relying on the standard prelude.

### Decision: Projection rules

**Choice:** Entries are sorted by path identifier and numbered in that order.
References and roots are deduplicated. Overlay ownership is reclaimable; any
base layer is protected. A path that names no entry maps to the index one past
the last entry, which the kernel rejects as out of range.

**Rationale:** Mantle's missing-root and missing-reference errors then map to
the kernel's out-of-range classes, and its base-to-overlay error maps to the
kernel's protected-closure class.

### Decision: Compare both directions

**Choice:** The rail compares Mantle's plan with the kernel's `plan`, and also
runs the kernel's `check_plan` on Mantle's output.

**Rationale:** Equality with `plan` catches any difference in the result.
Acceptance by `check_plan` ties each Mantle output directly to the proved
safety theorems, independent of how the kernel's own planner computes.

## Non-Claims

The projection is not verified. Parity is executable agreement on enumerated
cases, not a theorem about Mantle source. Size limits, duplicate path
identifiers, and layer indices other than 0 are outside the enumerated space.
When several violations coexist, Mantle reports them in its own order (for
example, a base-to-overlay reference before a dangling reference), and the
rail only requires that the reported one is among the kernel's violations.
