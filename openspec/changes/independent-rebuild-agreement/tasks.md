# Tasks: independent rebuild agreement

## Phase 1: Agreement core

- [ ] I1 Add pure agreement-report types and canonical compact JSON
      serialization over release attestation, witness classifications,
      digest sets, signer names, identities, selected independence-domain
      values, witness evidence classifications, and environment summaries.
      [covers=release.verification.tech.independent.agreement.report,release.verification.tech.witness.independence.evidence]
- [ ] I2 Add canonicalization tests proving discovery order does not change
      report bytes or BLAKE3 report digest.
      [covers=release.verification.tech.independent.agreement.report]
- [ ] I3 Add skipped/failed witness classification reasons for unknown key,
      invalid signature, revoked witness, digest mismatch, malformed environment
      summary, missing selector evidence, and duplicate independence domain.
      [covers=release.verification.tech.independent.agreement.report,release.verification.tech.witness.independence.evidence]

## Phase 2: Policy and verifier integration

- [ ] I4 Extend verifier-local policy with independent agreement thresholds and
      selectors for `witness_identity`, `signer_key_name`, and
      `rebuild_environment_summary.host_class`, and reject countable witnesses
      whose selected independence field is absent or empty.
      [covers=release.verification.social.independent.agreement.policy,release.verification.tech.witness.independence.evidence]
- [ ] I5 Extend release verification output with JSON field
      `independent_agreement_status`, class value
      `independent-rebuild-agreement` when satisfied, JSON fields
      `independent_agreement_report_digest`,
      `independent_agreement_counted_witness_count`,
      `independent_agreement_skipped_witness_count`,
      `independent_agreement_failed_witness_count`, per-witness classification
      reasons, visible digest/signature/independence/policy sufficiency states,
      and unsatisfied domain diagnostics.
      [covers=release.verification.tech.independent.agreement.class,release.verification.tech.witness.independence.evidence]
- [ ] I6 Allow verification directories or release evidence to carry an optional
      agreement report at `agreement-report.json` or bundle-local
      `independent-agreement/agreement-report.json`, reject ambiguous duplicate
      attachment filenames, and verify any present report against
      release/witness material when present.
      [covers=release.evidence.independent.agreement.attachment]

## Phase 3: CLI tests and docs

- [ ] I7 Add CLI tests for satisfying independent witnesses, same-domain witness
      rejection, unknown-key skip, invalid-signature witness classification,
      revoked witness skip, malformed environment evidence, missing selector
      evidence, digest-mismatch witness-set rejection, bundle without agreement
      remains basic-valid with absent status, duplicate agreement-report
      attachment rejection, and mismatched agreement attachment rejection.
      [covers=release.verification.tech.independent.agreement.report,release.verification.tech.independent.agreement.class,release.verification.social.independent.agreement.policy,release.evidence.independent.agreement.attachment,release.verification.tech.witness.independence.evidence]
- [ ] I8 Update release verification docs to describe independent agreement as
      policy-scoped evidence from accepted witness files and verifier-local
      policy, name `independent_agreement_status`,
      `independent_agreement_class`, report digest, counted/skipped/failed
      counts, and classification reasons, and avoid claiming full-source
      bootstrap, global reproducibility, or public witness discovery.
      [covers=release.verification.tech.independent.agreement.class,release.verification.tech.independent.agreement.docs]

## Validation

- [ ] V1 Run `openspec validate independent-rebuild-agreement --strict` and
      record the result. [covers=release.verification.tech.independent.agreement.report,release.verification.tech.independent.agreement.class,release.verification.social.independent.agreement.policy,release.evidence.independent.agreement.attachment,release.verification.tech.witness.independence.evidence,release.verification.tech.independent.agreement.docs]
- [ ] V2 Run agreement core canonicalization and policy selector unit tests.
      [covers=release.verification.tech.independent.agreement.report,release.verification.social.independent.agreement.policy,release.verification.tech.witness.independence.evidence]
- [ ] V3 Run release CLI agreement tests and record positive plus negative
      witness-set outcomes, including digest-mismatch witness sets,
      no-agreement basic-valid bundles, duplicate agreement attachments,
      mismatched agreement attachments, and invalid-signature witness
      classification while digest matching, signature validity, independence,
      and policy sufficiency remain separately visible.
      [covers=release.verification.tech.independent.agreement.report,release.verification.tech.independent.agreement.class,release.evidence.independent.agreement.attachment,release.verification.tech.witness.independence.evidence]
- [ ] V4 Run docs bounded-claim checks proving independent agreement docs name
      required JSON fields/classifications and do not claim full-source
      bootstrap, global reproducibility, or public witness discovery.
      [covers=release.verification.tech.independent.agreement.docs]
