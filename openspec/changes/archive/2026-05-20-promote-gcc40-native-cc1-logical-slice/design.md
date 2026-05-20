## Context

The installed GCC 4.0 `cc1` remains a pass1 bridge for general inputs, but it contains a checked no-TinyCC-delegation path for one bounded arithmetic/control-flow proof input. That receipt is intentionally partial and does not unblock parity.

## Goals / Non-Goals

**Goals:**
- Add exactly one new bounded logical/control-flow `cc1` proof input.
- Preserve the v1 arithmetic/control-flow regression evidence.
- Fail closed on stale v1-only receipts, missing markers, transcript digest drift, TinyCC delegation, or parity overclaiming.

**Non-Goals:**
- Full `cc1` native compilation.
- General C parsing/codegen correctness.
- Removing the pass1 bridge fallback.
- Completing live-bootstrap, Guix, or StageX parity.

## Decisions

### 1. Receipt version bump

**Choice:** Advance `gcc-4.0-native-cc1-arithmetic.json` to schema `mantle-gcc40-native-cc1-arithmetic-v2` and selected slice `logical-boolean-control-flow-v2`.

**Rationale:** The existing receipt path is already wired as the native `cc1` evidence check. Versioning it avoids adding parallel plumbing while making stale v1 evidence fail closed.

### 2. Static marker object output

**Choice:** The selected input writes a deterministic marker object-shaped file, as the prior arithmetic slice does, and the receipt records the transcript and output digest.

**Rationale:** This proves the bounded no-delegation seam without weakening the claim into full native object correctness.

## Risks / Trade-offs

- **Overclaim risk:** Mitigated by partial-only parity effect and tests asserting `gcc.4.0` remains blocking.
- **Fixture drift:** Mitigated by digest recomputation and stale-schema tests.
- **Archive drift:** Inspect and repair canonical bootstrap spec after archive.
