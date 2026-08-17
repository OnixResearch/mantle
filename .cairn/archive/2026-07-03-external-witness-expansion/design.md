## Context

The current Aspen witness proves one external replay under one policy. More witnesses should strengthen the social and technical claim only when they are cryptographically valid, independently attributable, and digest-matched to the same release universe.

## Decisions

### 1. Witness expansion reports are separate from final release verification

**Choice:** Add a witness roster/report layer that summarizes witness state, while final release/global verification remains authoritative for eligibility.

**Rationale:** Operators need visibility into witness coverage without duplicating policy admission logic.

### 2. Independence is policy data, not string decoration

**Choice:** Each witness entry records signer key name, witness identity, independence domain, host class, source acquisition mode, digest match, signature status, and policy decision.

**Rationale:** Same operator/domain witnesses should be useful evidence but must not inflate independent quorum counts.

### 3. Bad or unknown witnesses are counted separately

**Choice:** Unknown-key, bad-signature, wrong-release-digest, revoked, stale-request, and same-domain witnesses remain in the report with explicit skip/fail classes.

**Rationale:** Preserving rejected evidence makes quorum decisions auditable and prevents silent overclaiming.

## Risks / Trade-offs

- More witness metadata increases release verification output size.
- Privacy-sensitive operator details must stay bounded to declared identity/domain/host-class fields, not arbitrary logs.
