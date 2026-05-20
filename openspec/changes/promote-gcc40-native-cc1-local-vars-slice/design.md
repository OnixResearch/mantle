## Context

The GCC 4.0 pass1 bridge still delegates general `cc1` inputs to TinyCC, but selected bounded proof inputs have native no-delegation marker paths. The previous slices covered arithmetic/control-flow and logical/control-flow. A local-variable assignment/update input is a useful next semantic shape because it distinguishes simple statement sequencing and local state from the prior branch-only marker cases.

## Goals / Non-Goals

**Goals:**
- Add exactly one bounded local-variable assignment/update `cc1` proof input.
- Preserve previous arithmetic/control-flow and logical/control-flow regression evidence.
- Fail closed on stale receipts, missing markers, transcript digest drift, TinyCC delegation, missing regressions, or parity overclaiming.

**Non-Goals:**
- Full `cc1` native compilation.
- General C parser/codegen correctness.
- Removing the pass1 bridge fallback.
- Completing live-bootstrap, Guix, or StageX parity.

## Decisions

### 1. Continue versioned native-cc1 receipt

**Choice:** Advance the existing native `cc1` receipt file to a new schema, tentatively `mantle-gcc40-native-cc1-arithmetic-v3`, with selected slice `local-variable-assignment-v3`.

**Rationale:** The existing receipt path already records the no-delegation native `cc1` evidence. Keeping one receipt with preserved regressions makes stale/missing prior evidence fail closed.

### 2. Use deterministic marker output for the bounded slice

**Choice:** The selected proof input emits a deterministic marker object-shaped file and records transcript/output BLAKE3 digests, as the prior slices do.

**Rationale:** This verifies the bounded no-delegation seam without implying general native object correctness.

## Risks / Trade-offs

- **Overclaim risk:** Mitigated by partial-only parity effect and tests asserting `gcc.4.0` remains blocking.
- **Regression loss:** Mitigated by requiring v1 arithmetic and v2 logical regressions in the v3 receipt.
- **Archive drift:** Inspect and repair canonical bootstrap spec after archive because this requirement is cumulative and historically prone to OpenSpec replacement drift.
