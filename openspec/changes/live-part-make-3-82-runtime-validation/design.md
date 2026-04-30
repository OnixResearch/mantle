# Design: make 3.82 runtime validation

## Context

The parent make part has implementation and source-pin evidence, but its runtime checks depend on the TinyCC amd64 varargs/string-construction repair. Existing diagnostic evidence is intentionally not PASS evidence.

## Decisions

### 1. Keep runtime proof behind the repair change

**Choice:** defer V2-V4 to this runtime-validation follow-up and reference `repair-make-tcc-amd64-varargs` as the concrete blocker.

**Rationale:** archiving the parent implementation/audit change should not claim make execution is proven before the actual amd64 repair lands.

### 2. Require real Makefile execution smoke

**Choice:** validation must run a simple Makefile and confirm correct target execution, not merely check `make --version`.

**Rationale:** diagnostic smoke already proved version output while Makefile execution still segfaulted.

## Validation

Build `bootstrap/make-tcc.ncl`, run GNU Make 3.82 version and Makefile execution smoke, scan for host leakage, and run OpenSpec validation/gates.
