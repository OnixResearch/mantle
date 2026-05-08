## ADDED Requirements

### Requirement: Live-bootstrap part mpc 1.2.1 is independently tracked
Crunch MUST track the live-bootstrap implemented part `mpc-1.2.1` as an independent bootstrap change bound to `bootstrap/mpc-1.2.1.ncl`, while recording that upstream `parts.rst` currently labels the corresponding narrative heading `mpc 3.2.1`.
ID: bootstrap.part.mpc.1.2.1

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/mpc-1.2.1.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/mpc-1.2.1.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/mpc-1.2.1.ncl`
- AND it records a successful `crunch build bootstrap/mpc-1.2.1.ncl` transcript when declared prerequisite providers exist
- AND if declared prerequisite providers are absent, it records fail-closed blocker evidence without substituting host or legacy providers
- AND it records a smoke check for the produced output contract when an output exists

#### Scenario: Downstream blockers stay local

- GIVEN a downstream bootstrap stage fails after `bootstrap/mpc-1.2.1.ncl` succeeds
- WHEN this part's evidence is reviewed
- THEN the downstream failure does not invalidate this part's completed evidence
- AND a separate part change tracks the downstream failure
