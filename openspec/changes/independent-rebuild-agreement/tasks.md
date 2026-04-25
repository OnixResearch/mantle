# Tasks: independent rebuild agreement

## Phase 1: Agreement core

- [x] I1 Add pure agreement-report types and canonical compact JSON ✅ 2m 9s (started: 2026-04-25T20:02:30Z → completed: 2026-04-25T20:04:39Z)
      serialization over release attestation, witness classifications,
      digest sets, signer names, identities, selected independence-domain
      values, witness evidence classifications, and environment summaries.
      [covers=release.verification.tech.independent.agreement.report,release.verification.tech.witness.independence.evidence]
      Evidence: `cargo test -p crunch-attestation-core independent_agreement`
      passed in pueue task 13 (4 tests), covering canonical report types,
      BLAKE3 report digest, sorted witnesses, witness classifications, counts,
      and required counted independence domain.
- [x] I2 Add canonicalization tests proving discovery order does not change ✅ 2m 9s (started: 2026-04-25T20:02:30Z → completed: 2026-04-25T20:04:39Z)
      report bytes or BLAKE3 report digest.
      [covers=release.verification.tech.independent.agreement.report]
      Evidence: pueue task 13 passed `independent_agreement_report_digest_is_stable_across_witness_order`, proving canonical bytes and report digest are stable across witness discovery order.
- [x] I3 Add skipped/failed witness classification reasons for unknown key, ✅ 2m 9s (started: 2026-04-25T20:02:30Z → completed: 2026-04-25T20:04:39Z)
      invalid signature, revoked witness, digest mismatch, malformed environment
      summary, missing selector evidence, and duplicate independence domain.
      [covers=release.verification.tech.independent.agreement.report,release.verification.tech.witness.independence.evidence]
      Evidence: pueue task 13 passed classification tests for counted, unknown-key, invalid-signature, revoked, digest-mismatch, duplicate-independence-domain, missing-independence-evidence, and counted-domain validation.

## Phase 2: Policy and verifier integration

- [x] I4 Extend verifier-local policy with independent agreement thresholds and ✅ 1m 14s (started: 2026-04-25T20:05:10Z → completed: 2026-04-25T20:06:24Z)
      selectors for `witness_identity`, `signer_key_name`, and
      `rebuild_environment_summary.host_class`, and reject countable witnesses
      whose selected independence field is absent or empty.
      [covers=release.verification.social.independent.agreement.policy,release.verification.tech.witness.independence.evidence]
      Evidence: `cargo test -p crunch-attestation-core` passed in pueue task 16
      (70 tests), including selector coverage for witness identity, signer key,
      host class, duplicate-domain insufficiency, and empty selected
      independence-field rejection.
- [x] I5 Extend release verification output with JSON field ✅ 4m 52s (started: 2026-04-25T20:07:40Z → completed: 2026-04-25T20:12:32Z)
      `independent_agreement_status`, class value
      `independent-rebuild-agreement` when satisfied, JSON fields
      `independent_agreement_report_digest`,
      `independent_agreement_counted_witness_count`,
      `independent_agreement_skipped_witness_count`,
      `independent_agreement_failed_witness_count`, per-witness classification
      reasons, visible digest/signature/independence/policy sufficiency states,
      and unsatisfied domain diagnostics.
      [covers=release.verification.tech.independent.agreement.class,release.verification.tech.witness.independence.evidence]
      Evidence: `cargo test -p crunch --test release_cli
      attest_witness_show_and_release_verify_report_quorum_satisfied` passed in
      pueue task 23, asserting `independent_agreement_status`,
      `independent_agreement_class`, report digest, counted/skipped/failed
      counts, and per-witness `counted` classification remain visible beside
      existing technical/policy/final classes. Focused `release_verify` tests
      also passed in pueue task 19 (24 tests).
- [x] I6 Allow verification directories or release evidence to carry an optional ✅ 5m 22s (started: 2026-04-25T20:40:30Z → completed: 2026-04-25T20:45:52Z)
      agreement report at `agreement-report.json` or bundle-local
      `independent-agreement/agreement-report.json`, reject ambiguous duplicate
      attachment filenames, and verify any present report against
      release/witness material when present.
      [covers=release.evidence.independent.agreement.attachment]
      Evidence: `cargo test -p crunch --test release_cli agreement_attachment`
      passed in pueue task 14 (3 tests), covering matching verification-dir
      `agreement-report.json`, mismatched report digest rejection, and duplicate
      attachment filename rejection. Release-evidence manifests now have optional
      `independent_agreement_report` at
      `independent-agreement/agreement-report.json` with normal artifact digest
      verification while bundles without the field remain valid.

## Phase 3: CLI tests and docs

- [x] I7 Add CLI tests for satisfying independent witnesses, same-domain witness ✅ 10m 52s (started: 2026-04-25T20:46:10Z → completed: 2026-04-25T20:57:02Z)
      rejection, unknown-key skip, invalid-signature witness classification,
      revoked witness skip, malformed environment evidence, missing selector
      evidence, digest-mismatch witness-set rejection, bundle without agreement
      remains basic-valid with absent status, duplicate agreement-report
      attachment rejection, and mismatched agreement attachment rejection.
      [covers=release.verification.tech.independent.agreement.report,release.verification.tech.independent.agreement.class,release.verification.social.independent.agreement.policy,release.evidence.independent.agreement.attachment,release.verification.tech.witness.independence.evidence]
      Evidence: `cargo test -p crunch --test release_cli attest_release_verify`
      passed in pueue task 22 (11 tests) after adding satisfying, same-domain,
      invalid-signature, revoked, unknown-key, digest-mismatch, missing-evidence,
      duplicate-attachment, mismatched-attachment, and no-report/basic-valid
      assertions. Final expanded evidence in pueue task 31 passed 12
      `attest_release_verify` tests including malformed environment evidence;
      `cargo test -p crunch-attestation-core independent_agreement` passed 4
      core report tests in pueue task 32.
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
