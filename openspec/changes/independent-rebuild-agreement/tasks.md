# Tasks: independent rebuild agreement

## Phase 1: Agreement core

- [ ] I1 Add pure agreement-report types and canonical compact JSON
      serialization over release attestation, witness classifications,
      digest sets, signer names, identities, and environment summaries.
      [covers=release.verification.tech.independent.agreement.report]
- [ ] I2 Add canonicalization tests proving discovery order does not change
      report bytes or BLAKE3 report digest.
      [covers=release.verification.tech.independent.agreement.report]
- [ ] I3 Add skipped/failed witness classification reasons for unknown key,
      invalid signature, revoked witness, digest mismatch, and duplicate
      independence domain. [covers=release.verification.tech.independent.agreement.report]

## Phase 2: Policy and verifier integration

- [ ] I4 Extend verifier-local policy with independent agreement thresholds and
      selectors for `witness_identity`, `signer_key_name`, and
      `rebuild_environment_summary.host_class`.
      [covers=release.verification.social.independent.agreement.policy]
- [ ] I5 Extend release verification output with independent agreement status,
      report digest, counted/skipped/failed witness counts, and unsatisfied
      domain diagnostics. [covers=release.verification.tech.independent.agreement.class]
- [ ] I6 Allow verification directories or release evidence to carry an optional
      agreement report and verify it against release/witness material when
      present. [covers=release.evidence.independent.agreement.attachment]

## Phase 3: CLI tests and docs

- [ ] I7 Add CLI tests for satisfying independent witnesses, same-domain witness
      rejection, unknown-key skip, invalid-signature witness classification,
      revoked witness skip, digest-mismatch witness-set rejection, and
      mismatched agreement attachment rejection.
      [covers=release.verification.tech.independent.agreement.report,release.verification.tech.independent.agreement.class,release.verification.social.independent.agreement.policy,release.evidence.independent.agreement.attachment]
- [ ] I8 Update release verification docs to describe independent agreement as
      policy-scoped evidence, not proof of full-source bootstrap or global
      reproducibility. [covers=release.verification.tech.independent.agreement.class]

## Validation

- [ ] V1 Run `openspec validate independent-rebuild-agreement --strict` and
      record the result. [covers=release.verification.tech.independent.agreement.report,release.verification.tech.independent.agreement.class,release.verification.social.independent.agreement.policy,release.evidence.independent.agreement.attachment]
- [ ] V2 Run agreement core canonicalization and policy selector unit tests.
      [covers=release.verification.tech.independent.agreement.report,release.verification.social.independent.agreement.policy]
- [ ] V3 Run release CLI agreement tests and record positive plus negative
      witness-set outcomes, including digest-mismatch witness sets and
      invalid-signature witness classification while digest matching, signature
      validity, independence, and policy sufficiency remain separately visible.
      [covers=release.verification.tech.independent.agreement.report,release.verification.tech.independent.agreement.class,release.evidence.independent.agreement.attachment]
