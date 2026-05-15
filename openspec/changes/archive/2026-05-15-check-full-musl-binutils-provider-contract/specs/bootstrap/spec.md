## MODIFIED Requirements

### Requirement: Source-built bootstrap chain implementation

The chain MUST replace `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, and `bootstrap/seed-full.ncl` placeholder derivations before they can satisfy bootstrap completion status. Stage validation MUST record command transcripts for the ordered inventory (`bootstrap/stage0-posix.ncl`, `bootstrap/mes.ncl`, `bootstrap/tinycc.ncl`, `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, `bootstrap/seed-full.ncl`).

#### Scenario: Full musl/binutils provider-contract receipt is missing [r[bootstrap.source.chain.implementation.full-musl-binutils-missing-receipt]]

- GIVEN `bootstrap/musl-full.ncl` and `bootstrap/binutils-full.ncl` exist
- BUT `bootstrap/evidence/full-musl-binutils-provider-contract.json` is absent
- WHEN the bootstrap parity report evaluates `full-musl-binutils`
- THEN the row remains a live-bootstrap and Guix blocker
- AND the row notes identify the missing provider-contract receipt

#### Scenario: Full musl/binutils provider-contract receipt is evidence-backed partial [r[bootstrap.source.chain.implementation.full-musl-binutils-contract-partial]]

- GIVEN `bootstrap/evidence/full-musl-binutils-provider-contract.json` has the expected schema, derivation paths, `contract-only` status, required musl/binutils derivation markers, and explicit partial parity effect
- AND every required musl marker appears in `bootstrap/musl-full.ncl`
- AND every required binutils marker appears in `bootstrap/binutils-full.ncl`
- WHEN the bootstrap parity report evaluates `full-musl-binutils`
- THEN the row may report evidence-backed `partial`
- AND it MUST NOT report `complete` or unblock live-bootstrap or Guix parity until full toolchain correctness and source-root proof exist

#### Scenario: Full musl/binutils provider-contract marker drift fails closed [r[bootstrap.source.chain.implementation.full-musl-binutils-marker-drift]]

- GIVEN the full musl/binutils provider-contract receipt requires a marker that no longer appears in either derivation
- WHEN the bootstrap parity report evaluates `full-musl-binutils`
- THEN the row remains a blocker
- AND the row notes identify the missing marker and affected derivation
