## ADDED Requirements

### Requirement: rebuilt musl 1.1.24 runtime validation waits for prerequisite execution proof [r[bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation]]
The system MUST keep the rebuilt musl pass runtime proof incomplete until prerequisite runtime blockers are resolved and the produced musl output contract is smoke-tested.

#### Scenario: Source-level hardening is not runtime proof [r[bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation.source-hardening]]
- **GIVEN** `bootstrap/musl-1.1.24-tcc-musl.ncl` fails closed on missing startup objects
- **WHEN** the derivation has not been built successfully in Crunch
- **THEN** runtime validation remains incomplete

#### Scenario: Rebuilt musl contract is proven [r[bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation.output-contract]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/musl-1.1.24-tcc-musl.ncl` builds successfully
- **THEN** the evidence proves `libc.a`, installed headers, and a startup object exist without host fallback
