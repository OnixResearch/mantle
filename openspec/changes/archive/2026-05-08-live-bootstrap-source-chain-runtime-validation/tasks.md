# Tasks: Complete source-chain runtime validation

## Validation

- [x] V1 Confirm binutils-tcc, gcc-4.0, and gcc-4.7 runtime validation evidence is available or record blockers. Evidence: `evidence/V1-prerequisite-evidence-audit.md`. [covers=bootstrap.source.chain.runtime-validation]
- [x] V2 Validate transition builds from `bootstrap/binutils-tcc.ncl` through `bootstrap/gcc-10.ncl`. Evidence: `evidence/V2-transition-build-status.md` records explicit non-promotion: binutils-tcc bridge boundary archived, gcc-4.0 negative boundary archived, gcc-4.7 still active, so gcc-10 transition proof is blocked. [covers=bootstrap.source.chain.runtime-validation]
- [x] V3 Validate final provider stages and normalized contract. Evidence: `evidence/V3-final-provider-status.md` records final-provider stages blocked by incomplete transition evidence and preserves negative/conditional contract status. [covers=bootstrap.source.chain.runtime-validation]
- [x] V4 Validate final source-built provider proof and status promotion. Evidence: `evidence/V4-source-built-provider-proof-status.md` denies status promotion because gcc-4.0/gcc-4.7/final-provider prerequisites are not satisfied. [covers=bootstrap.source.chain.runtime-validation]
- [x] V5 Run OpenSpec validation/gates before archive. Evidence: `evidence/V5-openspec-validation.md`. [covers=bootstrap.source.chain.runtime-validation]
