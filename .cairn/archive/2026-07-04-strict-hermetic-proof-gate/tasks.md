## Implementation

- [x] [serial] I1 Add a pure proof-eligibility classifier over hermeticity mode, typed audit events, closure status, host-tool status, and workflow kind. r[verification_evidence.strict_hermetic_proof_gate]
- [x] [serial] I2 Wire release, deterministic-release, self-hosting, and witness shells to require strict hermetic eligibility before admitting proof evidence. r[verification_evidence.strict_hermetic_proof_gate]
- [x] [serial] I3 Update human/JSON reports and operator docs to distinguish strict proof admission from practical or impure diagnostic evidence. r[verification_evidence.strict_hermetic_proof_gate]

## Verification

- [x] [serial] V1 Positive: prove a strict hermetic fixture with no degraded audit events is admitted for the selected proof workflow. r[verification_evidence.strict_hermetic_proof_gate]
- [x] [serial] V2 Negative: prove practical mode, impure mode, missing closure facts, protected-env leakage, and undeclared-host-tool observations are rejected as proof evidence. r[verification_evidence.strict_hermetic_proof_gate]
- [x] [serial] V3 Run focused proof-eligibility tests plus Cairn validate and proposal/design/tasks gates for this change. r[verification_evidence.strict_hermetic_proof_gate]
