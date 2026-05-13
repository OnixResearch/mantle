## Context

`bootstrap/gcc-4.0.ncl` bridges multiple GCC generator executables. `genattr` currently emits a minimal `insn-attr.h` shape, but that shape is not checked after the native make attempt and lacks an explicit boundary marker.

## Goals / Non-Goals

**Goals:**
- Mark `genattr` as an explicit empty-attribute header boundary.
- Check the generated header inside the derivation.
- Keep parity status partial and evidence-backed.

**Non-Goals:**
- Build or trust native GCC 4.0 `genattr`.
- Claim instruction attribute correctness or complete `gcc.4.0` parity.

## Decisions

### 1. Check the generated header directly

**Choice:** Invoke the bridged `genattr` executable after the native make attempt and grep the emitted header for its guard, empty-attribute marker, disabled `HAVE_ATTR_enabled` contract, and legacy-label absence.

**Rationale:** This mirrors the existing generator boundary checks and catches drift without widening scope.

**Alternative:** Only rename the bridge output. Rejected because the derivation would not prove the installed generator emits the intended boundary shape.

## Risks / Trade-offs

**Boundary hardening can look like correctness.** Mitigation: spec, comments, parity notes, and tests keep `gcc.4.0` partial and state that native `genattr` correctness remains pending.
