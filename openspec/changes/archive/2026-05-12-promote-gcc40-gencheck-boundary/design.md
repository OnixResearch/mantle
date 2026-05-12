## Context

`gencheck` produces `tree-check.h` in a normal GCC build. Crunch currently bridges this path with generic stub-labeled outputs so the TinyCC/Mes handoff can pass the broader graph. This change makes that boundary explicit as disabled tree-checking output and checks it during the derivation.

## Goals / Non-Goals

**Goals:**
- Remove the generic `gencheck` stub labels from both the compiler-output object shim and generated header script.
- Add deterministic checks for the generated `tree-check.h` boundary.
- Keep `gcc.4.0` evidence-backed partial.

**Non-Goals:**
- Claim native `gencheck` execution correctness.
- Implement the full GCC tree-checking macro matrix.
- Promote native `cc1` correctness.

## Decisions

### 1. Disabled tree-checking boundary

**Choice:** Emit a guarded `tree-check.h` with a named disabled-tree-checking boundary marker.

**Rationale:** GCC can build with checking disabled, so this is a more precise boundary than generic stub text while preserving the current bootstrap shape.

**Alternative:** Run real `gencheck` now. Rejected for this increment because the native generator graph remains mediated by TinyCC/Mes boundary shims.

## Risks / Trade-offs

**Risk:** Boundary wording might be mistaken for native generator completion.  
**Mitigation:** Spec, parity evidence, and final report explicitly keep `gcc.4.0` partial and blocking parity.
