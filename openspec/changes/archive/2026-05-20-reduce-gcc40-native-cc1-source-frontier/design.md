## Context

`bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` now records the current native `cc1` source-build frontier separately from bounded installed-`cc1` semantic slices. The checked frontier keeps `gcc.4.0` evidence-backed `partial`, and blocker inventory now treats its exact covered source lines as metadata.

The remaining high-ROI work is not broader metadata classification; it is a small source-frontier reduction attempt that changes what the next maintainer can inspect about the real native `cc1` build.

## Goals / Non-Goals

**Goals:**
- Select one bounded native `cc1` source-build seam near the current TinyCC/Mes `c-parse`/`gengtype-yacc.c` boundary.
- Add or update receipt evidence that records the exact probe, source markers, frontier result, and retirement condition.
- Require fail-closed validation for stale markers, unsupported schema/status, missing frontier result, or parity overclaim.
- Keep `gcc.4.0` partial unless a real full native compiler proof exists.

**Non-Goals:**
- Full GCC 4.0 native compiler correctness.
- Broad refactoring of the GCC 4.0 derivation.
- More installed-frontend semantic smoke slices unless needed as a regression for the source-frontier probe.

## Decisions

### 1. Treat this as source-frontier evidence, not semantic promotion

**Choice:** The implementation should update a dedicated source-frontier receipt or add a successor receipt with a name/schema that clearly identifies the native source-frontier reduction attempt.

**Rationale:** The current blocker is source-build reach, not another bounded installed-frontend behavior. Keeping these evidence domains separate prevents overclaiming.

### 2. Require exact marker-backed frontier results

**Choice:** The receipt must name the old frontier, the new observed frontier or unchanged blocker, exact source/derivation markers, and an explicit `parity_effect` that remains partial-only.

**Rationale:** Exact markers make stale evidence fail closed and give future drains a concrete next boundary.

### 3. Keep the implementation slice small

**Choice:** Probe one source seam only. Candidate seams include narrowing the `gengtype-yacc.c` make target failure, isolating the TinyCC/Mes `c-parse` segfault boundary, or proving that one additional generated source/header prerequisite can be produced natively before the pass1 bridge.

**Rationale:** A broad native GCC build fix is too large for one drain. One frontier reduction is inspectable and verifiable.

## Risks / Trade-offs

- **No frontier movement:** Still acceptable if the receipt records a more precise stable blocker and fail-closed validation.
- **Overclaim risk:** Mitigated by partial-only status, no live-bootstrap/Guix/StageX unblock, and negative tests.
- **Line drift:** Mitigated by marker validation and refreshing blocker-inventory checks only for exact covered metadata lines.
