## Context

`bootstrap/gcc-4.0.ncl` currently bridges multiple GCC generator executables. `genpreds` still carries generic `bootstrap genpreds ... stub` labels in both header and C source output, which keeps placeholder inventory ambiguous even though the bridge is an intentional empty-predicate boundary.

## Goals / Non-Goals

**Goals:**
- Replace generic `genpreds` stub labels with explicit empty-predicate boundary markers.
- Check both header and source generator outputs inside the derivation.
- Keep parity status partial and evidence-backed.

**Non-Goals:**
- Build or execute native GCC 4.0 `genpreds`.
- Claim target predicate correctness or complete `gcc.4.0` parity.

## Decisions

### 1. Check generated boundary outputs directly

**Choice:** Invoke the bridged `genpreds` executable in the derivation after the native make attempt and grep the generated header/source outputs for guards, boundary markers, and legacy-label absence.

**Rationale:** This mirrors the existing `gencheck` and generator-header boundary checks and catches drift without widening scope.

**Alternative:** Only rename comments and rely on the placeholder inventory. Rejected because inventory does not prove the bridged executable emits the intended shapes.

## Risks / Trade-offs

**Boundary hardening can look like correctness.** Mitigation: spec, comments, parity notes, and tests keep `gcc.4.0` partial and state that native `genpreds` correctness remains pending.
