## Context

The current GCC 4.0 pass1 bridge intentionally uses bounded wrappers for generator outputs while native GCC correctness remains incomplete. Recent work tightened `gencheck`, `genpreds`, `genattr`, and `gengtype` boundaries. `genemit` still emits a generic `genemit_bootstrap_stub` source body.

## Goals / Non-Goals

**Goals:**
- Replace the generic `genemit` source stub label with a named empty-emit boundary.
- Add checks that fail closed if the boundary output or legacy label drifts.
- Preserve the partial parity classification.

**Non-Goals:**
- Implement real native `genemit` semantics.
- Claim GCC 4.0 native compiler correctness or unblock live-bootstrap/Guix parity.

## Decisions

### 1. Empty-emit source boundary

**Choice:** Emit a source file with `gcc40_genemit_empty_emit_source_boundary` plus an explicit `empty-emit source boundary` marker.

**Rationale:** This is more precise than a generic `stub` label and gives parity tests a deterministic seam to validate.

**Alternative:** Attempt real native `genemit` correctness now. Rejected for this increment because it is larger and would conflate boundary labeling with compiler correctness.

## Risks / Trade-offs

**Boundary facades can be mistaken for correctness** → The spec, derivation checks, and final report state this remains partial and not native generator correctness.
