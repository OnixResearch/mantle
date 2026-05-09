## ADDED Requirements

### Requirement: Live-bootstrap part gawk 3.0.4 is independently tracked
Crunch MUST track the live-bootstrap part `gawk 3.0.4` as an independent bootstrap change bound to `bootstrap/gawk-3.0.4-musl.ncl`.
ID: bootstrap.part.gawk.3.0.4

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/gawk-3.0.4-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim build, smoke, leakage-clean, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/gawk-3.0.4-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/gawk-3.0.4-musl.ncl`
- AND it records either a successful `crunch build bootstrap/gawk-3.0.4-musl.ncl` transcript or explicit prerequisite-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no output path exists yet
- AND it does not substitute host awk/gawk, Nix-provided awk/gawk, host GCC, or legacy compiler outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/gawk-3.0.4-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor or downstream runtime failure
