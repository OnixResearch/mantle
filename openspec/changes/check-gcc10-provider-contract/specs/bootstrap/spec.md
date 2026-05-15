## MODIFIED Requirements

### Requirement: Source-built bootstrap chain implementation

Mantle MUST implement the live-bootstrap stage chain through a source-built provider that satisfies the normalized seed contract without using the legacy musl.cc binary provider.
ID: bootstrap.source.chain.implementation

The chain MUST replace `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, and `bootstrap/seed-full.ncl` placeholder derivations before they can satisfy bootstrap completion status. Stage validation MUST record command transcripts for the ordered inventory (`bootstrap/stage0-posix.ncl`, `bootstrap/mes.ncl`, `bootstrap/tinycc.ncl`, `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, and `bootstrap/seed-full.ncl`), `bootstrap/selftest.ncl`, `bootstrap/integration-test.ncl`, and final self-build proof. Validation MUST fail closed when a stage emits placeholder text, exits through a deferred task, or falls back to the legacy provider. The final provider MUST expose the normalized seed contract fields consumed by later bootstrap derivations. The bootstrap parity report MUST require checked provider-contract evidence before treating `gcc.10` as evidence-backed partial; that receipt MUST be contract-only, MUST name `bootstrap/gcc-10.ncl`, MUST validate required configure/build/install/smoke markers in the derivation, and MUST NOT make the row complete without full native/source-chain correctness evidence.

#### Scenario: GCC 10 provider-contract receipt is missing [r[bootstrap.source.chain.implementation.gcc10-missing-receipt]]

- GIVEN `bootstrap/gcc-10.ncl` exists
- BUT `bootstrap/evidence/gcc-10-provider-contract.json` is absent
- WHEN the bootstrap parity report evaluates `gcc.10`
- THEN the row remains a live-bootstrap and Guix blocker
- AND the row notes identify the missing provider-contract receipt

#### Scenario: GCC 10 provider-contract receipt is evidence-backed partial [r[bootstrap.source.chain.implementation.gcc10-contract-partial]]

- GIVEN `bootstrap/evidence/gcc-10-provider-contract.json` has the expected schema, derivation, `contract-only` status, required derivation markers, and explicit partial parity effect
- AND every required marker appears in `bootstrap/gcc-10.ncl`
- WHEN the bootstrap parity report evaluates `gcc.10`
- THEN the row may report evidence-backed `partial`
- AND it MUST NOT report `complete` or unblock live-bootstrap or Guix parity until native/full GCC 10 correctness and source transcripts exist

#### Scenario: GCC 10 provider-contract marker drift fails closed [r[bootstrap.source.chain.implementation.gcc10-marker-drift]]

- GIVEN the GCC 10 provider-contract receipt requires a marker that no longer appears in `bootstrap/gcc-10.ncl`
- WHEN the bootstrap parity report evaluates `gcc.10`
- THEN the row remains a blocker
- AND the row notes identify the missing marker
