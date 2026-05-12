## Context

The current GCC 4.0 handoff seeds `insn-constants.h` and `insn-flags.h` before the native `make -C gcc` path can rely on real generated headers. Those files are intentionally minimal, but their current wording is a generic bootstrap stub. The native-boundary receipt now lets us tighten this exact seam and prevent drift.

## Goals / Non-Goals

**Goals:**
- Replace generic `genconstants`/`genflags` stub labels with a precise empty-machine-header boundary.
- Check the boundary in the derivation so future edits cannot silently reintroduce the generic stub text.
- Keep `gcc.4.0` partial and blocking parity.

**Non-Goals:**
- Claim real `genconstants`/`genflags` native generator correctness.
- Replace `gengtype` or broader generated source stubs.
- Prove native GCC `cc1` correctness.

## Decisions

### 1. Promote wording plus validation, not completion

**Choice:** Rename the two header bodies to empty-machine boundary headers and add `grep` checks for their guards and absence of old stub labels.

**Rationale:** This is a narrow, verifiable improvement at the next boundary recorded by the native-boundary receipt. It reduces ambiguous placeholder debt without hiding that the real generator remains future work.

**Alternative:** Attempt a full real `genconstants` native build now. Rejected for this increment because the broader native generator graph is still mediated by TinyCC/Mes source-boundary work.

## Risks / Trade-offs

**Risk:** Wording-only changes could overstate progress.  
**Mitigation:** Spec and parity notes explicitly keep `gcc.4.0` partial; validation calls this an empty-machine-header boundary, not real native generator completion.
