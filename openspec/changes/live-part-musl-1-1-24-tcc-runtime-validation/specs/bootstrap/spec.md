## ADDED Requirements

### Requirement: musl 1.1.24 tcc runtime validation waits for prerequisite execution proof [r[bootstrap.part.musl.1.1.24.tcc.runtime-validation]]
The system MUST keep the first musl pass runtime proof incomplete until prerequisite make/tcc execution blockers are resolved and the produced musl output contract is smoke-tested.

#### Scenario: Source-level hardening is not runtime proof [r[bootstrap.part.musl.1.1.24.tcc.runtime-validation.source-hardening]]
- **GIVEN** `bootstrap/musl-1.1.24-tcc.ncl` fails closed on missing startup objects
- **WHEN** the derivation has not been built successfully in Crunch
- **THEN** runtime validation remains incomplete

#### Scenario: First musl contract is proven [r[bootstrap.part.musl.1.1.24.tcc.runtime-validation.output-contract]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/musl-1.1.24-tcc.ncl` builds successfully
- **THEN** the evidence proves `libc.a`, installed headers, and a startup object exist without host fallback
