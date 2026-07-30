# Tasks

## Phase 1: Shared record authority

- [x] [depends:persist-rust-unit-castore-results] I1 Extend the Rust cache core with bounded signed result envelopes, producer-policy identity, full-key trust matching, and deterministic authority decisions. r[cache_substitution.rust_unit_action_result_discovery]
- [x] [serial] I2 Add positive and negative signature tests for accepted keys, unknown keys, duplicate signer names with different key material, modified records, wrong action references, and wrong result references. r[cache_substitution.rust_unit_action_result_discovery.candidate_admission]

## Phase 2: Discovery and content transfer

- [x] [serial] I3 Add domain-tagged Rust unit lookup to provider-neutral result sources without changing derivation `ActionResultRecord` or PathInfo semantics. r[cache_substitution.rust_unit_action_result_discovery]
- [x] [serial] I4 Add bounded ordered candidate lookup with explicit offline policy, source priority, metadata limits, candidate limits, and redacted source identity. r[cache_substitution.rust_unit_action_result_discovery.offline]
- [x] [serial] I5 Fetch missing referenced blobs and directories through the object-store interface, verify BLAKE3 identities, and run complete local Rust result admission before materialization. r[cache_substitution.rust_unit_action_result_discovery.candidate_admission]
- [x] [parallel] I6 Add negative tests for oversized indexes, malformed references, duplicate-conflicting records, redirects, timeouts, transfer truncation, corrupt objects, missing children, and offline network attempts. r[cache_substitution.rust_unit_action_result_discovery.candidate_admission]

## Phase 3: Atomic publication and conflict handling

- [x] [serial] I7 Publish complete immutable objects first, the signed Rust unit record second, and the no-clobber action index candidate last. r[cache_substitution.rust_unit_action_result_discovery.publication]
- [x] [serial] I8 Preserve multiple result references for one action, deduplicate exact results, and emit a strong-reuse blocker for different admissible artifact sets. r[cache_substitution.rust_unit_action_result_discovery.conflicts]
- [x] [parallel] I9 Add concurrent publication tests that prove readers never observe partial result state and writers never overwrite a different immutable result. r[cache_substitution.rust_unit_action_result_discovery.publication]

## Phase 4: Rust-plan policy and evidence

- [x] [serial] I10 Add explicit remote read and publication policy to Rust topology execution. Keep local output reuse and local castore reuse ahead of remote transfer. r[rust_package_planning.unit_execution.topology.shared_cache_receipts]
- [x] [serial] I11 Extend unit and topology receipts with sanitized source identity, result identity, authority disposition, local or remote route, rejection reason, transferred bytes, reused bytes, and compiler execution. r[rust_package_planning.unit_execution.topology.shared_cache_receipts]
- [x] [parallel] I12 Add a clean-client integration test that restores a signed remote Rust topology and observes zero compiler invocations. r[cache_substitution.rust_unit_action_result_discovery.clean_client]
- [x] [parallel] I13 Add fallback tests for untrusted, incomplete, unavailable, and policy-rejected candidates plus a conflict test that does not choose by source order. r[cache_substitution.rust_unit_action_result_discovery.conflicts]
- [x] [parallel] I14 Update operator, policy, machine-artifact, trust, and non-claim documentation. r[rust_package_planning.unit_execution.topology.shared_cache_receipts]

## Phase 5: Verification and lifecycle evidence

- [x] [serial] V1 Run focused Rust cache core, result-source, object-transfer, store, and `rust-plan` integration tests. Record exact command output in `cairn/changes/share-rust-unit-action-results/evidence/verification.md`. r[rust_package_planning.unit_execution.topology.shared_cache_receipts]
- [x] [serial] V2 Run clean-client, offline, invalid-signature, incomplete-tree, conflict, and atomic-publication rails with both positive and negative cases. r[cache_substitution.rust_unit_action_result_discovery]
- [x] [serial] V3 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`, all three gates for this change, and `tracey coverage`. Record exact output before sync and archive. r[rust_package_planning.unit_execution.topology.shared_cache_receipts]
  - Evidence: repository validation and all three change gates passed. Pre-sync Tracey retained existing repository-wide findings and reported the new identifiers as expected dangling references until spec sync.
