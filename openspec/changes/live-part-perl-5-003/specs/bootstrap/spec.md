## ADDED Requirements

### Requirement: Live-bootstrap part perl 5.003 is independently tracked
Crunch MUST track the live-bootstrap part `perl 5.003` as an independent bootstrap change bound to `bootstrap/perl-5.003-musl.ncl`.
ID: bootstrap.part.perl.5.003

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/perl-5.003-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/perl-5.003-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/perl-5.003-musl.ncl`
- AND it records a successful `crunch build bootstrap/perl-5.003-musl.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/perl-5.003-musl.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure
