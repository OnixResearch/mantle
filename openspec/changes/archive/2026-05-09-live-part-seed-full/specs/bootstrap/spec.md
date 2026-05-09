## ADDED Requirements

### Requirement: Live-bootstrap part source-built full seed normalization is independently tracked
Crunch MUST track the live-bootstrap part `gcc 10.5.0 through binutils 2.41` as an independent bootstrap change bound to `bootstrap/seed-full.ncl`.
ID: bootstrap.part.seed.full

The part scope MUST include only the source pins, patches, derivation logic, output contract, and evidence needed for `bootstrap/seed-full.ncl` and direct predecessor compatibility. Broader chain validation MAY depend on this part, but MUST NOT replace this part's own build and smoke evidence. If declared predecessor toolchains are unavailable or unvalidated, the part MUST record prerequisite-gated evidence and MUST NOT claim full-source seed promotion, leakage-clean runtime proof, or end-to-end source-built success.

#### Scenario: Part evidence is isolated

- GIVEN implementation work touches `bootstrap/seed-full.ncl`
- WHEN the part is marked complete
- THEN the change records source-pin audit evidence for `bootstrap/seed-full.ncl`
- AND it records the normalized seed output contract and source-chain audit evidence
- AND it records either a successful `crunch build bootstrap/seed-full.ncl` transcript or explicit prerequisite-gated build evidence
- AND it does not promote the full-source seed until GCC 10.5.0, musl 1.2.5, and binutils 2.41 predecessor proofs are trusted

#### Scenario: Downstream blockers stay local

- GIVEN a predecessor or downstream bootstrap stage fails before full `bootstrap/seed-full.ncl` promotion proof exists
- WHEN this part's evidence is reviewed
- THEN the failure does not invalidate this part's completed normalization-contract evidence
- AND a separate part change tracks the predecessor or downstream runtime failure
