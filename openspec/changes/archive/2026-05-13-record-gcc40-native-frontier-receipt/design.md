## Context

`gcc.4.0` is already evidence-backed partial with placeholder inventory and native-boundary receipt. Boundary wrappers for late generated-source tools have been split. The remaining ROI is no longer another wrapper split; it is recording the actual frontier that prevents a native/full GCC 4.0 claim.

## Goals / Non-Goals

**Goals:**
- Add a checked native-frontier receipt section.
- Validate markers against `bootstrap/gcc-4.0.ncl` so the receipt drifts closed.
- Preserve the explicit partial/non-native claim.

**Non-Goals:**
- Do not claim native `cc1`, native generator correctness, or live-bootstrap/Guix parity.
- Do not remove bridge/stub implementation debt in this change.

## Decisions

### 1. Extend the existing native-boundary receipt

**Choice:** Add a `native_frontier` object to `gcc-4.0-native-boundary.json` rather than creating a separate file.

**Rationale:** The frontier is part of the same evidence contract: native make attempt, pass1 bridge, and remaining native-correctness blockers.

**Alternative:** A separate frontier JSON was rejected because it would duplicate derivation and parity-effect metadata.

## Risks / Trade-offs

**Overclaiming risk** → The schema keeps `status=boundary-only` and exact `parity_effect` unchanged.

**Receipt rot** → Validation requires non-empty frontier markers and checks each marker exists in the derivation.
