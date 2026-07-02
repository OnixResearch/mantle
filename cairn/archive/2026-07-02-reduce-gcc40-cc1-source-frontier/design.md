## Context

The current `bootstrap/evidence/gcc-4.0-native-boundary.json` records `native_frontier.status = frontier-only`. It lists promoted installed-`cc1` semantic slices through pointer dereference, while `source_frontier_markers` still document the TinyCC/Mes source-boundary diagnostic and pass1 bridge fallback.

This change treats the source-build frontier separately from installed-`cc1` semantic slices. Success can mean either a proven movement to a new narrower frontier or a better bounded diagnostic for the same blocker. It cannot mean native GCC 4.0 correctness.

## Decisions

### 1. Work one source frontier only

**Choice:** Choose a single current `cc1` source-build frontier marker, such as the focused `c-parse.o` include/diagnostic boundary, and probe only that seam.

**Rationale:** GCC 4.0 has many bridge markers. A one-frontier probe keeps the evidence reviewable and avoids accidental broad source rewrites.

### 2. Receipt-first evidence

**Choice:** The implementation must update or create checked JSON evidence before parity can change. The receipt names schema, selected frontier, prior marker, attempted command or patch scope, observed result, transcript digest, exact source markers, partial-only parity effect, and retirement condition.

**Rationale:** The bootstrap parity report should consume deterministic evidence, not prose memory or transient logs.

### 3. Fail closed on overclaim

**Choice:** If the probe succeeds for the bounded frontier, `gcc.4.0` may report a narrower evidence-backed partial status. If evidence is stale, missing, delegated to TinyCC, or broader than the selected frontier, the row remains a blocker.

**Rationale:** The bridge reduction must reduce ambiguity without claiming source-built GCC.

## Risks / Trade-offs

- The selected source frontier may not move. That is still useful if the new receipt names the exact blocker and validates that the pass1 bridge remains bounded.
- Diagnostic derivations can become large. Keep transcript digests and compact summaries in tracked evidence; leave generated logs under ignored targets.
