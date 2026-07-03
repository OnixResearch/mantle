## Implementation

- [ ] [serial] I1 Re-run the documented expensive proof command or a deliberately scoped proof-refresh command on the current tree. r[verification_evidence.expensive_proof_evidence_refresh]
- [ ] [serial] I2 Record command, environment highlights, output bundle paths, final verdict, relevant BLAKE3 digests, and blocker details when present. r[verification_evidence.expensive_proof_evidence_refresh]
- [ ] [serial] I3 Update proof docs/status wording so current evidence is cited and stale proof claims are removed or narrowed. r[verification_evidence.expensive_proof_evidence_refresh]
- [ ] [serial] I4 If the proof blocks on an implementation-owned issue, create or link the follow-on Cairn change instead of claiming proof success. r[verification_evidence.expensive_proof_evidence_refresh]

## Verification

- [ ] [serial] V1 Positive: inspect the refreshed proof bundle or blocker bundle and assert the evidence summary matches the recorded verdict and digests. r[verification_evidence.expensive_proof_evidence_refresh]
- [ ] [serial] V2 Negative: verify missing, stale, or mismatched proof-bundle paths are reported as not-current evidence. r[verification_evidence.expensive_proof_evidence_refresh]
- [ ] [serial] V3 Negative: verify blocked proof evidence does not produce Nix-free fixed-point success wording. r[verification_evidence.expensive_proof_evidence_refresh]
- [ ] [serial] V4 Run relevant proof/docs checks, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[verification_evidence.expensive_proof_evidence_refresh]
