# Shared Rust unit action-result verification

Date: 2026-07-30

This transcript records focused positive and negative rails. It does not claim full workspace correctness, compiler correctness, universal reproducibility, remote executor correctness, or release eligibility.

## Core authority tests

```text
$ nix develop -c cargo test -p crunch-rust-cache-core
git: 'remote-https' is not a git command. See 'git --help'.
fatal: remote helper 'https' aborted session
warning: could not get HEAD ref for repository 'https://github.com/trailofbits/dylint'; using expired cached ref 'refs/heads/master'
git: 'remote-https' is not a git command. See 'git --help'.
fatal: remote helper 'https' aborted session
warning: could not get HEAD ref for repository 'https://github.com/tvlfyi/wu-manber.git'; using expired cached ref 'refs/heads/master'
git: 'remote-https' is not a git command. See 'git --help'.
fatal: remote helper 'https' aborted session
warning: could not get HEAD ref for repository 'https://github.com/OnixResearch/nickel-export'; using expired cached ref 'refs/heads/main'
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_rust_cache_core-bc4aef194d0732a3)

running 17 tests
test tests::policy_rejects_invalid_bounds ... ok
test tests::action_rejects_unclassified_absolute_path_and_bad_digest ... ok
test tests::action_identity_is_canonical_and_excludes_output_root ... ok
test tests::local_reuse_admits_one_complete_verified_result ... ok
test tests::result_rejects_path_escape_and_tampered_reference ... ok
test tests::result_and_index_are_canonical ... ok
test tests::local_reuse_reports_conflicting_artifact_sets ... ok
test tests::local_reuse_rejects_incomplete_and_mismatched_candidates ... ok
test shared::tests::untrusted_producer_policy_is_rejected ... ok
test shared::tests::unknown_key_is_rejected ... ok
test shared::tests::modified_record_is_rejected_before_signature_authority ... ok
test shared::tests::wrong_expected_result_reference_is_rejected ... ok
test shared::tests::wrong_expected_action_reference_is_rejected ... ok
test shared::tests::duplicate_signer_name_does_not_replace_full_key_match ... ok
test tests::every_declared_action_input_invalidates_identity ... ok
test shared::tests::accepted_full_key_and_policy_admit_envelope ... ok
test shared::tests::modified_signature_is_rejected ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/architecture.rs (/home/brittonr/.cargo-target/debug/deps/architecture-4817a7dc14914c36)

running 1 test
test rust_cache_core_has_no_filesystem_process_store_or_network_dependencies ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests crunch_rust_cache_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

exit_status=0
```

## Shared source and object-transfer tests

```text
$ nix develop -c cargo test -p crunch-rust-cache shared::tests:: -- --nocapture
git: 'remote-https' is not a git command. See 'git --help'.
fatal: remote helper 'https' aborted session
warning: could not get HEAD ref for repository 'https://github.com/trailofbits/dylint'; using expired cached ref 'refs/heads/master'
git: 'remote-https' is not a git command. See 'git --help'.
fatal: remote helper 'https' aborted session
warning: could not get HEAD ref for repository 'https://github.com/tvlfyi/wu-manber.git'; using expired cached ref 'refs/heads/master'
git: 'remote-https' is not a git command. See 'git --help'.
fatal: remote helper 'https' aborted session
warning: could not get HEAD ref for repository 'https://github.com/OnixResearch/nickel-export'; using expired cached ref 'refs/heads/main'
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: variant `Unimplemented` is never constructed
   --> vendor/snix-castore/src/directoryservice/combinators.rs:181:5
    |
167 | pub enum Error {
    |          ----- variant in this enum
...
181 |     Unimplemented,
    |     ^^^^^^^^^^^^^
    |
    = note: `Error` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `snix-castore` (lib) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_rust_cache-c2dad822e5101668)

running 15 tests
test shared::tests::oversized_candidate_index_is_rejected_before_file_reads ... ok
test shared::tests::malformed_candidate_reference_is_rejected_before_discovery ... ok
test shared::tests::concurrent_different_immutable_writers_never_overwrite ... ok
test shared::tests::offline_policy_never_opens_remote_source ... ok
test shared::tests::http_declared_metadata_limit_is_enforced_before_body_read ... ok
test shared::tests::http_redirect_is_rejected_without_following_location ... ok
test shared::tests::wrong_full_key_rejects_signed_candidate ... ok
test shared::tests::missing_object_rejects_candidate_without_output ... ok
test shared::tests::concurrent_exact_publication_deduplicates_without_overwrite ... ok
test shared::tests::candidate_marker_is_last_visibility_edge ... ok
test shared::tests::corrupt_object_is_rejected_before_local_admission ... ok
test shared::tests::truncated_object_rejects_incomplete_tree_transfer ... ok
test shared::tests::clean_client_restores_signed_remote_result ... ok
test shared::tests::conflicting_admissible_results_block_strong_reuse ... ok
test shared::tests::http_timeout_is_bounded_and_rejected ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.14s

exit_status=0
```

## Complete Rust cache crate tests

```text
$ nix develop -c cargo test -p crunch-rust-cache
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: variant `Unimplemented` is never constructed
   --> vendor/snix-castore/src/directoryservice/combinators.rs:181:5
    |
167 | pub enum Error {
    |          ----- variant in this enum
...
181 |     Unimplemented,
    |     ^^^^^^^^^^^^^
    |
    = note: `Error` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `snix-castore` (lib) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_rust_cache-c2dad822e5101668)

running 25 tests
test shared::tests::oversized_candidate_index_is_rejected_before_file_reads ... ok
test shared::tests::malformed_candidate_reference_is_rejected_before_discovery ... ok
test tests::cross_filesystem_commit_facts_fail_closed ... ok
test shared::tests::offline_policy_never_opens_remote_source ... ok
test shared::tests::concurrent_different_immutable_writers_never_overwrite ... ok
test shared::tests::http_redirect_is_rejected_without_following_location ... ok
test shared::tests::http_declared_metadata_limit_is_enforced_before_body_read ... ok
test tests::index_symlink_is_rejected_without_following ... ok
test tests::artifact_entry_bound_rejects_publication_without_result_state ... ok
test tests::existing_output_blocks_restore_without_mutation ... ok
test tests::publish_rejects_symlink_artifact ... ok
test tests::artifact_mismatch_cleans_interrupted_restore_staging ... ok
test shared::tests::concurrent_exact_publication_deduplicates_without_overwrite ... ok
test tests::retention_plan_keeps_accepted_roots_and_prunes_only_stale_records ... ok
test tests::publish_then_restore_excludes_mutable_receipt ... ok
test shared::tests::missing_object_rejects_candidate_without_output ... ok
test shared::tests::truncated_object_rejects_incomplete_tree_transfer ... ok
test shared::tests::clean_client_restores_signed_remote_result ... ok
test shared::tests::candidate_marker_is_last_visibility_edge ... ok
test tests::incomplete_castore_candidate_is_rejected ... ok
test shared::tests::wrong_full_key_rejects_signed_candidate ... ok
test tests::different_complete_results_report_conflict ... ok
test shared::tests::corrupt_object_is_rejected_before_local_admission ... ok
test shared::tests::conflicting_admissible_results_block_strong_reuse ... ok
test shared::tests::http_timeout_is_bounded_and_rejected ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

   Doc-tests crunch_rust_cache

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

exit_status=0
```

## Clean-client topology restore

```text
$ nix develop -c cargo test -p mantle --bin mantle rust_plan::tests::clean_client_shared_cache_hit_skips_second_compiler_invocation -- --exact --nocapture
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: variant `Unimplemented` is never constructed
   --> vendor/snix-castore/src/directoryservice/combinators.rs:181:5
    |
167 | pub enum Error {
    |          ----- variant in this enum
...
181 |     Unimplemented,
    |     ^^^^^^^^^^^^^
    |
    = note: `Error` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `snix-castore` (lib) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-0d8716b4c2df31e4)

running 1 test
test rust_plan::tests::clean_client_shared_cache_hit_skips_second_compiler_invocation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1932 filtered out; finished in 0.26s

exit_status=0
```

## Untrusted candidate compiler fallback

```text
$ nix develop -c cargo test -p mantle --bin mantle rust_plan::tests::untrusted_shared_candidate_records_rejection_before_compiler_fallback -- --exact --nocapture
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: variant `Unimplemented` is never constructed
   --> vendor/snix-castore/src/directoryservice/combinators.rs:181:5
    |
167 | pub enum Error {
    |          ----- variant in this enum
...
181 |     Unimplemented,
    |     ^^^^^^^^^^^^^
    |
    = note: `Error` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `snix-castore` (lib) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-0d8716b4c2df31e4)

running 1 test
test rust_plan::tests::untrusted_shared_candidate_records_rejection_before_compiler_fallback ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1932 filtered out; finished in 0.47s

exit_status=0
```

## Shared policy and signing-key CLI tests

```text
$ nix develop -c cargo test -p mantle --bin mantle shared_rust_ -- --nocapture
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: variant `Unimplemented` is never constructed
   --> vendor/snix-castore/src/directoryservice/combinators.rs:181:5
    |
167 | pub enum Error {
    |          ----- variant in this enum
...
181 |     Unimplemented,
    |     ^^^^^^^^^^^^^
    |
    = note: `Error` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `snix-castore` (lib) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-0d8716b4c2df31e4)

running 2 tests
test tests::shared_rust_cache_requires_local_execution_and_explicit_trust ... ok
test tests::shared_rust_signing_key_loader_requires_private_matching_keypair ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1931 filtered out; finished in 0.00s

exit_status=0
```

## Store completeness tests

```text
$ nix develop -c cargo test -p crunch-store completeness::tests::
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: variant `Unimplemented` is never constructed
   --> vendor/snix-castore/src/directoryservice/combinators.rs:181:5
    |
167 | pub enum Error {
    |          ----- variant in this enum
...
181 |     Unimplemented,
    |     ^^^^^^^^^^^^^
    |
    = note: `Error` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `snix-castore` (lib) generated 1 warning
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/vendor/snix-store)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/crates/crunch-store)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/crates/crunch-delta)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.46s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_store-af9805a039d6042b)

running 9 tests
test completeness::tests::node_visit_limit_accepts_last_supported_node_and_rejects_overflow ... ok
test completeness::tests::symlink_is_always_complete ... ok
test completeness::tests::directory_with_missing_blob_child_is_incomplete ... ok
test completeness::tests::completeness_rechecks_and_rejects_removed_directory ... ok
test completeness::tests::empty_directory_requires_existence ... ok
test completeness::tests::chunked_blob_metadata_must_match_declared_size ... ok
test completeness::tests::blob_node_rejects_declared_size_mismatch ... ok
test completeness::tests::blob_node_requires_blob_presence ... ok
test completeness::tests::bounded_depth_rejects_extremely_deep_trees ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 232 filtered out; finished in 0.00s

exit_status=0
```

## Cache core and shell clippy

```text
$ nix develop -c cargo clippy -p crunch-rust-cache-core -p crunch-rust-cache --all-targets --no-deps -- -D warnings
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
vendor/snix-castore/src/directoryservice/combinators.rs:181:5: warning: variant `Unimplemented` is never constructed
warning: `snix-castore` (lib) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s
exit_status=0
```

## Mantle test compilation

```text
$ nix develop -c cargo test -p mantle --bin mantle --no-run
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-remote-credentials/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: variant `Unimplemented` is never constructed
   --> vendor/snix-castore/src/directoryservice/combinators.rs:181:5
    |
167 | pub enum Error {
    |          ----- variant in this enum
...
181 |     Unimplemented,
    |     ^^^^^^^^^^^^^
    |
    = note: `Error` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: `snix-castore` (lib) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.29s
  Executable unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-0d8716b4c2df31e4)
exit_status=0
```

## Evidence summary

- Signed-envelope authority, exact full-key matching, modified-record rejection, and wrong-reference rejection passed.
- Ordered directory and HTTP discovery, offline no-open behavior, bounded metadata, redirect and timeout rejection passed.
- Missing, corrupt, and truncated object cases did not produce output.
- Exact concurrent publication deduplicated. Different immutable bytes were not overwritten.
- Conflicting admissible artifact sets produced a strong-reuse conflict.
- A clean client restored a signed result with no second compiler invocation.
- An untrusted result recorded bounded rejection evidence before compiler fallback.
- Publication receipts record the result identity and ordered object, envelope, and candidate steps.

## Bounded non-claims

