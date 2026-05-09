## ADDED Requirements

### Requirement: Live-bootstrap part m4 1.4.7 is independently tracked
Crunch MUST track the live-bootstrap part `m4 1.4.7` as an independent bootstrap change bound to `bootstrap/m4-1.4.7-musl.ncl`.
ID: bootstrap.part.m4.1.4.7

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/m4-1.4.7-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor outputs are unavailable or unvalidated, or if the current derivation intentionally uses a bootstrap bridge instead of a full direct GNU m4 binary, the part MUST record gate evidence and MUST NOT claim full toolchain promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/m4-1.4.7-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/m4-1.4.7-musl.ncl`
- AND it records either a successful `crunch build bootstrap/m4-1.4.7-musl.ncl` transcript or explicit prerequisite/runtime-gated build evidence
- AND it records either a smoke check for the produced output contract or explicit evidence that no full direct-build output path exists yet
- AND it does not substitute host m4, Nix-provided m4, or legacy tool outputs for bootstrap proof

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor, direct GNU m4, or downstream bootstrap stage fails before full `bootstrap/m4-1.4.7-musl.ncl` runtime proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening evidence
- AND a separate part change tracks the predecessor, direct-build, or downstream runtime failure
