# Tasks

- [x] [serial] I1 Write a tracked evidence transcript for `provider-fixed-point-release-evidence-2026-06-25` that captures the deterministic release proof command, required provider/deterministic verifier command, receipt checker command, and key BLAKE3 digests. r[verification_evidence.release_reproducibility_transcripts]
- [x] [serial] I2 Keep generated release bundle, deterministic proof, rebuild, and summary artifacts under ignored `target/` paths and stage only lifecycle evidence text. r[verification_evidence.release_reproducibility_transcripts]
- [x] [serial] V1 Re-run or inspect same-turn evidence for `mantle release reproduce`, `mantle --json release verify --require-deterministic-release --require-provider-fixed-point-proof`, and `scripts/check-real-release-determinism-receipt.rs`. r[verification_evidence.release_reproducibility_transcripts]
- [x] [serial] V2 Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and record exact output plus tracked status before committing. r[verification_evidence.release_reproducibility_transcripts]
