## Implementation

- [x] [serial] I1 Add a pure proof-mode hermeticity audit classifier with closed-by-default handling for typed event classes. r[verification_evidence.proof_mode_hermeticity_audit_gate]
- [x] [serial] I2 Thread classifier results into release, deterministic-release, self-hosting, and witness proof admission paths. r[verification_evidence.proof_mode_hermeticity_audit_gate]
- [x] [serial] I3 Render allowed downgrade policy basis, blocked event classes, and narrower proof claims in human/JSON evidence. r[verification_evidence.proof_mode_hermeticity_audit_gate]

## Verification

- [x] [serial] V1 Positive: admit a strict proof fixture with no hermeticity audit events and stable report fields. r[verification_evidence.proof_mode_hermeticity_audit_gate]
- [x] [serial] V2 Negative: introduce unknown, degraded, and unapproved audit event classes and prove proof admission fails closed. r[verification_evidence.proof_mode_hermeticity_audit_gate]
- [x] [serial] V3 Positive downgrade: approve a narrow policy exception and prove the report names the downgrade and does not satisfy stricter claims. r[verification_evidence.proof_mode_hermeticity_audit_gate]
- [x] [serial] V4 Run focused audit-classifier tests plus Cairn validate and proposal/design/tasks gates for this change. r[verification_evidence.proof_mode_hermeticity_audit_gate]
