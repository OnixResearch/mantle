# Decentralized release verification

## Why

Crunch release evidence proves that one packaged release bundle is internally
consistent and linked to one successful full proof run, but it does not yet
prove independent rebuild agreement or resistance to single-builder and
single-maintainer compromise.

To approach stronger release-trust claims, crunch needs a decentralized release
verification model where multiple independent rebuilders can attest to the same
release candidate and verifiers can evaluate both technical agreement and
social-policy sufficiency.

## What Changes

- add a technical specification for canonical compact-JSON release
  attestations, witness attestations, per-output `(name, algorithm, digest)`
  digest tuples, digest comparison, and verifier status reporting
- add a separate social-policy specification for witness roles, quorum rules,
  independence requirements, file-based revocation input, and dispute handling
- define technical classes (`bundle-consistent`, `self-proof-valid`,
  `external-witness-match`) and a policy-dependent final release class
  (`quorum-satisfied`) for progressive decentralized verification
- keep organization-specific policy outside canonical artifact digests so the
  protocol stays reusable
- keep attestation storage and discovery file-based in the first phase so the
  design does not depend on a transparency log or external witness service
- treat the attestation digest as the release-attestation identity, and require
  design work to define the versioned signature suite and detached-signature
  encoding used by witness material

## Capabilities

### New Capabilities

- `release-attestations`: a release can publish a canonical attestation that
  binds release evidence to the expected output digest set
- `witness-attestations`: independent rebuilders can publish signed witness
  records against one release attestation identity
- `tiered-release-verification`: verifiers can report technical validity and
  policy sufficiency separately
- `externalized-trust-policy`: organizations can apply local signer, quorum,
  and independence rules without changing release-attestation hashes

## Impact

- **Files**: new change-local specs under
  `openspec/changes/decentralized-release-verification/specs/`
- **Architecture**: introduces a two-layer trust model instead of one bundled
  verification story
- **CLI**: future work will extend `crunch release verify` or sibling commands
  to consume release and witness attestations plus policy files
- **Testing**: future work will need canonical compact-JSON stability tests,
  signature-suite tests, witness mismatch tests, local file-discovery tests,
  and policy/quorum tests

## Non-Goals

- prove absolute trust or eliminate all stage0 and hardware assumptions
- require one global signer policy for all crunch users
- embed mutable local policy into release-attestation digests
- implement transparency-log infrastructure in this planning change
- require an external witness-discovery or revocation service in this planning
  change; first-phase discovery and revocation stay file-based
