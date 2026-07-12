## Phase 1: Publication plan and state machine

- [x] [serial] Define a pure publication plan and state machine for normalized inputs, absent final destination, deterministic artifact layout, assembly phases, verification result, and commit eligibility. r[mantle.release_provenance.bundle_publication.boundary]
- [x] [serial] Bind the plan to release id, planned layout, available input identities, and policy with BLAKE3 while excluding temporary path randomness from evidence identity. r[mantle.release_provenance.bundle_publication.boundary]
- [x] [serial] Add plain assertions for valid transitions, preexisting destinations, phase failures, verification rejection, and commit races. r[mantle.release_provenance.bundle_publication.boundary.test]

## Phase 2: Stage, verify, and commit

- [x] [serial] Create a private capability-scoped sibling stage on the destination filesystem and assemble every planned artifact there. r[mantle.release_provenance.bundle_publication.atomic_commit]
- [x] [serial] Write the canonical manifest last and run production bundle verification against the complete staging root. r[mantle.release_provenance.bundle_publication.staging_validation]
- [x] [serial] Publish with an atomic no-clobber rename and emit CLI success only after commit. r[mantle.release_provenance.bundle_publication.atomic_commit]
- [x] [serial] Reject preexisting or concurrently created files, symlinks, empty directories, and nonempty directories without modifying them. r[mantle.release_provenance.bundle_publication.fixtures.negative.race]

## Phase 3: Failure isolation and retry

- [x] [serial] Confine all pre-commit failures to Mantle-owned staging state and leave the final destination absent or unchanged. r[mantle.release_provenance.bundle_publication.failure_isolation]
- [x] [serial] Add ownership-marker and plan-identity checks for bounded stale-stage cleanup or quarantine. r[mantle.release_provenance.bundle_publication.stale_stage]
- [x] [serial] Make corrected retries create a fresh stage and complete without manual final-path cleanup. r[mantle.release_provenance.bundle_publication.retry]

## Phase 4: Regression fixtures

- [x] [parallel] Add a positive production-path create, verify, and retry fixture. r[mantle.release_provenance.bundle_publication.fixtures.positive] r[mantle.release_provenance.bundle_publication.retry]
- [x] [parallel] Add named test-adapter failures after input copy, artifact hashing, manifest serialization, staged verification, cleanup, and pre-commit phases. r[mantle.release_provenance.bundle_publication.fixtures.negative.verification] r[mantle.release_provenance.bundle_publication.failure_isolation]
- [x] [parallel] Add preexisting destination, destination symlink, concurrent winner, unrecognized stale sibling, and no-clobber commit fixtures. r[mantle.release_provenance.bundle_publication.fixtures.negative.race] r[mantle.release_provenance.bundle_publication.stale_stage]
- [x] [parallel] Observe the public path through every named phase and prove it is absent before commit and complete after commit. r[mantle.release_provenance.bundle_publication.validation.visibility]

## Phase 5: Validation and documentation

- [x] [serial] Document absent-destination semantics, atomic visibility, safe retry/cleanup, no-clobber behavior, and the durability non-claim. r[mantle.release_provenance.bundle_publication.atomic_commit] r[mantle.release_provenance.bundle_publication.failure_isolation]
- [x] [serial] Run focused release evidence unit/integration tests and the full publication failure matrix. r[mantle.release_provenance.bundle_publication.validation]
  - Evidence: `194` no-std core tests, `36` focused root release tests, `13` release-create CLI tests, wasm check, core clippy with warnings denied, and touched-file rustfmt all pass; see `evidence/validation.md`.
- [x] [serial] Run Cairn validation and proposal, design, and tasks gates before sync/archive. r[mantle.release_provenance.bundle_publication.validation]
  - Evidence: Cairn validation and all three gates pass against the active change; final post-checkbox receipts are recorded in `evidence/validation.md`. No sync or archive was run.