- The focused rails do not prove compiler correctness.
- They do not prove full Cargo compatibility.
- They do not prove universal reproducibility.
- They do not prove remote executor correctness.
- They do not prove release eligibility.
## Cairn repository validation before sync

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
{
  "change_issues": [],
  "changes": 7,
  "cross_repo_dependencies": [],
  "cross_repo_evidence_issues": [],
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "resolution": {
    "diagnostics": [],
    "locator_consulted": false,
    "registry_lookup": "local_locator_only",
    "selected": {
      "project_identity": null,
      "root": ".",
      "source": "explicit_root",
      "store_id": null,
      "store_identity": null
    },
    "valid": true
  },
  "spec_findings": [],
  "spec_issues": [],
  "spec_substance": [
    {
      "path": "./cairn/changes/add-daemon-backed-rustc-wrapper/specs/rustc-cache-adapter/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/changes/materialize-stagex-lineage-provider/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/share-rust-unit-action-results/specs/cache-substitution/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/share-rust-unit-action-results/specs/rust-package-planning/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/artifact-auth-adoption/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/artifact-auth-operational-receipt/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/artifact-auth-shell-verification/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 48,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 33,
      "scenario_blocks": 77,
      "substantive_requirement_blocks": 33
    },
    {
      "path": "./cairn/specs/build-scheduling/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/build-tool-boundary/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/cache-substitution/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/examples/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/flake-source-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 1,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 22,
      "scenario_blocks": 52,
      "substantive_requirement_blocks": 22
    },
    {
      "path": "./cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/i386-tinycc27/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/kani-toolchain-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/kernel-bundle-oci/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 14
    },
    {
      "path": "./cairn/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/machine-artifact-contracts/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 32,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "./cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 24,
      "scenario_blocks": 80,
      "substantive_requirement_blocks": 24
    },
    {
      "path": "./cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/release-provenance/spec.md",
      "requirement_blocks": 69,
      "scenario_blocks": 102,
      "substantive_requirement_blocks": 69
    },
    {
      "path": "./cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 45,
      "scenario_blocks": 131,
      "substantive_requirement_blocks": 45
    },
    {
      "path": "./cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 134,
      "scenario_blocks": 462,
      "substantive_requirement_blocks": 134
    },
    {
      "path": "./cairn/specs/source-transports/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "./cairn/specs/spacewasm-reference-materialization/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/store-transports/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 26,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 58,
      "scenario_blocks": 164,
      "substantive_requirement_blocks": 58
    },
    {
      "path": "./cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 40,
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-daemon-backed-rustc-wrapper/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 58,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-daemon-backed-rustc-wrapper/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 20,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-daemon-backed-rustc-wrapper/specs/rustc-cache-adapter/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 16,
      "substantive_lines": 72,
      "substantive_requirement_blocks": 6,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 22,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-daemon-backed-rustc-wrapper/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 22,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 22
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_lines": 66,
      "substantive_requirement_blocks": 7,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 33,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 33,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 33,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 33
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 36,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_lines": 66,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 31,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 31,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 31,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 31
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/materialize-stagex-lineage-provider/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/materialize-stagex-lineage-provider/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/materialize-stagex-lineage-provider/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/materialize-stagex-lineage-provider/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 8
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 9,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 9,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 9,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 9
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 8
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/specs/cache-substitution/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/specs/rust-package-planning/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 17,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 17,
      "task_done": 16,
      "task_in_progress": 0,
      "task_todo": 1
    }
  ],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}
exit_status=0
```

## Cairn proposal gate before sync

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal share-rust-unit-action-results --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
{
  "change": "share-rust-unit-action-results",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "54b53d44ff8d1926992b2956f75c05feaf93e10e9e9a620c2022577143323da3",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "860fcf019180b030d855e90c3475a102dc6cdeb4aeb5214badeb84e0dd95145f",
  "probe_evidence": null,
  "receipt_hash": "6e3a865d95e959b8c4376f778ff6ce144825b592a88478a00f25a91a8a089429",
  "stage": "proposal",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
```

