## Context

Recent GCC 4.0 increments tightened `genemit`, `gengtype`, `genattr`, `genpreds`, and related generator boundaries without claiming native compiler correctness. `genrecog` remains in a grouped fallback with `genextract`, `genpeep`, `genopinit`, `genoutput`, and `genattrtab`.

## Goals / Non-Goals

**Goals:**
- Give `genrecog` its own deterministic empty-recognition boundary.
- Make the derivation fail closed if the emitted source or legacy label drifts.
- Preserve `gcc.4.0` as partial until real native generator/compiler correctness is proven.

**Non-Goals:**
- Implement real GCC recognizer generation.
- Change the remaining grouped generators.
- Unblock live-bootstrap/Guix parity.

## Decisions

### 1. Split only `genrecog`

**Choice:** Add a dedicated `build/genrecog` case before the remaining grouped case.

**Rationale:** This is the smallest reviewable increment and avoids conflating multiple generator contracts.

**Alternative:** Split the entire grouped family at once. Rejected because the established drain pattern is one bounded seam per commit.

## Risks / Trade-offs

**Boundary hardening can be mistaken for native correctness** → The spec, checks, and report explicitly keep `gcc.4.0` partial.
