## Implementation

- [x] [serial] I1 Add an operator guide covering self-build, Cargo-free fixed-point proof, Nix-free demo-bundle validation, prerequisites, outputs, and cleanup. r[verification_evidence.operator_proof_guide]
  - Evidence: `docs/operator-proof-guide.md` now covers the three proof families, prerequisite categories, output files, and cleanup rules.
- [x] [serial] I2 Document how to interpret success, blocked, failed, and stale-evidence outcomes with explicit claim boundaries. r[verification_evidence.operator_proof_guide]
  - Evidence: `docs/operator-proof-guide.md` includes success/blocked/failed/stale reporting rules and exact non-claim vocabulary.
- [x] [serial] I3 Link guide wording to CLI help or README entries so operators can discover the workflow. r[verification_evidence.operator_proof_guide]
  - Evidence: `README.md` links `docs/operator-proof-guide.md` and the guide drift command near the validation/evidence docs.
- [x] [serial] I4 Add a drift check or docs test for guide command snippets, output paths, and proof-bundle field names. r[verification_evidence.operator_proof_guide]
  - Evidence: `scripts/check-operator-proof-guide.rs` validates snippets, output paths, demo summary fields, outcomes, and non-claims.

## Verification

- [x] [serial] V1 Positive: run the docs/guide check against current command snippets and proof-bundle fields. r[verification_evidence.operator_proof_guide]
  - Evidence: pueue task 105, `nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs` -> `operator proof guide drift check passed`.
- [x] [serial] V2 Negative: fixture a stale command, missing bundle field, or overbroad proof claim and assert the guide check rejects it. r[verification_evidence.operator_proof_guide]
  - Evidence: pueue task 100, `nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs --self-test` -> `operator proof guide checker self-test passed`.
- [x] [serial] V3 Positive: inspect the guide and verify it names prerequisites, commands, success criteria, blockers, evidence paths, replay hints, and non-claims. r[verification_evidence.operator_proof_guide]
  - Evidence: manual inspection of `docs/operator-proof-guide.md`; focused validator tests in pueue task 106 passed `4 passed; 0 failed` for `nix_free_demo_bundle`.
- [x] [serial] V4 Run docs checks, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[verification_evidence.operator_proof_guide]
  - Evidence: pueue task 107 `git diff --check` passed; task 108 `cairn validate` returned `valid: true`; task 109 Cairn proposal/design/tasks gates returned PASS.
