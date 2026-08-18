# Tasks

## Contract

- [x] [serial] Specify that provider-bound witness rebuild may use only witness-produced provider fixed-point proof candidates and must still fail closed for invalid or missing candidates. r[verification_evidence.release_witness_rebuild_multi_output]
  - Evidence: `evidence/provider-bound-witness-replay-validation.md` records the spec delta and bounded non-claim.

## Implementation

- [x] [serial] Export witness-owned provider proof and extracted repo paths to the workflow driver. r[verification_evidence.release_witness_rebuild_multi_output]
  - Evidence: `src/witness_rebuild.rs` exports `CRUNCH_WITNESS_REBUILD_REPO_DIR` and `CRUNCH_WITNESS_PROVIDER_FIXED_POINT_PROOF_BUNDLE_DIR`; validation transcript task 75/task 79 covers behavior.
- [x] [serial] Extend rebuilt output candidate collection to validate workflow-produced provider fixed-point proof bundles and include contained stage binaries by digest. r[verification_evidence.release_witness_rebuild_multi_output]
  - Evidence: `collect_rebuilt_output_paths_uses_provider_fixed_point_candidate_by_digest`, invalid-proof, and path-escape tests passed in `evidence/provider-bound-witness-replay-validation.md`.
- [x] [serial] Add helper workflow support for running provider fixed-point proof when operators provide source-built provider inputs. r[verification_evidence.release_witness_rebuild_multi_output]
  - Evidence: `scripts/rebuild-witness-request.sh` generates a provider-bound driver wrapper from explicit provider envs; helper positive/negative tests and retained-request preflight are recorded in `evidence/provider-bound-witness-replay-validation.md`.

## Verification

- [x] [serial] Add positive tests for matching provider fixed-point proof candidates. r[verification_evidence.release_witness_rebuild_multi_output]
  - Evidence: `collect_rebuilt_output_paths_uses_provider_fixed_point_candidate_by_digest` passed in `evidence/provider-bound-witness-replay-validation.md`.
- [x] [serial] Add negative tests for invalid provider proof and provider stage path escape. r[verification_evidence.release_witness_rebuild_multi_output]
  - Evidence: `collect_rebuilt_output_paths_rejects_invalid_provider_fixed_point_proof`, `collect_rebuilt_output_paths_rejects_provider_fixed_point_stage_escape`, and helper missing-env tests passed.
- [x] [serial] Run focused witness rebuild tests, affected release CLI tests, build, diff check, Cairn validation, and record the remaining independent-rebuild blocker if the full helper still cannot complete. r[verification_evidence.release_witness_rebuild_multi_output]
  - Evidence: `evidence/provider-bound-witness-replay-validation.md` records passing focused tests/build/diff/Cairn validation plus the retained `gmp-6.2.1` network blocker.
