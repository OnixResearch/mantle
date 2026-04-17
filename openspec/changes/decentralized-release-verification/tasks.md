# Tasks: Decentralized release verification

## Phase 1: Technical attestation foundations

- [ ] Define a canonical release-attestation schema that binds a release
      identifier to the verified release-evidence manifest digest, published
      binary digest set, proof identity, and workflow identity
- [ ] Define a canonical witness-attestation schema that binds one witness
      result to one release-attestation digest, including the bounded
      `rebuild_environment_summary` field set and canonical key ordering from
      design decision 3
- [ ] Define canonical compact-JSON and digest rules for both attestation
      types
- [ ] Define the versioned signature suite and detached-signature encoding for
      witness attestations and any signed release-attestation material,
      including the `<key-name>:<base64-ed25519-signature>` wire format from
      design decision 4
- [ ] Define the published binary digest-set schema as deterministic per-output
      `(name, algorithm, digest)` tuples and define comparison semantics for
      witness results
- [ ] Define trust-tier output fields that separate technical class, policy
      status, and final class, and enumerate the initial technical classes
      `bundle-consistent`, `self-proof-valid`, `external-witness-match`, plus
      the policy-dependent final class `quorum-satisfied` (design decision 6)

## Phase 2: Social trust policy foundations

- [ ] Define a social-policy schema for trusted roles and quorum thresholds
      with file-based signer lists
- [ ] Define independence requirements so the same actor or witness domain
      cannot satisfy all required witness slots
      (release-verification-social / Same actor cannot satisfy all required witness slots)
- [ ] Define how the verifier consumes trusted-key and role bindings without
      embedding that policy into release-attestation or witness-attestation
      digests, ensuring policy updates alone never change attestation digests
      (release-verification-social / Policy update does not change technical artifact digests)
- [ ] Define policy outcomes for technically valid but policy-insufficient
      witness sets
      (release-verification-social / Insufficient quorum fails policy even after technical agreement)
- [ ] Define revocation and dispute-handling expectations for previously
      published witness material, including file-based revocation input from a
      verifier-local policy artifact
      (release-verification-social / Revoked witness no longer satisfies release policy;
      release-verification-social / Revocation input comes from a file-based policy artifact)
- [ ] Define how satisfied quorum promotes the final class to
      `quorum-satisfied`
      (release-verification-social / Satisfied quorum promotes final release class)

## Phase 3: CLI and workflow plan

- [ ] Define CLI entry points for inspecting release attestations, inspecting
      witness attestations, and verifying witness sets against one release
- [ ] Define machine-readable verifier output for technical failures, policy
      failures, and final trust-tier reporting, reusing the established
      envelope JSON convention for release and witness attestation display
      and the deterministic `stored_path` semantics from design decision 7
      (design decision 7)
- [ ] Define file-based attestation and policy discovery layout for the first
      phase, including the verification-directory layout for
      `release-attestation.json`, `witnesses/*.json`, `policy.json`, and
      `revocations.json`
      (release-verification-tech / Verifier discovers witness files from the verification directory)
- [ ] Define example operator workflows for self-proof-only, single external
      witness, and quorum-satisfied release publication
- [ ] Define publication guidance for release evidence, release attestation,
      witness attestations, and local or published policy profiles

## Phase 4: Validation design

- [ ] Add a test that canonical release attestation digest is stable across
      repeated serialization
      (release-verification-tech / Canonical release attestation digest is stable)
- [ ] Add a test that canonical witness attestation digest is stable across
      repeated serialization
      (design verification strategy / canonical witness-attestation bytes are stable across repeated serialization)
- [ ] Add a test that a release attestation binds the release-evidence manifest
      digest and published binary digest set
      (release-verification-tech / Release attestation binds release evidence)
- [ ] Add a test that a witness naming a wrong release-attestation digest is
      rejected
      (release-verification-tech / Witness with wrong release reference is rejected)
- [ ] Add a test that a witness with a wrong rebuilt output digest is rejected
      (release-verification-tech / Witness with wrong rebuilt output digest is rejected)
- [ ] Add a test that a witness with a missing or invalid signature is rejected
      before digest comparison
      (release-verification-tech / Witness with invalid signature is rejected)
- [ ] Add a test that a release attestation with a missing or invalid
      signature is rejected before witness evaluation
      (design decision 4 / release attestation signature verification)
- [ ] Add a test that the verifier discovers witness files from the
      verification-directory layout alone
      (release-verification-tech / Verifier discovers witness files from the verification directory)
- [ ] Add a test that discovery rejects witness attestations with missing
      `.sig` sidecars before digest comparison
      (design verification strategy / discovery rejects missing `.sig` sidecars before digest comparison)
- [ ] Add a test that technically valid witness material can still fail policy
      evaluation
      (release-verification-tech / Technically valid but policy-insufficient witness set)
- [ ] Add a test that zero witnesses yield technical class `self-proof-valid`
      (release-verification-tech / Self-proof tier remains technically valid without witnesses)
- [ ] Add a test that one matching external witness raises technical class to
      `external-witness-match`
      (release-verification-tech / Matching external witness raises technical class)
- [ ] Add a test that changing `policy.json` does not change attestation
      digests
      (release-verification-social / Policy update does not change technical artifact digests)
- [ ] Add a test that insufficient quorum fails policy while keeping technical
      success
      (release-verification-social / Insufficient quorum fails policy even after technical agreement)
- [ ] Add a test that satisfied quorum promotes final class to
      `quorum-satisfied`
      (release-verification-social / Satisfied quorum promotes final release class)
- [ ] Add a test that the same witness domain cannot satisfy independence rules
      alone
      (release-verification-social / Same actor cannot satisfy all required witness slots)
- [ ] Add a test that a revoked witness key or witness attestation digest
      degrades policy status without changing the technical result
      (release-verification-social / Revoked witness no longer satisfies release policy)
- [ ] Add a test that revocation input is loaded from `revocations.json` before
      policy evaluation
      (release-verification-social / Revocation input comes from a file-based policy artifact)
- [ ] Add a test that discovery applies `revocations.json` before quorum
      counting
      (design verification strategy / discovery applies `revocations.json` before quorum counting)

## Validation

- [x] Run `openspec validate decentralized-release-verification`
- [x] Run a tasks gate after the technical and social specs, design, and task
      wording all stabilize
  - Evidence: `openspec_gate stage=tasks change=decentralized-release-verification`
    returned `Tasks pass. No edits needed.` after the Phase 4 traceability
    expansion.
