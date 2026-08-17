## Evidence production

- [x] [serial] I1 Create a fresh release evidence bundle whose current-code provider fixed-point proof validates and reports a matched release artifact path/digest. r[verification_evidence.provider_bound_release_evidence_refresh_transcripts]
  - Evidence: `evidence/provider-bound-release-evidence-2026-06-26.md` records release id `provider-bound-release-evidence-2026-06-26`, provider status `valid`, matched artifact `binaries/01-mantle`, and BLAKE3 `b4fdeca80db000a4e003513417b665ae2a1014f6752ed9ff65bb3b6b57ce48f3`.
- [x] [serial] I2 Run deterministic release proof over the complete packaged binary set and verify with both deterministic-release and provider fixed-point gates. r[verification_evidence.provider_bound_release_evidence_refresh_transcripts]
  - Evidence: `evidence/provider-bound-release-evidence-2026-06-26.md` records pueue tasks 92 and 93 with deterministic-release status `eligible` and provider proof status `valid`.
- [x] [serial] I3 Run portable replay from copied artifacts plus a negative missing-proof replay. r[verification_evidence.provider_bound_release_evidence_refresh_transcripts]
  - Evidence: `evidence/provider-bound-release-evidence-2026-06-26.md` records pueue task 104 with copied-bundle positive replay and missing-proof negative replay exiting 3.
- [x] [serial] I4 Record generated artifact paths, key BLAKE3 digests, verifier status, and bounded non-claims in tracked Cairn evidence. r[verification_evidence.provider_bound_release_evidence_refresh_transcripts]
  - Evidence: `evidence/provider-bound-release-evidence-2026-06-26.md` records generated paths, proof/sandbox/provider digests, verifier status, and non-claims.

## Verification

- [x] [serial] V1 Run receipt checker and required release verifier on the generated bundle. r[verification_evidence.provider_bound_release_evidence_refresh_transcripts]
  - Evidence: `evidence/provider-bound-release-evidence-2026-06-26.md` records pueue tasks 93 and 99, including required verifier success and `real release determinism proof receipt valid`.
- [x] [serial] V2 Run `git diff --check`, Cairn validation, and proposal/design/tasks gates before archive. r[verification_evidence.provider_bound_release_evidence_refresh_transcripts]
  - Evidence: `evidence/final-validation-2026-06-26.md` records `git diff --check`, Cairn validation, and proposal/design/tasks gates all passing.
