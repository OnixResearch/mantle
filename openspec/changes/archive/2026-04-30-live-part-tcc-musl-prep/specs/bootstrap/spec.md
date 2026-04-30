## ADDED Requirements

### Requirement: Live-bootstrap part tcc musl prep is independently tracked
Crunch MUST track the live-bootstrap part `musl 1.1.24 and musl_target` as an independent bootstrap change bound to `bootstrap/tcc-musl-prep.ncl`.
ID: bootstrap.part.tcc.musl.prep

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tcc-musl-prep.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tcc-musl-prep.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tcc-musl-prep.ncl`
- AND it records a successful `crunch build bootstrap/tcc-musl-prep.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tcc-musl-prep.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure
