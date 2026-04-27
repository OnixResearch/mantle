# Complete live-bootstrap source chain

## Why

Parent change `repair-live-bootstrap-archive-status` restored honest status after
`live-bootstrap-seed-chain` and `live-bootstrap-intermediate-tools` were archived
with deferred validation and placeholder derivations. The full-source bootstrap
claim remains blocked because the live-bootstrap ladder is still not functional.

Current placeholders:

- `bootstrap/binutils-tcc.ncl`
- `bootstrap/gcc-4.0.ncl`
- `bootstrap/gcc-4.7.ncl`
- `bootstrap/gcc-10.ncl`
- `bootstrap/musl-full.ncl`
- `bootstrap/binutils-full.ncl`
- `bootstrap/seed-full.ncl`

`bootstrap/gcc-4.0.ncl` must implement the mandatory gcc-4.0.4 transition before
the full chain can satisfy the normalized seed provider contract.

## What Changes

- Replace placeholder pre-GCC, GCC-ladder, musl, binutils, and final-provider
  derivations with functional chain-internal builds.
- Validate each stage in order, from `bootstrap/stage0-posix.ncl` through
  `bootstrap/seed-full.ncl`.
- Bind final self-build proof evidence to the source-built provider, including
  provider kind, manifest digest, provider output digest, proof bundle digest,
  and documentation separating remaining trust roots from eliminated
  binary-provider trust.

## Capabilities

### Modified Capabilities

- `bootstrap.fullsource.claim.evidence`: source-built chain implementation and
  proof evidence become the only path that may promote full-source bootstrap
  status.
- `bootstrap.stagex.selfbuild.proof`: StageX-class proof evidence is mandatory
  for this change's final completion; if the proof cannot record those fields,
  StageX-class status must remain blocked and the unfinished proof work must stay
  active.

## Scope

- **In scope**: Functional live-bootstrap derivations, stage-by-stage validation,
  source-built provider proof metadata, and bootstrap-maturity docs/status proof.
- **Out of scope**: Removing the legacy musl.cc seed before the source-built
  provider passes selftest, integration-test, and self-build proof.

## Evidence Needed

This change is complete only when transcripts exist for every stage build,
`bootstrap/selftest.ncl`, `bootstrap/integration-test.ncl`, and `crunch
self-build` with the source-built provider. Full-source evidence must include
provider kind, manifest digest, provider output digest, proof bundle digest, and
the docs update that keeps trust-root status explicit. StageX-class evidence is
mandatory before this change can archive: audited seed digest, lineage manifest
digest, stage graph digest, normalized provider digest, staged source digest,
stage1/stage2 binary digests, bootstrap-tool digests, protected execution audit
digest when used, final proof bundle digest, and canonical reproducibility report
digest.
