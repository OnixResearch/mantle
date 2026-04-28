## ADDED Requirements

### Requirement: Live-bootstrap part tinycc 0.9.26 is independently tracked
Crunch MUST track the live-bootstrap part `tinycc 0.9.26` as an independent bootstrap change bound to `bootstrap/tinycc-mes.ncl`.
ID: bootstrap.part.tinycc.0.9.26

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tinycc-mes.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tinycc-mes.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tinycc-mes.ncl`
- AND it records a successful `crunch build bootstrap/tinycc-mes.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/tinycc-mes.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure
