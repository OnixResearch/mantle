## Context

GCC 4.0 remains partial. The libiberty demangle seam is not a blocker because demangling is required for runtime C compilation; it is a blocker because the source still carries a generic `*_stub` marker. Replacing that marker with a named disabled-demangle boundary tightens evidence and shrinks the frontier map without pretending to implement full C++ demangling.

## Goals / Non-Goals

**Goals:**
- Remove the generic `libiberty_cp_demangle_bootstrap_stub` marker.
- Install a small, compilable `cp-demangle.c` boundary that documents disabled C++ demangling.
- Update checked frontier evidence and tests.

**Non-Goals:**
- Do not implement real Itanium/C++ demangling.
- Do not mark GCC 4.0 complete.

## Decisions

### 1. Boundary marker over fake implementation

**Choice:** Use `gcc40_cp_demangle_disabled_boundary` and comments describing disabled demangling.

**Rationale:** This is honest evidence: it reduces generic placeholder debt while keeping the semantic limitation explicit.

**Alternative:** Copy full upstream cp-demangle was rejected as too broad for this bounded increment.

## Risks / Trade-offs

**Overclaiming risk** → Tests and frontier evidence continue to classify GCC 4.0 as partial.
