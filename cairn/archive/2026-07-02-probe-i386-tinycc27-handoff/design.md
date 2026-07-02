## Context

Archived i386 evidence records two important facts: native i386 execution is feasible, and the Mes runtime/header layout can now create a real `libtcc1.o` and `libtcc1.a`. The next blocker remains the TinyCC 0.9.27 object handoff, previously recorded as `tcc27_compile_object` with rc 139.

This change keeps the probe below Make. It should answer whether `tcc26-i386` plus the repaired i386 Mes runtime can compile the selected TinyCC 0.9.27 source object, and if not, what exact compiler/runtime failure is first.

## Decisions

### 1. Keep the proof sibling-only

**Choice:** Use a diagnostic or spike derivation that does not switch production bootstrap derivations to i386.

**Rationale:** The path is promising but not ready for production. A sibling proof can move the evidence boundary without destabilizing the amd64 bootstrap chain.

### 2. Use real runtime artifacts, not placeholder success

**Choice:** The accepted handoff evidence must use the real i386 Mes runtime archive. Placeholder archives may be used only to continue diagnostic exploration and must be reported as blocked.

**Rationale:** The previous blocker work specifically separated partial layout progress from complete runtime-library success.

### 3. Summary is the primary contract

**Choice:** The derivation writes a compact summary naming status, selected source/object, runtime inputs, rc/signal when failing, output digest when succeeding, and next action.

**Rationale:** Generated logs can be large. Review should depend on a deterministic summary plus digest references.

## Risks / Trade-offs

- The probe may still segfault before a useful object. That is acceptable if the summary names the first blocker and preserves non-claim wording.
- i386 execution depends on host kernel support. Validation must distinguish unsupported host execution from a TinyCC handoff failure.
