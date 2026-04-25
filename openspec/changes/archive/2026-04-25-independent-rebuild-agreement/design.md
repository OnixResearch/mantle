## Context

The release verification stack already separates technical validity from social
policy and can process witness attestations. The missing piece is a first-class
"independent rebuild agreement" object and output class that says which
witnesses counted, why they counted, and which independence policy they
satisfied.

## Goals / Non-Goals

**Goals**
- Canonicalize an agreement report over release and witness material.
- Make witness independence evidence explicit in verifier output.
- Let policy define the required count and independence selector.
- Keep unknown or invalid witnesses explainable instead of fatal when enough
  trusted witnesses remain.

**Non-Goals**
- Hosted witness discovery or transparency logs.
- New signature algorithms.
- Full-source bootstrap or byte-for-byte artifact reproduction.

## Decisions

### 1. Agreement report is derived, not hand-authored

**Choice:** the verifier derives the agreement report from release attestation,
witness sidecars, trusted keys, revocations, and policy. Optional attachments
are cache/check artifacts only: release bundles discover
`independent-agreement/agreement-report.json`, verification directories discover
`agreement-report.json`, and discovery rejects any second agreement-report file
for the same release. A present attachment must match the derived report's
release-attestation digest, witness identities/signers, rebuilt digest sets,
classification results, and BLAKE3 report digest.

**Rationale:** hand-authored agreement JSON would be another artifact to trust.
Derived reports keep the source of truth in signed attestations plus local
policy.

**Canonical report schema:** compact JSON contains `schema`, `release_attestation_digest`, `policy_digest`, `independence_selector`, `required_witness_count`, `counted_witness_count`, `skipped_witness_count`, `failed_witness_count`, `witnesses`, and `artifact_digest_sets`. Witness entries include `witness_identity`, `signer_key_name`, `signature_status`, `digest_match`, `independence_domain`, `policy_counted`, `classification_reason`, `rebuilt_output_digests`, and `environment_summary`. Witness entries sort by `(witness_identity, signer_key_name, independence_domain, canonical witness digest)`; digest tuples sort by `(name, algorithm, digest)`. The report digest is BLAKE3 over those canonical compact JSON bytes.

**Alternative:** ask publishers to write an agreement file.

**Why not:** that lets publishers overstate which witnesses counted.

### 2. Counted/skipped/failed witnesses all appear in output

**Compatibility with existing witness validation:** wrong release-attestation
references and wrong rebuilt output digest sets remain fatal for signature-valid
witnesses because they contradict the release being verified. Missing, unknown,
or cryptographically invalid witness signatures are no longer fatal by
themselves for agreement reporting; they are classified and skipped/failed so
other trusted witnesses can still satisfy policy.

**Choice:** report every discovered witness with classification. Classification
inputs are trusted signer key name from detached-signature verification,
`witness_identity`, `rebuild_environment_summary.host_class`, rebuilt output
digests, revocation status, and the active policy selector. Reason values are
stable strings: `counted`, `unknown-key`, `invalid-signature`, `revoked`,
`digest-mismatch`, `duplicate-independence-domain`, `missing-independence-evidence`,
and `malformed-environment-evidence`.

**JSON output contract:** `crunch attest release-verify --json` adds
`independent_agreement_status` (`satisfied` or `unsatisfied`), optional
`independent_agreement_class` with value `independent-rebuild-agreement` only
when satisfied, `independent_agreement_report_digest`,
`independent_agreement_counted_witness_count`,
`independent_agreement_skipped_witness_count`,
`independent_agreement_failed_witness_count`, and per-witness classification
records with digest-match, signature-valid, independence-domain, policy-counted,
and reason fields.

**Rationale:** operators need to diagnose why quorum failed, especially when
some witnesses are unknown-key, revoked, duplicate-domain, or mismatched.

### 3. Reuse existing independence selectors first

**Choice:** start with `witness_identity`, `signer_key_name`, and
`rebuild_environment_summary.host_class`. Existing policy fields remain the
source of truth: `min_matching_witnesses` is the required agreement count and
`independence_field` is the selector. Agreement logic reuses the existing
selector implementation and adds stricter output: witnesses with absent or empty
selected domains are skipped with `missing-independence-evidence`; witnesses
that duplicate an already counted domain remain technically matching but are not
policy-counted.

**Rationale:** these are already meaningful in the current attestation model and
avoid inventing organization identity before the project has that governance.

## Implementation Sketch

1. Add a pure agreement-report builder in `crunch-attestation-core`.
2. Extend verifier output with agreement status, report digest, and per-witness
   classification.
3. Extend policy parsing/tests for agreement thresholds.
4. Add CLI tests for satisfying, duplicate-domain, unknown-key, revoked, and
   digest-mismatch witness sets.
5. Document the bounded claim: independent agreement under configured policy.

## Risks / Trade-offs

**Independence metadata can be self-asserted.** Policy can only evaluate the
fields it is given. Docs must call out that stronger identity vetting is social
process outside first-phase file verification.

**Many statuses can confuse users.** Keep JSON complete, but human output should
summarize count, required count, and failing domains.

## Validation Plan

- Canonical report stability tests.
- Policy unit tests for each selector.
- CLI integration tests for agreement satisfied and unsatisfied cases.
- Attachment tests for bundle-local `independent-agreement/agreement-report.json`,
  verification-dir `agreement-report.json`, duplicate attachment rejection, and
  digest/linkage mismatch rejection.
- Docs checks proving independent agreement is policy-scoped and does not claim
  full-source bootstrap or global reproducibility.
- `openspec validate independent-rebuild-agreement --strict`.
