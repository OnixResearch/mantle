## Context

Commit `957754d6` established source-frontier v2 evidence around a c-parse/decl0 diagnostic probe. The diagnostic log and derivation show the probe now passes the copied `fd_bad` branch when forced false and reaches markers `4`, `6`, `5`, `Y`, and `w`.

## Goals

- Record the narrower fdopen/output-return diagnostic frontier as source-frontier v3 evidence.
- Keep the evidence fail-closed by requiring exact diagnostic derivation markers.
- Preserve all non-promotion semantics for `gcc.4.0`.

## Non-Goals

- Do not claim native GCC 4.0 correctness.
- Do not claim full `c-parse.o` or native `cc1` source build success.
- Do not broaden the diagnostic derivation beyond the bounded probe.

## Decisions

### 1. Receipt-only frontier reduction

**Choice:** Update the checked source-frontier receipt and validation schema to v3 rather than touching production GCC build behavior.

**Rationale:** The durable value is narrowing the diagnosed frontier without destabilizing bootstrap derivations.

### 2. Exact marker contract

**Choice:** Require diagnostic markers for the copied `fd_bad` bypass and subsequent `fdopen`/output-return markers.

**Rationale:** Marker drift should invalidate the receipt rather than allowing stale frontier claims.

## Risks

- **Archive drift:** OpenSpec archive may overwrite the cumulative GCC ladder text. Mitigation: inspect and restore cumulative spec text before commit if needed.
- **Overclaim:** Mitigation: receipt/spec/test wording must explicitly keep `gcc.4.0` partial and source-frontier-only.
