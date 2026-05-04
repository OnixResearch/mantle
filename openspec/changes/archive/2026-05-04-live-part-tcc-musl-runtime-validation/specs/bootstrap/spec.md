## ADDED Requirements

### Requirement: tcc musl runtime validation waits for prerequisite execution proof [r[bootstrap.part.tcc.musl.runtime-validation]]
The system MUST keep the musl-linked TinyCC runtime proof incomplete until prerequisite runtime blockers are resolved and the produced compiler can compile a trivial C program using its installed runtime archive.

#### Scenario: Runtime archive is mandatory [r[bootstrap.part.tcc.musl.runtime-validation.libtcc1]]
- **GIVEN** `bootstrap/tcc-musl.ncl` produces `bin/tcc`
- **WHEN** `lib/tcc/libtcc1.a` is missing
- **THEN** runtime validation remains incomplete

#### Scenario: Compiler smoke proves output [r[bootstrap.part.tcc.musl.runtime-validation.compile-smoke]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/tcc-musl.ncl` builds and compiles a trivial C program successfully
- **THEN** the evidence records output path, smoke results, fallback status, and leakage-scan results
