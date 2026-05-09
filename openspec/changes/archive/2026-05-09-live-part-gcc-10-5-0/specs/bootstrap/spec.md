## ADDED Requirements

### Requirement: Live-bootstrap part gcc 10.5.0 is independently tracked
Crunch MUST track the live-bootstrap part `gcc 10.5.0` as an independent bootstrap change bound to `bootstrap/gcc-10.ncl`.
ID: bootstrap.part.gcc.10.5.0

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/gcc-10.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, final-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/gcc-10.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/gcc-10.ncl`
- AND it records either a successful `crunch build bootstrap/gcc-10.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host GCC, Nix-provided GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/gcc-10.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure
