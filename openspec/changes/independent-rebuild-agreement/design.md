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
witness sidecars, trusted keys, revocations, and policy.

**Rationale:** hand-authored agreement JSON would be another artifact to trust.
Derived reports keep the source of truth in signed attestations plus local
policy.

**Alternative:** ask publishers to write an agreement file.

**Why not:** that lets publishers overstate which witnesses counted.

### 2. Counted/skipped/failed witnesses all appear in output

**Choice:** report every discovered witness with classification.

**Rationale:** operators need to diagnose why quorum failed, especially when
some witnesses are unknown-key, revoked, duplicate-domain, or mismatched.

### 3. Reuse existing independence selectors first

**Choice:** start with `witness_identity`, `signer_key_name`, and
`rebuild_environment_summary.host_class`.

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
- `openspec validate independent-rebuild-agreement --strict`.
