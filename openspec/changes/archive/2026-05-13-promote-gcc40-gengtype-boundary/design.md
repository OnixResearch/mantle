## Context

`bootstrap/gcc-4.0.ncl` seeds and bridges GCC 4.0's `gengtype` output forest so C frontend compilation can pass the generated-header boundary. The seam is intentional but still labeled with generic `stub` text.

## Goals / Non-Goals

**Goals:**
- Mark `gengtype` outputs as explicit empty-GTY header/descriptor boundaries.
- Check representative generated outputs inside the derivation.
- Keep parity status partial and evidence-backed.

**Non-Goals:**
- Build or trust native GCC 4.0 `gengtype`.
- Claim GC root traversal correctness or complete `gcc.4.0` parity.

## Decisions

### 1. Check representative generated outputs

**Choice:** Invoke the bridged `gengtype` executable in a scratch directory after the native make attempt and grep `gt-cgraph.h` plus `gtype-desc.c` for boundary markers and legacy-label absence.

**Rationale:** Checking every generated header would add noise; one representative header plus the descriptor source proves the bridge emits the intended boundary family.

**Alternative:** Only rename comments. Rejected because derivation-local checks catch drift in the actual bridged executable.

## Risks / Trade-offs

**Boundary hardening can look like correctness.** Mitigation: spec, comments, parity notes, and tests keep `gcc.4.0` partial and state that native `gengtype` correctness remains pending.
