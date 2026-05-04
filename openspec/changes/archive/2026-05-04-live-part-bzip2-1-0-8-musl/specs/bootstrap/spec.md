## ADDED Requirements

### Requirement: Live-bootstrap part bzip2 1.0.8 (musl) is independently tracked [r[bootstrap.part.bzip2.1.0.8.musl]]
Crunch MUST track the live-bootstrap part `bzip2 1.0.8` as an independent bootstrap change bound to `bootstrap/bzip2-1.0.8-musl.ncl`.

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/bzip2-1.0.8-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated [r[bootstrap.part.bzip2.1.0.8.musl.evidence-isolated]]

- GIVEN implementation work touches `bootstrap/bzip2-1.0.8-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/bzip2-1.0.8-musl.ncl`
- AND it records a successful `crunch build bootstrap/bzip2-1.0.8-musl.ncl` transcript
- AND it records a smoke check for the produced output contract

#### Scenario: Downstream blockers stay local [r[bootstrap.part.bzip2.1.0.8.musl.downstream-local]]

- GIVEN a downstream bootstrap stage fails after `bootstrap/bzip2-1.0.8-musl.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure
