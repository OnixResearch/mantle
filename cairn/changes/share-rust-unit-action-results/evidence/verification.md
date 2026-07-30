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
