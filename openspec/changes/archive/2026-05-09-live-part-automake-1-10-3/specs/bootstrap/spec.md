## ADDED Requirements

### Requirement: Live-bootstrap part automake 1.10.3 is independently tracked
Crunch MUST track the live-bootstrap part `automake 1.10.3` as an independent bootstrap change bound to `bootstrap/automake-1.10.3.ncl`.
ID: bootstrap.part.automake.1.10.3

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/automake-1.10.3.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/automake-1.10.3.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/automake-1.10.3.ncl`
- AND it records either a successful `crunch build bootstrap/automake-1.10.3.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided Automake, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/automake-1.10.3.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure
