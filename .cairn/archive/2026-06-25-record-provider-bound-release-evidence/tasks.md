## Evidence production

- [x] [serial] I1 Create a fresh release evidence bundle whose provider fixed-point proof validates and reports a matched release artifact path/digest. r[verification_evidence.provider_bound_release_evidence_transcripts]
  - Evidence: `evidence/provider-bound-release-evidence-2026-06-25.md` records release id `provider-bound-release-evidence-2026-06-25`, provider status `valid`, matched artifact `binaries/01-mantle`, and BLAKE3 `288023366ab86aafdc2384a113d208a295693b145567d757af56bfe6dafa9ec3`.
- [x] [serial] I2 Run deterministic release proof over the complete packaged binary set and verify with both deterministic-release and provider fixed-point gates. r[verification_evidence.provider_bound_release_evidence_transcripts]
  - Evidence: pueue tasks 220/224 in `evidence/provider-bound-release-evidence-2026-06-25.md` record required verifier status `eligible` and proof verdict `self-rebuild-match`.
- [x] [serial] I3 Run portable replay from copied artifacts plus a negative missing-proof replay. r[verification_evidence.provider_bound_release_evidence_transcripts]
  - Evidence: pueue task 213 in `evidence/provider-bound-release-evidence-2026-06-25.md` records copied-bundle positive replay and missing-proof negative replay exiting 3.
- [x] [serial] I4 Record generated artifact paths, key BLAKE3 digests, verifier status, and bounded non-claims in tracked Cairn evidence. r[verification_evidence.provider_bound_release_evidence_transcripts]
  - Evidence: `evidence/provider-bound-release-evidence-2026-06-25.md` records generated paths, proof/sandbox/provider digests, current verifier status, non-claims, and the next provider fixed-point blocker.

## Verification

- [x] [serial] V1 Run receipt checker and required release verifier on the generated bundle. r[verification_evidence.provider_bound_release_evidence_transcripts]
  - Evidence: pueue tasks 220 and 222 in `evidence/provider-bound-release-evidence-2026-06-25.md` record the required verifier output slice and receipt-checker `real release determinism proof receipt valid` output.
- [x] [serial] V2 Run Cairn validation and proposal/design/tasks gates before archive. r[verification_evidence.provider_bound_release_evidence_transcripts]
  - Evidence: `evidence/final-validation-2026-06-25.md` records `git diff --check`, `cairn validate`, and proposal/design/tasks gates all exiting 0.
