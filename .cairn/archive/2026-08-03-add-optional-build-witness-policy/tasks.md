# Tasks: Add optional build-witness policy

## Phase 1: Baseline and contracts

- [x] [serial] V0 Run the current `crunch-attestation-core` policy tests and focused `release_cli` policy and witness tests before core changes. Record exact output in `cairn/changes/add-optional-build-witness-policy/evidence/baseline.md`. r[verification_evidence.optional_build_witness_policy]
- [x] [serial] I1 Define named optional, compatibility, and quorum profile inputs with explicit count and selector bounds. r[verification_evidence.optional_build_witness_policy]
- [x] [serial] I2 Specify stable quorum statuses `not-required`, `satisfied`, and `insufficient` in human and JSON output. r[verification_evidence.optional_build_witness_policy]

## Phase 2: Pure policy core and CLI shell

- [x] [serial] I3 Add pure policy construction for `optional-witness` with a zero minimum and optional trusted witness identities. r[verification_evidence.optional_build_witness_policy]
- [x] [serial] I4 Add pure policy construction for opt-in `witness-quorum` with an explicit positive minimum and one supported independence selector. r[verification_evidence.optional_build_witness_policy]
- [x] [serial] I5 Extend the CLI shell with the two profiles while preserving `self-proof-only`, `single-witness`, current policy bytes, atomic writes, and no-key-generation behavior. r[verification_evidence.optional_build_witness_policy]
- [x] [serial] I6 Extend witness summaries so optional valid, skipped, failed, duplicate, stale, revoked, and digest-mismatched evidence remains visible without implicit quorum admission. r[verification_evidence.optional_build_witness_policy]
- [x] [serial] I7 Keep `--require-stagex-no-quorum`, bootstrap parity, and full-source fixed-point checks independent from witness policy selection. r[verification_evidence.optional_build_witness_policy]

## Phase 3: Positive and negative verification

- [x] [parallel] V1 Add positive tests for optional mode with zero, one, and multiple valid witnesses and for explicit quorum success under each currently supported independence selector. r[verification_evidence.optional_build_witness_policy]
- [x] [parallel] V2 Add compatibility tests for existing `self-proof-only`, `single-witness`, and hand-authored `mantle-release-policy-v1` files. r[verification_evidence.optional_build_witness_policy]
- [x] [parallel] V3 Add negative tests for a missing quorum minimum, zero quorum minimum, unsupported selector, count overflow, too few trusted identities, duplicate identities, unknown keys, bad signatures, revocations, stale requests, and wrong release digests. r[verification_evidence.optional_build_witness_policy]
- [x] [parallel] V4 Prove optional mode never counts invalid evidence and never changes bootstrap-parity or StageX no-quorum status. Prove explicit quorum insufficiency blocks only the selected quorum claim. r[verification_evidence.optional_build_witness_policy]

## Phase 4: Documentation and lifecycle

- [x] [serial] I8 Update release, witness, StageX, and operator documentation with exact commands, policy-state wording, migration guidance, and bounded non-claims. r[verification_evidence.optional_build_witness_policy]
- [x] [serial] V5 Run `nix develop -c cargo test -p crunch-attestation-core`, `nix develop -c cargo test -p mantle --bin mantle release_attestation::`, and `nix develop -c cargo test -p mantle --test release_cli attest_policy -- --nocapture`. Record exact output in the change evidence. r[verification_evidence.optional_build_witness_policy]
- [x] [serial] V6 Run `nix develop -c cargo fmt --check -p mantle -v`, focused first-party Clippy, machine-contract checks when output changes, `git diff --check`, Cairn validation, traceability coverage, all three change gates, and the relevant Nix checks. r[verification_evidence.optional_build_witness_policy]
