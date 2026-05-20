## Context

The current native-boundary receipt proves that the GCC 4.0 build reaches a native `make -j1 -C "$dir"` attempt and then intentionally installs a pass1 bridge after a TinyCC/Mes source boundary. Recent drains improved bounded installed-`cc1` semantic slices, but the remaining high-ROI question is where the real native source build is still blocked and how to keep that blocker machine-checkable.

## Goals / Non-Goals

**Goals:**
- Add one small checked receipt for the current real native `cc1` build frontier.
- Require exact source/build markers in `bootstrap/gcc-4.0.ncl` so marker drift fails closed.
- Keep the parity effect partial-only and preserve all existing GCC 4.0 evidence contracts.
- State a retirement condition: replace this receipt when real native `cc1` source build evidence supersedes the pass1 bridge boundary.

**Non-Goals:**
- Full native `cc1` correctness.
- New installed-`cc1` marker-only semantic slice.
- Broad live-bootstrap/Guix/StageX unblocking.

## Decisions

### 1. Use a dedicated frontier receipt rather than extending the semantic-slice receipt

**Choice:** Create a separate receipt, expected path `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, and wire it as an additional evidence check for `gcc.4.0`.

**Rationale:** The native source-build frontier is not another bounded installed-frontend semantic slice. Separating it avoids overloading the arithmetic/logical/local-vars receipt and makes retirement explicit.

### 2. Validate exact derivation markers and partial-only effect

**Choice:** The validator must require exact markers for the native make attempt, source-boundary diagnostic, pass1 bridge fallback, and at least one source-frontier note currently present in `bootstrap/gcc-4.0.ncl`.

**Rationale:** Exact markers make the receipt useful as drift detection while preserving the non-claim that native GCC correctness is not proven.

## Risks / Trade-offs

- **Overclaim risk:** Mitigated by partial-only parity effect and explicit no-complete/no-unblock tests.
- **Stale receipt risk:** Mitigated by exact-marker validation and drift regressions.
- **Low product value if too descriptive:** Keep the receipt tied to actionable source-build markers and a retirement condition, not prose-only documentation.
