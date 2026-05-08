## ADDED Requirements

### Requirement: Live-bootstrap part autoconf 2.52 is independently tracked
Crunch MUST track the live-bootstrap part `autoconf 2.52` as an independent bootstrap change bound to `bootstrap/autoconf-2.52.ncl`.
ID: bootstrap.part.autoconf.2.52

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/autoconf-2.52.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/autoconf-2.52.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/autoconf-2.52.ncl`
- AND it records a successful `crunch build bootstrap/autoconf-2.52.ncl` transcript when declared prerequisite providers exist
- AND if declared prerequisite providers are absent, blocked, or unvalidated, it records fail-closed blocker evidence without substituting host or legacy providers
- AND it records a smoke check for the produced output contract when an output exists

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/autoconf-2.52.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure
