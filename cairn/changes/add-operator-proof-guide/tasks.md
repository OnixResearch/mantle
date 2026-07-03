## Implementation

- [ ] [serial] I1 Add an operator guide covering self-build, Cargo-free fixed-point proof, Nix-free demo-bundle validation, prerequisites, outputs, and cleanup. r[verification_evidence.operator_proof_guide]
- [ ] [serial] I2 Document how to interpret success, blocked, failed, and stale-evidence outcomes with explicit claim boundaries. r[verification_evidence.operator_proof_guide]
- [ ] [serial] I3 Link guide wording to CLI help or README entries so operators can discover the workflow. r[verification_evidence.operator_proof_guide]
- [ ] [serial] I4 Add a drift check or docs test for guide command snippets, output paths, and proof-bundle field names. r[verification_evidence.operator_proof_guide]

## Verification

- [ ] [serial] V1 Positive: run the docs/guide check against current command snippets and proof-bundle fields. r[verification_evidence.operator_proof_guide]
- [ ] [serial] V2 Negative: fixture a stale command, missing bundle field, or overbroad proof claim and assert the guide check rejects it. r[verification_evidence.operator_proof_guide]
- [ ] [serial] V3 Positive: inspect the guide and verify it names prerequisites, commands, success criteria, blockers, evidence paths, replay hints, and non-claims. r[verification_evidence.operator_proof_guide]
- [ ] [serial] V4 Run docs checks, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[verification_evidence.operator_proof_guide]