## Cairn design gate before sync

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design share-rust-unit-action-results --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
{
  "change": "share-rust-unit-action-results",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "78b1eb5651d560f1b087817b338a157a4199cfad3dbb57b19ae1876b6c6690e5",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "860fcf019180b030d855e90c3475a102dc6cdeb4aeb5214badeb84e0dd95145f",
  "probe_evidence": null,
  "receipt_hash": "a923186e48f82daa89159d7e30e31cf723f44c7d5c1a12bc4ef65a452e64e3ba",
  "stage": "design",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/specs/cache-substitution/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/specs/rust-package-planning/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
```

## Cairn tasks gate before sync

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks share-rust-unit-action-results --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
{
  "change": "share-rust-unit-action-results",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "9a91c684b6386fd0bee5c095ed186b6cc9d6113ca2bb0986c93bdb902bfff73b",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "860fcf019180b030d855e90c3475a102dc6cdeb4aeb5214badeb84e0dd95145f",
  "probe_evidence": {
    "declarations_present": false,
    "probes": []
  },
  "receipt_hash": "80f4ac69864cf7188691a16ea319a85f7cdc472e317fe187cf246a87fd432c96",
  "stage": "tasks",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/specs/cache-substitution/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/specs/rust-package-planning/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 17,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 17,
      "task_done": 16,
      "task_in_progress": 0,
      "task_todo": 1
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
```

## Tracey coverage before sync

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
{
  "command": "traceability coverage",
  "dangling": [
    "build_correctness.action_spec",
    "build_correctness.cas_object_store",
    "build_correctness.hermetic_execution_policy",
    "build_correctness.nickel_eval_source_closure",
    "build_correctness.output_reference_scanning",
    "build_correctness.reuse_and_substitution",
    "build_scheduling.deterministic_priority_kernel",
    "build_scheduling.deterministic_starvation_bound",
    "build_scheduling.lazy_known_critical_path",
    "build_scheduling.priority_decision_evidence",
    "build_scheduling.resource_locality_preference",
    "build_tool_boundary.correctness_primitives_frontend_neutral",
    "cache_substitution.rust_unit_action_result_discovery",
    "cache_substitution.rust_unit_action_result_discovery.candidate_admission",
    "cache_substitution.rust_unit_action_result_discovery.clean_client",
    "cache_substitution.rust_unit_action_result_discovery.conflicts",
    "cache_substitution.rust_unit_action_result_discovery.offline",
    "cache_substitution.rust_unit_action_result_discovery.publication",
    "compiled-eval.backend-boundary",
    "compiled-eval.backend-boundary.default",
    "compiled-eval.backend-boundary.swap",
    "compiled-eval.benchmark-guardrail",
    "compiled-eval.benchmark-guardrail.recorded",
    "compiled-eval.benchmark-guardrail.regression",
    "compiled-eval.cranelift-first",
    "compiled-eval.cranelift-first.initial",
    "compiled-eval.cranelift-first.llvm-later",
    "compiled-eval.cranelift-prototype-subset",
    "compiled-eval.cranelift-prototype-subset.default",
    "compiled-eval.cranelift-prototype-subset.unsupported",
    "compiled-eval.future-semantics",
    "compiled-eval.future-semantics.contracts",
    "compiled-eval.future-semantics.shape",
    "compiled-eval.private-backend-seam",
    "compiled-eval.private-backend-seam.callers",
    "compiled-eval.private-backend-seam.no-leak",
    "compiled-eval.profiling-gate",
    "compiled-eval.profiling-gate.evidence",
    "compiled-eval.profiling-gate.no-evidence",
    "external_batch_dispatchers.adapter_confinement",
    "external_batch_dispatchers.canonical_identity",
    "external_batch_dispatchers.diagnostics",
    "external_batch_dispatchers.fenced_lifecycle",
    "external_batch_dispatchers.final_validation",
    "external_batch_dispatchers.hardware_workload_composition",
    "external_batch_dispatchers.protocol",
    "external_batch_dispatchers.resources",
    "external_batch_dispatchers.slurm_adapter",
    "external_batch_dispatchers.typed_configuration",
    "external_batch_dispatchers.worker_handoff",
    "operator_diagnostics.remote_execution_telemetry",
    "operator_diagnostics.telemetry_exporter_isolation",
    "project_workflows.input_fetch_policy",
    "project_workflows.input_fetch_policy_preflight",
    "project_workflows.nixtamal_importer",
    "project_workflows.project_lock_importers",
    "remote_builds.attempt_scoped_transfer_resume",
    "remote_builds.diagnostic_trace_context",
    "remote_builds.durable_attempt_fencing",
    "remote_builds.idempotent_attempt_reporting",
    "remote_builds.immutable_attempt_log_segments",
    "remote_builds.pure_attempt_decisions",
    "remote_builds.pure_log_cursor_kernel",
    "rust_package_planning.unit_execution.topology.shared_cache_receipts",
    "store_transports.archive_export_closure",
    "store_transports.archive_import_idempotent",
    "store_transports.content_presence_early_cutoff",
    "store_transports.overlay_cli_declaration.scenario.missing-base-fails",
    "store_transports.overlay_cli_declaration.scenario.ordered-stack",
    "store_transports.overlay_composition.scenario.prefix-mismatch",
    "store_transports.overlay_composition.scenario.read-through-no-backfill",
    "store_transports.overlay_composition.scenario.shadow",
    "store_transports.overlay_composition.scenario.write-routing",
    "store_transports.overlay_provenance_layer.scenario.base-trust",
    "store_transports.overlay_provenance_layer.scenario.no-inherited-trust",
    "store_transports.pathinfo_final_nar_migration",
    "store_transports.receiver_driven_backpressure",
    "store_transports.resumable_castore_sessions",
    "verification_evidence.build_correctness_receipts",
    "verification_evidence.provider_fixed_point_path_normalization"
  ],
  "input_hash": "7e90d5715421d03dd44d336ae3aca3ac2f7c1859dab966b549e9e95d37aafd8e",
  "missing": [
    "bootstrap_inventory.blocker_signal",
    "bootstrap_inventory.bootstrap_gauntlet_inventory_pressure",
    "bootstrap_inventory.early_native_bootstrap_parity",
    "bootstrap_inventory.final_native_toolchain_parity",
    "bootstrap_inventory.full_source_rust_provider_binding",
    "bootstrap_inventory.offline_bootstrap_source_bundles",
    "bootstrap_inventory.perl_5_005_03_gcc_runtime",
    "build_correctness.determinism_normalization_policy",
    "build_correctness.dynamic_plan_nominal.compatibility",
    "build_correctness.dynamic_plan_nominal.compatibility.golden",
    "build_correctness.dynamic_plan_nominal.compile_time",
    "build_correctness.dynamic_plan_nominal.digests",
    "build_correctness.dynamic_plan_nominal.docs",
    "build_correctness.dynamic_plan_nominal.final_checks",
    "build_correctness.dynamic_plan_nominal.graph",
    "build_correctness.dynamic_plan_nominal.graph.ids",
    "build_correctness.dynamic_plan_nominal.graph.validation",
    "build_correctness.dynamic_plan_nominal.integration",
    "build_correctness.dynamic_plan_nominal.integration.compatibility",
    "build_correctness.dynamic_plan_nominal.integration.evidence",
    "build_correctness.dynamic_plan_nominal.integration.lifecycle",
    "build_correctness.dynamic_plan_nominal.integration.policy",
    "build_correctness.dynamic_plan_nominal.integration.preservation",
    "build_correctness.dynamic_plan_nominal.integration.source",
    "build_correctness.dynamic_plan_nominal.integration.target",
    "build_correctness.dynamic_plan_nominal.integration.validation",
    "build_correctness.dynamic_plan_nominal.integration.validation.negative",
    "build_correctness.dynamic_plan_nominal.octet",
    "build_correctness.dynamic_plan_nominal.octet.guard",
    "build_correctness.dynamic_plan_nominal.values",
    "build_correctness.dynamic_plan_nominal.values.valid",
    "build_correctness.dynamic_plan_nominal.values.validation",
    "build_correctness.dynamic_plan_nominal.wire_boundary",
    "build_correctness.dynamic_plan_nominal.wire_boundary.compatible",
    "build_correctness.explicit_environment_allowlist",
    "build_correctness.host_tool_attestation_inventory",
    "build_correctness.no_network_by_default",
    "build_correctness.receipt_bound_path",
    "build_tool_boundary.mantle_naming_consistency",
    "examples.rust_compatibility_surface_matrix",
    "foreign_derivation_import.adapter_neutral_ir",
    "foreign_derivation_import.checked_fixtures",
    "foreign_derivation_import.cli_file_boundary",
    "foreign_derivation_import.direct_drv_producer",
    "foreign_derivation_import.drv_dir_closure_producer",
    "foreign_derivation_import.fetch_and_cache_policy",
    "foreign_derivation_import.import_receipt",
    "foreign_derivation_import.integration_boundary",
    "foreign_derivation_import.live_nixpkgs_export_to_plan_proof",
    "foreign_derivation_import.nix_hash_domain_boundary",
    "foreign_derivation_import.nixpkgs_eval_boundary",
    "foreign_derivation_import.nixpkgs_producer_adapter",
    "foreign_derivation_import.nixpkgs_receipt_non_claims",
    "foreign_derivation_import.nixpkgs_substitution_first",
    "foreign_derivation_import.operator_cli_surface",
    "foreign_derivation_import.operator_trust_model_docs",
    "foreign_derivation_import.package_index_boundary",
    "foreign_derivation_import.pure_translation_core",
    "foreign_derivation_import.receipt_non_claims_documentation",
    "foreign_derivation_import.sandbox_capability_audit",
    "foreign_derivation_import.store_prefix_rewrite_policy",
    "foreign_derivation_import.trust_model_guard",
    "gcc40_bridge.source_frontier_reduction",
    "i386_tinycc27.handoff_probe",
    "kernel_bundle_oci.reports",
    "kernel_bundle_oci.verification",
    "kernelscript_experiment.artifacts",
    "kernelscript_experiment.codegen",
    "kernelscript_experiment.compiler",
    "kernelscript_experiment.evidence",
    "kernelscript_experiment.handoff",
    "kernelscript_experiment.kernel_inputs",
    "kernelscript_experiment.profile",
    "kernelscript_experiment.verification",
    "mantle.artifact_auth_adoption.source",
    "mantle.build_correctness.release_determinism.fixtures.negative.target_copy",
    "mantle.build_correctness.release_determinism.fixtures.positive",
    "mantle.flake_source_inventory.self_description",
    "mantle.kani_toolchain_evidence.bundle_linkage",
    "mantle.kani_toolchain_evidence.bundle_linkage.match",
    "mantle.kani_toolchain_evidence.bundle_linkage.mismatch",
    "mantle.kani_toolchain_evidence.identity",
    "mantle.kani_toolchain_evidence.identity.complete",
    "mantle.kani_toolchain_evidence.identity.missing",
    "mantle.kani_toolchain_evidence.negative_fixtures",
    "mantle.kani_toolchain_evidence.negative_fixtures.solver",
    "mantle.kani_toolchain_evidence.negative_fixtures.stale",
    "mantle.kani_toolchain_evidence.non_claims",
    "mantle.kani_toolchain_evidence.non_claims.missing",
    "mantle.kani_toolchain_evidence.operator_docs",
    "mantle.kani_toolchain_evidence.operator_docs.visible",
    "mantle.kani_toolchain_evidence.positive_fixtures",
    "mantle.kani_toolchain_evidence.positive_fixtures.valid",
    "mantle.kani_toolchain_evidence.valence_boundary",
    "mantle.kani_toolchain_evidence.valence_boundary.external",
    "mantle.kani_toolchain_evidence.valence_boundary.no_promotion",
    "mantle.nickel_export_cutover.authority",
    "mantle.nickel_export_cutover.boundary",
    "mantle.nickel_export_cutover.dual_run",
    "mantle.nickel_export_cutover.rollback",
    "mantle.nickel_export_cutover.source",
    "mantle.nickel_export_cutover.validation",
    "mantle.source_transports.vendor_source_manifests.contract",
    "mantle.source_transports.vendor_source_manifests.docs",
    "mantle.source_transports.vendor_source_manifests.final_validation",
    "mantle.source_transports.vendor_source_manifests.final_validation.fixtures",
    "mantle.source_transports.vendor_source_manifests.fixtures.negative",
    "mantle.source_transports.vendor_source_manifests.fixtures.positive",
    "mantle.source_transports.vendor_source_manifests.validation",
    "mantle.spacewasm_materialization.build",
    "mantle.spacewasm_materialization.bundle",
    "mantle.spacewasm_materialization.evidence",
    "mantle.spacewasm_materialization.fixtures",
    "mantle.spacewasm_materialization.functional_core",
    "mantle.spacewasm_materialization.nonclaims",
    "mantle.spacewasm_materialization.profile",
    "mantle.spacewasm_materialization.source",
    "mantle.spacewasm_materialization.validation",
    "mantle.verification_evidence.oxide_release_worker.cancel_safety.cleanup",
    "mantle.verification_evidence.oxide_release_worker.cancel_safety.mid_upload",
    "mantle.verification_evidence.oxide_release_worker.ephemeral_workers.complete_receipt",
    "mantle.verification_evidence.oxide_release_worker.ephemeral_workers.policy_override",
    "mantle.verification_evidence.oxide_release_worker.reference_inventory.boundary",
    "mantle.verification_evidence.oxide_release_worker.reference_inventory.non_authority",
    "mantle.verification_evidence.oxide_release_worker.tuf_bundles.signed_targets",
    "mantle.verification_evidence.oxide_release_worker.tuf_bundles.tag_mismatch",
    "mantle.verification_evidence.oxide_release_worker.validation.negative",
    "mantle.verification_evidence.oxide_release_worker.validation.positive",
    "mantle.wasm_component.build",
    "mantle.wasm_component.bundle",
    "mantle.wasm_component.cohort",
    "mantle.wasm_component.composition",
    "mantle.wasm_component.config",
    "mantle.wasm_component.digest_roles",
    "mantle.wasm_component.evidence",
    "mantle.wasm_component.final_validation",
    "mantle.wasm_component.functional_core",
    "mantle.wasm_component.package_resolution",
    "mantle.wasm_component.precompile",
    "mantle.wasm_component.validation",
    "mantle.wasm_component.virtualization",
    "mantle.wasm_component.wizer",
    "operator_diagnostics.offline_build_runbook",
    "project_workflows.cargo_import_vendored_sources",
    "project_workflows.freshness_probe_proof_rail",
    "project_workflows.input_trust_policy_proof_rail",
    "project_workflows.lock_importer_proof_rail",
    "project_workflows.offline_cargo_digest_bound_evidence",
    "project_workflows.retention_root_proof_rail",
    "realization_routing.remote_route_plan_cli",
    "realization_routing.source_bundle_route_execution",
    "remote_builds.coordinator_worker_runtime",
    "remote_builds.operator_e2e_rail",
    "remote_builds.operator_e2e_rail_composition_proof",
    "remote_builds.operator_observability",
    "remote_builds.output_trust_admission_pipeline",
    "remote_builds.production_ci_build_separation",
    "remote_builds.production_coordinator_runtime",
    "remote_builds.production_cryptographic_output_trust",
    "remote_builds.production_operator_configuration",
    "remote_builds.production_streaming_transfer",
    "remote_builds.scheduler_build_service_dispatch",
    "remote_builds.source_bundle_input_sync",
    "remote_builds.stdio_ssh_hardened_bindings",
    "rust_package_planning.bundle_deterministic_release_proof",
    "rust_package_planning.bundle_provider_fixed_point_release_evidence",
    "rust_package_planning.cargo_free_fixed_point_blocker_resolution",
    "rust_package_planning.cargo_free_fixed_point_command",
    "rust_package_planning.cargo_free_fixed_point_frontier_rerun",
    "rust_package_planning.cargo_free_fixed_point_proof",
    "rust_package_planning.cargo_free_self_build_command",
    "rust_package_planning.cargo_free_self_build_proof",
    "rust_package_planning.cargo_free_topology_proof",
    "rust_package_planning.cargo_oracle_parity",
    "rust_package_planning.cargo_oracle_parity.capture",
    "rust_package_planning.cargo_oracle_parity.compare",
    "rust_package_planning.compatibility_surface_matrix",
    "rust_package_planning.compiler_policy_adapter",
    "rust_package_planning.compiler_policy_adapter.cache_identity",
    "rust_package_planning.compiler_policy_adapter.cache_identity.raw_rustc_rejected",
    "rust_package_planning.compiler_policy_adapter.cli",
    "rust_package_planning.compiler_policy_adapter.fail_closed",
    "rust_package_planning.compiler_policy_adapter.fail_closed.missing_material",
    "rust_package_planning.compiler_policy_adapter.invocation",
    "rust_package_planning.compiler_policy_adapter.non_claims",
    "rust_package_planning.compiler_policy_adapter.non_claims.scope",
    "rust_package_planning.compiler_policy_adapter.octet",
    "rust_package_planning.compiler_policy_adapter.receipts",
    "rust_package_planning.compiler_policy_adapter.standards_gate",
    "rust_package_planning.fail_closed_boundaries",
    "rust_package_planning.fail_closed_boundaries.non_claims",
    "rust_package_planning.fail_closed_boundaries.unsupported",
    "rust_package_planning.full_source_rust_provider_binding",
    "rust_package_planning.host_target_split",
    "rust_package_planning.host_target_split.build_scripts",
    "rust_package_planning.host_target_split.proc_macros",
    "rust_package_planning.native_build_oracle_target_scope",
    "rust_package_planning.native_build_oracle_target_scope.normal_build",
    "rust_package_planning.native_build_script_execution_env",
    "rust_package_planning.native_build_script_package_metadata_env",
    "rust_package_planning.native_build_script_package_name_env",
    "rust_package_planning.native_build_script_profile_env",
    "rust_package_planning.native_build_script_runtime",
    "rust_package_planning.native_build_script_target_cfg_env",
    "rust_package_planning.native_build_topology_dev_dependency_scope",
    "rust_package_planning.native_build_topology_dev_dependency_scope.dev_test_explicit",
    "rust_package_planning.native_build_topology_dev_dependency_scope.normal_build",
    "rust_package_planning.native_compile_env_allowlist",
    "rust_package_planning.native_crate_disambiguators",
    "rust_package_planning.native_custom_build_alias_binding",
    "rust_package_planning.native_dependency_cap_lints",
    "rust_package_planning.native_dependency_feature_edge_scope",
    "rust_package_planning.native_executor_hardening",
    "rust_package_planning.native_feature_resolution",
    "rust_package_planning.native_git_dependency_source_scope",
    "rust_package_planning.native_host_artifact_topology_execution",
    "rust_package_planning.native_host_artifact_topology_execution.binds",
    "rust_package_planning.native_host_artifact_topology_execution.blockers",
    "rust_package_planning.native_host_artifact_topology_execution.bounded_claim",
    "rust_package_planning.native_host_artifact_topology_execution.executes",
    "rust_package_planning.native_host_artifact_topology_execution.metadata",
    "rust_package_planning.native_host_dependency_producer_coverage",
    "rust_package_planning.native_host_real_unit_identity",
    "rust_package_planning.native_host_unit_graph_planning",
    "rust_package_planning.native_host_unit_graph_planning.blockers",
    "rust_package_planning.native_host_unit_graph_planning.compare",
    "rust_package_planning.native_host_unit_graph_planning.consumes_native",
    "rust_package_planning.native_host_unit_graph_planning.receipts",
    "rust_package_planning.native_host_unit_graph_planning.tests",
    "rust_package_planning.native_host_unit_graph_planning.verify",
    "rust_package_planning.native_link_lib_metadata",
    "rust_package_planning.native_linked_build_script_metadata_env",
    "rust_package_planning.native_manifest_edition_derivations",
    "rust_package_planning.native_manifest_links_env",
    "rust_package_planning.native_manifest_lock_planner",
    "rust_package_planning.native_proc_macro_crate_type_classification",
    "rust_package_planning.native_proc_macro_host_dependency_binding",
    "rust_package_planning.native_proc_macro_target_name_normalization",
    "rust_package_planning.native_real_unit_identity",
    "rust_package_planning.native_registry_dependency_version_resolution",
    "rust_package_planning.native_registry_host_artifact_topology_execution",
    "rust_package_planning.native_registry_host_artifact_topology_execution.blockers",
    "rust_package_planning.native_registry_host_artifact_topology_execution.executes",
    "rust_package_planning.native_registry_host_artifact_topology_execution.receipts",
    "rust_package_planning.native_registry_host_artifact_topology_execution.source_and_host_facts",
    "rust_package_planning.native_registry_host_artifact_topology_execution.tests",
    "rust_package_planning.native_registry_host_artifact_topology_execution.verify",
    "rust_package_planning.native_registry_source",
    "rust_package_planning.native_registry_source.blockers",
    "rust_package_planning.native_registry_source.lockfile_identity",
    "rust_package_planning.native_registry_source.oracle_compare",
    "rust_package_planning.native_registry_source.receipts",
    "rust_package_planning.native_registry_source.tests",
    "rust_package_planning.native_registry_source.vendor_digest",
    "rust_package_planning.native_registry_source.verify",
    "rust_package_planning.native_registry_topology_execution",
    "rust_package_planning.native_registry_topology_execution.blockers",
    "rust_package_planning.native_registry_topology_execution.executes",
    "rust_package_planning.native_registry_topology_execution.receipts",
    "rust_package_planning.native_registry_topology_execution.source_facts",
    "rust_package_planning.native_registry_topology_execution.tests",
    "rust_package_planning.native_registry_topology_execution.verify",
    "rust_package_planning.native_registry_transitive_producer_coverage",
    "rust_package_planning.native_registry_unified_host_topology_execution",
    "rust_package_planning.native_registry_unified_host_topology_execution.blockers",
    "rust_package_planning.native_registry_unified_host_topology_execution.executes",
    "rust_package_planning.native_registry_unified_host_topology_execution.receipts",
    "rust_package_planning.native_registry_unified_host_topology_execution.source_and_host_facts",
    "rust_package_planning.native_registry_unified_host_topology_execution.tests",
    "rust_package_planning.native_registry_unified_host_topology_execution.verify",
    "rust_package_planning.native_selected_host_units",
    "rust_package_planning.native_self_package_lib_binding",
    "rust_package_planning.native_self_target_cfg_predicate_scope",
    "rust_package_planning.native_target_cfg_optional_dependency_scope",
    "rust_package_planning.native_target_proc_macro_host_extern_binding",
    "rust_package_planning.native_topology_hardening",
    "rust_package_planning.native_topology_parallel_evidence",
    "rust_package_planning.native_topology_race_free_fixtures",
    "rust_package_planning.native_topology_validation_stability",
    "rust_package_planning.native_transitive_search_paths",
    "rust_package_planning.native_unified_topology_execution",
    "rust_package_planning.native_unified_topology_execution.binds",
    "rust_package_planning.native_unified_topology_execution.blockers",
    "rust_package_planning.native_unified_topology_execution.executes",
    "rust_package_planning.native_unified_topology_execution.receipts",
    "rust_package_planning.native_unified_topology_execution.tests",
    "rust_package_planning.native_unified_topology_execution.verify",
    "rust_package_planning.native_unit_graph",
    "rust_package_planning.native_unit_graph_planning",
    "rust_package_planning.native_unit_graph_planning.blockers",
    "rust_package_planning.native_unit_graph_planning.compare",
    "rust_package_planning.native_unit_graph_planning.consumes_native",
    "rust_package_planning.native_unit_graph_planning.receipts",
    "rust_package_planning.native_unit_graph_planning.tests",
    "rust_package_planning.native_unit_graph_planning.verify",
    "rust_package_planning.native_unit_variant_artifacts",
    "rust_package_planning.no_cargo_oracle_cli",
    "rust_package_planning.provider_fixed_point_release_artifact_binding",
    "rust_package_planning.provider_fixed_point_release_verifier",
    "rust_package_planning.rust_topology_external_linker",
    "rust_package_planning.rust_topology_tool_path_env",
    "rust_package_planning.rustc_dev_guide.backend_invocation",
    "rust_package_planning.rustc_dev_guide.compiler_policy_adapter",
    "rust_package_planning.rustc_dev_guide.final_validation",
    "rust_package_planning.rustc_dev_guide.reference_map",
    "rust_package_planning.rustc_dev_guide.reference_map.pinned",
    "rust_package_planning.rustc_dev_guide.source_provider_patch_plan",
    "rust_package_planning.source_built_rust_seed_closure",
    "rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard",
    "rust_package_planning.source_built_toolchain_closure.bootstrap_patch_plan_boundary",
    "rust_package_planning.source_built_toolchain_closure.explicit_native_promotion",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_libgcc_eh_unwind",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_linker_wrapper_path",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_proc_macro_runtime",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_rustc_driver_rlib",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_prefix_runtime",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_rustc_probe_runtime",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_crt_normalization",
    "rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_pie_normalization",
    "rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier",
    "rust_package_planning.source_built_toolchain_closure.native_materialization",
    "rust_package_planning.source_built_toolchain_closure.native_static_pie_crt",
    "rust_package_planning.source_built_toolchain_closure.provider_contract_independence",
    "rust_package_planning.source_built_toolchain_closure.provider_fixed_point",
    "rust_package_planning.source_built_toolchain_closure.provider_status",
    "rust_package_planning.source_built_toolchain_closure.selectable_rust_source_route",
    "rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env",
    "rust_package_planning.source_built_toolchain_closure.source_root_musl_rust_provider_tools",
    "rust_package_planning.source_built_toolchain_closure.target_aliases",
    "rust_package_planning.source_closure",
    "rust_package_planning.source_closure.identities",
    "rust_package_planning.source_closure.offline",
    "rust_package_planning.source_root_host_target_topology",
    "rust_package_planning.unit_derivation_graph",
    "rust_package_planning.unit_derivation_graph.cache_identity",
    "rust_package_planning.unit_derivation_graph.units",
    "rust_package_planning.unit_execution",
    "rust_package_planning.unit_execution.bounded_claim",
    "rust_package_planning.unit_execution.build_script_metadata",
    "rust_package_planning.unit_execution.build_script_metadata.binds",
    "rust_package_planning.unit_execution.build_script_metadata.blockers",
    "rust_package_planning.unit_execution.build_script_metadata.captures",
    "rust_package_planning.unit_execution.build_script_metadata.executes",
    "rust_package_planning.unit_execution.build_script_metadata.link_binding",
    "rust_package_planning.unit_execution.build_script_metadata.link_lib_binding",
    "rust_package_planning.unit_execution.build_script_metadata.link_metadata_blockers",
    "rust_package_planning.unit_execution.build_script_metadata.link_search_binding",
    "rust_package_planning.unit_execution.dependency_chain",
    "rust_package_planning.unit_execution.dependency_chain.cli",
    "rust_package_planning.unit_execution.dependency_chain.cli.executes",
    "rust_package_planning.unit_execution.dependency_chain.produced_artifact",
    "rust_package_planning.unit_execution.host_artifacts",
    "rust_package_planning.unit_execution.host_artifacts.binds",
    "rust_package_planning.unit_execution.host_artifacts.blockers",
    "rust_package_planning.unit_execution.host_artifacts.blockers.missing_material",
    "rust_package_planning.unit_execution.host_artifacts.executes",
    "rust_package_planning.unit_execution.supported_unit",
    "rust_package_planning.unit_execution.target_topology",
    "rust_package_planning.unit_execution.target_topology.blockers",
    "rust_package_planning.unit_execution.target_topology.blockers.unsupported",
    "rust_package_planning.unit_execution.target_topology.executes",
    "rust_package_planning.unit_execution.topology",
    "rust_package_planning.unit_execution.topology.binds_all_artifacts",
    "rust_package_planning.unit_execution.topology.blockers",
    "rust_package_planning.unit_execution.topology.mixed_order",
    "rust_package_planning.unit_execution.topology.output_reuse",
    "rust_package_planning.unit_execution.topology.output_reuse.repeated",
    "rust_package_planning.unit_execution.topology.output_reuse_blockers",
    "rust_package_planning.unit_execution.topology.output_reuse_blockers.stale",
    "rust_package_planning.unit_execution.topology.unified_cli",
    "rust_package_planning.unit_execution_blockers",
    "rust_package_planning.unit_execution_blockers.dependency_chain",
    "rust_package_planning.unit_execution_blockers.dependency_chain.missing_material",
    "rust_package_planning.unit_execution_blockers.dependency_chain.unsupported_shape",
    "rust_package_planning.unit_execution_blockers.missing_material",
    "rust_package_planning.unit_execution_blockers.unsupported_boundary",
    "rust_package_planning.unit_execution_receipts",
    "rust_package_planning.unit_execution_receipts.build_script_metadata",
    "rust_package_planning.unit_execution_receipts.build_script_metadata.cli",
    "rust_package_planning.unit_execution_receipts.dependency_chain",
    "rust_package_planning.unit_execution_receipts.dependency_chain.cli",
    "rust_package_planning.unit_execution_receipts.dependency_chain.cli.ordered_units",
    "rust_package_planning.unit_execution_receipts.dependency_chain.ordered_units",
    "rust_package_planning.unit_execution_receipts.failure",
    "rust_package_planning.unit_execution_receipts.host_artifacts",
    "rust_package_planning.unit_execution_receipts.host_artifacts.cli",
    "rust_package_planning.unit_execution_receipts.host_artifacts.cli.json",
    "rust_package_planning.unit_execution_receipts.host_artifacts.ordered",
    "rust_package_planning.unit_execution_receipts.output_identity",
    "rust_package_planning.unit_execution_receipts.target_topology",
    "rust_package_planning.unit_execution_receipts.target_topology.cli",
    "rust_package_planning.unit_execution_receipts.target_topology.cli.json",
    "rust_package_planning.unit_execution_receipts.target_topology.ordered",
    "rust_package_planning.vendor_material_checksum_repair",
    "rust_package_planning.vendor_source_material_drift_diagnostics",
    "rust_package_planning.wrapperless_source_root_fixed_point",
    "verification_evidence.adversarial_hermeticity_gauntlet",
    "verification_evidence.bootstrap_pressure_gauntlet",
    "verification_evidence.cairn_lifecycle_runner",
    "verification_evidence.continuous_reproducibility_gauntlet",
    "verification_evidence.dependency_audit_actionable_findings",
    "verification_evidence.dependency_audit_policy_regression",
    "verification_evidence.dependency_audit_upstream_blockers",
    "verification_evidence.dependency_audit_waiver_resolution",
    "verification_evidence.dependency_security_audit",
    "verification_evidence.expensive_proof_evidence_refresh",
    "verification_evidence.external_witness_expansion",
    "verification_evidence.lifecycle_evidence_transcript",
    "verification_evidence.lifecycle_runner_fail_closed",
    "verification_evidence.nix_free_demo_bundle_cli",
    "verification_evidence.nix_free_demo_bundle_generator",
    "verification_evidence.nix_free_demo_bundle_manifest",
    "verification_evidence.nix_free_demo_generator_non_claims",
    "verification_evidence.nix_free_proof_demo_bundle",
    "verification_evidence.nix_mantle_comparison_corpus",
    "verification_evidence.operator_proof_guide",
    "verification_evidence.portable_release_verification_replay",
    "verification_evidence.proof_before_claim",
    "verification_evidence.proof_mode_hermeticity_audit_gate",
    "verification_evidence.provider_bound_release_evidence_refresh_transcripts",
    "verification_evidence.provider_bound_release_evidence_transcripts",
    "verification_evidence.release_repeatability_matrix",
    "verification_evidence.release_reproducibility_transcripts",
    "verification_evidence.strict_hermetic_proof_gate",
    "verification_evidence.strict_hermeticity_regression_suite",
    "verification_evidence.substitution_cache_attack_gauntlet"
  ],
  "policy": "cairn-default",
  "policy_hash": "860fcf019180b030d855e90c3475a102dc6cdeb4aeb5214badeb84e0dd95145f",
  "profile": {
    "evidence_sources": [
      {
        "extensions": [
          "rs"
        ],
        "root": "crates",
        "source_kind": "rust_source"
      },
      {
        "extensions": [
          "ncl"
        ],
        "root": "crates",
        "source_kind": "nickel_source"
      },
      {
        "extensions": [
          "rs",
          "ncl"
        ],
        "root": "tools",
        "source_kind": "tool_source"
      }
    ],
    "id": "cairn-default",
    "marker_verbs": [
      "impl",
      "verify",
      "depends",
      "related"
    ],
    "requirement_sources": [
      {
        "extensions": [
          "md"
        ],
        "root": "cairn/specs",
        "source_kind": "accepted_specs"
      }
    ]
  },
  "profile_id": "cairn-default",
  "receipt_hash": "8204f3e75b9c77329d39a91d1eba5a094ab2dfde3e364759f6dc4a848510b634",
  "referenced": 270,
  "requirements": 696,
  "root": ".",
  "source_summary": {
    "evidence_files": 270,
    "evidence_manifest_hash": "ae1ebb962bfbc1d1f3934c0b3363e385291271e5676b6d6497d1d6675cad3323",
    "requirement_files": 32,
    "requirement_manifest_hash": "f02be519379300b36d47dbaf7faec3811cb8ea933544ea55f58ca99fa1e63aa8"
  },
  "valid": false,
  "verdict": "fail"
}
error: tracey coverage failed
exit_status=1
```

Tracey-Status: 1
Tracey-New-Identifier-Check: inspect transcript; shared identifiers are referenced by tools/tracey_refs.rs

## Final tasks gate before sync

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks share-rust-unit-action-results --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
{
  "change": "share-rust-unit-action-results",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "0ee47bf4f8f27bd458770ab480531bec7ff050d513c38f3ae23a8ba6d9fefa8f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "860fcf019180b030d855e90c3475a102dc6cdeb4aeb5214badeb84e0dd95145f",
  "probe_evidence": {
    "declarations_present": false,
    "probes": []
  },
  "receipt_hash": "b14021a9b87a5030b8c0f39fce56e69a996526e2ff0c398e7b5f43fb9fe0ae47",
  "stage": "tasks",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/specs/cache-substitution/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/specs/rust-package-planning/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 17,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 18,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 17,
      "task_done": 17,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
```
