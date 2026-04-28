## ADDED Requirements

### Requirement: Live-bootstrap part source-built full seed normalization is independently tracked
Crunch MUST track the live-bootstrap part `gcc 10.5.0 through binutils 2.41` as an independent bootstrap change bound to `bootstrap/seed-full.ncl`.
ID: bootstrap.part.seed.full

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/seed-full.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/seed-full.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/seed-full.ncl`
- AND it records a successful `crunch build bootstrap/seed-full.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/seed-full.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure
