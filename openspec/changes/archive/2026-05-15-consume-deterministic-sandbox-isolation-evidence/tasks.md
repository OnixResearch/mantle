## Phase 1: Spec

- [x] [serial] Update release-verification spec for verifier deterministic artifact consumption.

## Phase 2: Implementation

- [x] [serial] Add `release verify` CLI inputs for deterministic proof, isolation evidence, and require-deterministic promotion.
- [x] [serial] Load canonical deterministic artifacts, evaluate eligibility, and report paths/digests/blockers.
- [x] [serial] Add CLI tests for valid, missing, and malformed/stale isolation evidence behavior.

## Phase 3: Verification

- [x] [serial] Run targeted release CLI tests, OpenSpec validation, formatting, and diff checks.
