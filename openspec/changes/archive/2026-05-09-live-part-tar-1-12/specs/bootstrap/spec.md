## ADDED Requirements

### Requirement: Live-bootstrap part tar 1.12 is independently tracked
Crunch MUST track the live-bootstrap part `tar 1.12` as an independent bootstrap change bound to `bootstrap/tar-tcc.ncl`.
ID: bootstrap.part.tar.1.12

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/tar-tcc.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/tar-tcc.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/tar-tcc.ncl`
- AND it records either a successful `crunch build bootstrap/tar-tcc.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host tar, Nix-provided tar, or legacy archive tools for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/tar-tcc.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure
