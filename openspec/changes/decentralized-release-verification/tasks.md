# Tasks: Decentralized release verification

## Phase 1: Technical attestation foundations

- [x] Define a canonical release-attestation schema that binds a release
      identifier to the verified release-evidence manifest digest, published
      binary digest set, proof identity, and workflow identity
- [x] Define a canonical witness-attestation schema that binds one witness
      result to one release-attestation digest, including the bounded
      `rebuild_environment_summary` field set and canonical key ordering from
      design decision 3
- [x] Define canonical compact-JSON and digest rules for both attestation
      types
- [x] Define the versioned signature suite and detached-signature encoding for
      witness attestations and any signed release-attestation material,
      including the `<key-name>:<base64-ed25519-signature>` wire format from
      design decision 4
- [x] Define the published binary digest-set schema as deterministic per-output
      `(name, algorithm, digest)` tuples and define comparison semantics for
      witness results
- [x] Define trust-tier output fields that separate technical class, policy
      status, and final class, and enumerate the initial technical classes
      `bundle-consistent`, `self-proof-valid`, `external-witness-match`, plus
      the policy-dependent final class `quorum-satisfied` (design decision 6)

## Phase 2: Social trust policy foundations

- [x] Define a social-policy schema for trusted roles and quorum thresholds
      with file-based signer lists
  - Evidence: `crates/crunch-attestation/src/policy.rs` defines `ReleasePolicy`
    with `min_matching_witnesses`, `independence_field`, `trusted_release_signers`,
    and `trusted_witness_signers`. `policy_json_round_trip` test verifies serde.
- [x] Define independence requirements so the same actor or witness domain
      cannot satisfy all required witness slots
      (release-verification-social / Same actor cannot satisfy all required witness slots)
  - Evidence: `evaluate_policy()` counts distinct `witness_identity` values
    among matching witnesses and requires `distinct_identities >= min_matching_witnesses`.
    `same_identity_cannot_satisfy_independence` test verifies two attestations
    from the same identity fail independence with `InsufficientIndependence`.
- [x] Define how the verifier consumes trusted-key and role bindings without
      embedding that policy into release-attestation or witness-attestation
      digests, ensuring policy updates alone never change attestation digests
      (release-verification-social / Policy update does not change technical artifact digests)
  - Evidence: `ReleasePolicy` and `ReleaseRevocations` are separate types
    never included in canonical attestation bytes. `policy_change_does_not_affect_attestation_digests`
    test proves changing policy leaves release attestation digest unchanged.
- [x] Define policy outcomes for technically valid but policy-insufficient
      witness sets
      (release-verification-social / Insufficient quorum fails policy even after technical agreement)
  - Evidence: `evaluate_policy()` returns `PolicyStatus::Insufficient` with
    `PolicyFailureReason::InsufficientQuorum` or `InsufficientIndependence`
    while preserving `TechnicalClass::ExternalWitnessMatch`.
    `insufficient_quorum_fails_policy_with_technical_success` test verifies.
- [x] Define revocation and dispute-handling expectations for previously
      published witness material, including file-based revocation input from a
      verifier-local policy artifact
      (release-verification-social / Revoked witness no longer satisfies release policy;
      release-verification-social / Revocation input comes from a file-based policy artifact)
  - Evidence: `ReleaseRevocations` defines `revoked_witness_keys` and
    `revoked_witness_attestation_digests_blake3`. `evaluate_policy()` filters
    revoked witnesses before quorum counting. Tests:
    `revoked_key_degrades_policy_without_changing_technical_result`,
    `revoked_attestation_digest_degrades_policy`,
    `revocations_applied_before_quorum_counting`.
- [x] Define how satisfied quorum promotes the final class to
      `quorum-satisfied`
      (release-verification-social / Satisfied quorum promotes final release class)
  - Evidence: `evaluate_policy()` calls `FinalClass::resolve(technical, Satisfied)`
    which returns `QuorumSatisfied`. `satisfied_quorum_promotes_final_class`
    test verifies two independent matching witnesses promote to `QuorumSatisfied`.

## Phase 3: CLI and workflow plan

- [ ] Define CLI entry points for inspecting release attestations, inspecting
      witness attestations, and verifying witness sets against one release
- [ ] Define machine-readable verifier output for technical failures, policy
      failures, and final trust-tier reporting, reusing the established
      envelope JSON convention for release and witness attestation display
      and the deterministic `stored_path` semantics from design decision 7
      (design decision 7)
- [x] Define file-based attestation and policy discovery layout for the first
      phase, including the verification-directory layout for
      `release-attestation.json`, `witnesses/*.json`, `policy.json`, and
      `revocations.json`
      (release-verification-tech / Verifier discovers witness files from the verification directory)
  - Evidence: `crates/crunch-attestation/src/discovery.rs` implements
    `VerificationDirectory::discover()` loading release attestation + `.sig`,
    `witnesses/*.json` + `.json.sig`, `policy.json`, and optional
    `revocations.json`. Tests cover zero/two witnesses, missing witnesses dir,
    missing revocations, missing `.sig` sidecar rejection, wrong schema tag,
    and end-to-end discovery → policy evaluation.
- [ ] Define example operator workflows for self-proof-only, single external
      witness, and quorum-satisfied release publication
- [ ] Define publication guidance for release evidence, release attestation,
      witness attestations, and local or published policy profiles

## Phase 4: Validation design

- [x] Add a test that canonical release attestation digest is stable across
      repeated serialization
      (release-verification-tech / Canonical release attestation digest is stable)
  - Evidence: `release::tests::release_attestation_digest_is_stable_across_serializations`
- [x] Add a test that canonical witness attestation digest is stable across
      repeated serialization
      (design verification strategy / canonical witness-attestation bytes are stable across repeated serialization)
  - Evidence: `release::tests::witness_attestation_digest_is_stable_across_serializations`
- [x] Add a test that a release attestation binds the release-evidence manifest
      digest and published binary digest set
      (release-verification-tech / Release attestation binds release evidence)
  - Evidence: `release::tests::release_attestation_binds_manifest_and_binary_digests`
- [x] Add a test that a witness naming a wrong release-attestation digest is
      rejected
      (release-verification-tech / Witness with wrong release reference is rejected)
  - Evidence: `policy::tests::witness_with_wrong_release_reference_does_not_count`
- [x] Add a test that a witness with a wrong rebuilt output digest is rejected
      (release-verification-tech / Witness with wrong rebuilt output digest is rejected)
  - Evidence: `policy::tests::witness_with_wrong_rebuilt_digest_does_not_count`
- [x] Add a test that a witness with a missing or invalid signature is rejected
      before digest comparison
      (release-verification-tech / Witness with invalid signature is rejected)
  - Evidence: `discovery::tests::rejects_witness_missing_sig_sidecar` verifies
    discovery fails with `MissingSigSidecar` before any digest comparison.
    `release::tests::detached_signature_rejects_*` cover parse-level rejection.
- [x] Add a test that a release attestation with a missing or invalid
      signature is rejected before witness evaluation
      (design decision 4 / release attestation signature verification)
  - Evidence: `discovery::tests::rejects_release_missing_sig_sidecar` verifies
    discovery fails with `Io` error when `.sig` is missing.
- [x] Add a test that the verifier discovers witness files from the
      verification-directory layout alone
      (release-verification-tech / Verifier discovers witness files from the verification directory)
  - Evidence: `discovery::tests::discovers_release_and_witnesses_from_directory`
    verifies full layout discovery from the file system alone.
- [x] Add a test that discovery rejects witness attestations with missing
      `.sig` sidecars before digest comparison
      (design verification strategy / discovery rejects missing `.sig` sidecars before digest comparison)
  - Evidence: `discovery::tests::rejects_witness_missing_sig_sidecar`
- [x] Add a test that technically valid witness material can still fail policy
      evaluation
      (release-verification-tech / Technically valid but policy-insufficient witness set)
  - Evidence: `policy::tests::insufficient_quorum_fails_policy_with_technical_success`
- [x] Add a test that zero witnesses yield technical class `self-proof-valid`
      (release-verification-tech / Self-proof tier remains technically valid without witnesses)
  - Evidence: `policy::tests::zero_witnesses_yield_self_proof_valid`
- [x] Add a test that one matching external witness raises technical class to
      `external-witness-match`
      (release-verification-tech / Matching external witness raises technical class)
  - Evidence: `policy::tests::one_matching_witness_raises_technical_class`
- [x] Add a test that changing `policy.json` does not change attestation
      digests
      (release-verification-social / Policy update does not change technical artifact digests)
  - Evidence: `policy::tests::policy_change_does_not_affect_attestation_digests`
- [x] Add a test that insufficient quorum fails policy while keeping technical
      success
      (release-verification-social / Insufficient quorum fails policy even after technical agreement)
  - Evidence: `policy::tests::insufficient_quorum_fails_policy_with_technical_success`
- [x] Add a test that satisfied quorum promotes final class to
      `quorum-satisfied`
      (release-verification-social / Satisfied quorum promotes final release class)
  - Evidence: `policy::tests::satisfied_quorum_promotes_final_class`
- [x] Add a test that the same witness domain cannot satisfy independence rules
      alone
      (release-verification-social / Same actor cannot satisfy all required witness slots)
  - Evidence: `policy::tests::same_identity_cannot_satisfy_independence`
- [x] Add a test that a revoked witness key or witness attestation digest
      degrades policy status without changing the technical result
      (release-verification-social / Revoked witness no longer satisfies release policy)
  - Evidence: `policy::tests::revoked_key_degrades_policy_without_changing_technical_result`,
    `policy::tests::revoked_attestation_digest_degrades_policy`
- [x] Add a test that revocation input is loaded from `revocations.json` before
      policy evaluation
      (release-verification-social / Revocation input comes from a file-based policy artifact)
  - Evidence: `policy::tests::revocations_applied_before_quorum_counting`
    (revocations are loaded and applied before quorum/independence evaluation)
- [x] Add a test that discovery applies `revocations.json` before quorum
      counting
      (design verification strategy / discovery applies `revocations.json` before quorum counting)
  - Evidence: `discovery::tests::discovery_applies_revocations_before_quorum`
    writes a layout with two witnesses, revokes one via `revocations.json`,
    runs discovery → policy evaluation, and asserts quorum fails.

## Validation

- [x] Run `openspec validate decentralized-release-verification`
- [x] Run a tasks gate after the technical and social specs, design, and task
      wording all stabilize
  - Evidence: `openspec_gate stage=tasks change=decentralized-release-verification`
    returned `Tasks pass. No edits needed.` after the Phase 4 traceability
    expansion.
