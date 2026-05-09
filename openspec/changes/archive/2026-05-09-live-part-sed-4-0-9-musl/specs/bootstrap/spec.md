## ADDED Requirements

### Requirement: Live-bootstrap part sed 4.0.9 (musl) is independently tracked
Crunch MUST track the live-bootstrap part `sed 4.0.9` as an independent bootstrap change bound to `bootstrap/sed-4.0.9-musl.ncl`.
ID: bootstrap.part.sed.4.0.9.musl

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/sed-4.0.9-musl.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. Because this derivation currently preserves an explicit `sed-tcc` bridge while the TinyCC/musl source compile boundary is blocked, the part MUST record bridge/gate evidence and MUST NOT claim musl source-build, leakage-clean, tool-provider, or chain-promotion success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/sed-4.0.9-musl.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/sed-4.0.9-musl.ncl`
- AND it records source-pin audit evidence for `bootstrap/sed-4.0.9-musl.ncl`
- AND it records explicit bridge/gate evidence for the `sed-tcc` runtime copy
- AND it does not substitute the bridge output for musl source-build proof

#### Scenario: Downstream blockers stay local

- GIVEN the TinyCC/musl sed source compile boundary remains blocked or a downstream bootstrap stage fails
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed source-hardening and bridge-gate evidence
- AND a separate part change tracks the compile-boundary or downstream runtime failure
