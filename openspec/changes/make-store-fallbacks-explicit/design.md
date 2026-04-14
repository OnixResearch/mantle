# Design: Make store fallbacks explicit

## Context

Crunch's store layer owns the data that defines sandbox inputs and cache state.
If that layer degrades silently, the build result still exists, but the
operator's trust claim changes.

That means fallback policy belongs in the hermeticity model, not just in log
messages.

## Goals / Non-Goals

**Goals:**

- surface store-layer fallback behavior as typed audit events
- fail strict builds before sandbox start on weakened store semantics
- preserve a practical mode for local work and bootstrap edges

**Non-Goals:**

- redesign the store backend architecture
- change fetcher behavior
- remove all practical fallbacks entirely

## Decisions

### 1. Persistent PathInfo fallback becomes mode-dependent

**Choice:** failure to open persistent `PathInfo` becomes a reported degraded
mode in practical runs and a hard error in strict runs.

**Rationale:** persistent state is part of crunch's own trust boundary.

### 2. Missing closure facts become mode-dependent

**Choice:** strict mode fails if a source input lacks closure data needed for
sandbox assembly. Practical mode may still mount the declared path alone, but
must report that fact.

**Rationale:** a best-effort closure is weaker than a verified closure and needs
a different operator claim.

## Risks / Trade-offs

**Practical mode remains weaker**
That is intentional. The point is to make the weakness visible and contain it to
declared runs.
