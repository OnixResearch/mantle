# Baseline tests

## foreign_derivation_import

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/fuse-backend-rs)
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/nix-compat)
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-attestation-core)
   Compiling nix-compat-derive v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/nix-compat-derive)
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-castore)
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-tracing)
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-build)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-store)
   Compiling crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-action-result-core)
   Compiling crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-gc-core)
   Compiling crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-overlay-core)
   Compiling crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-repair-core)
   Compiling crunch-rust-cache-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rust-cache-core)
   Compiling crunch-eval v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-eval)
   Compiling crunch-wasm-component-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-wasm-component-core)
   Compiling crunch-hardware-simulation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-hardware-simulation-core)
   Compiling crunch-shell-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-shell-core)
   Compiling crunch-delta-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-delta-core)
   Compiling crunch-bootstrap-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-bootstrap-core)
   Compiling crunch-kernelscript-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-kernelscript-core)
   Compiling crunch-release-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-release-core)
   Compiling mantlepkgs-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/mantlepkgs-core)
   Compiling crunch-shell v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-shell)
   Compiling mantle-portable-client-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/mantle-portable-client-core)
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-attestation)
   Compiling crunch-project-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-project-core)
   Compiling crunch-hardware-simulation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-hardware-simulation)
   Compiling crunch-wasm-component v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-wasm-component)
   Compiling crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-nar)
   Compiling crunch-glue v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-glue)
   Compiling crunch-project v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-project)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-store)
   Compiling crunch-rust-cache v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rust-cache)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-build)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-delta)
   Compiling crunch-rustc-wrapper v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rustc-wrapper)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 29.67s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/mantle-d895557f628ed146)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 174 filtered out; finished in 0.00s

```

## foreign_graph_compiler

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on artifact directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.50s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/mantle-d895557f628ed146)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 174 filtered out; finished in 0.00s

```

## fetch_build_service

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling regex-automata v0.4.14
   Compiling rustix v1.1.4
   Compiling mio v1.2.0
   Compiling digest v0.10.7
   Compiling indexmap v2.13.1
   Compiling rustls v0.23.37
   Compiling num-traits v0.2.19
   Compiling lazy_static v1.5.0
   Compiling derive_more-impl v2.1.1
   Compiling uuid v1.23.0
   Compiling rand v0.9.3
   Compiling rand_chacha v0.9.0
   Compiling sharded-slab v0.1.7
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/fuse-backend-rs)
   Compiling sha1 v0.10.6
   Compiling blake3 v1.8.2
   Compiling sha2 v0.10.9
   Compiling curve25519-dalek v4.1.3
   Compiling md-5 v0.10.6
   Compiling tokio v1.51.0
   Compiling sha1-checked v0.10.0
   Compiling petgraph v0.6.5
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-attestation-core)
   Compiling artifact-auth-core v0.1.0 (ssh://git@github.com/OnixResearch/onix-artifact.git?rev=c932138d880ddf4c2967f4c024b489b5c0022bf1#c932138d)
   Compiling crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-repair-core)
   Compiling crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-gc-core)
   Compiling crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-overlay-core)
   Compiling toml_edit v0.25.10+spec-1.1.0
   Compiling chrono v0.4.44
   Compiling crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-action-result-core)
   Compiling derive_more v2.1.1
   Compiling proc-macro-crate v3.5.0
   Compiling ed25519-dalek v2.2.0
   Compiling num_enum_derive v0.7.6
   Compiling artifact-auth-ed25519 v0.1.0 (ssh://git@github.com/OnixResearch/onix-artifact.git?rev=c932138d880ddf4c2967f4c024b489b5c0022bf1#c932138d)
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-attestation)
   Compiling tempfile v3.27.0
   Compiling xattr v1.6.1
   Compiling nix-archive v0.1.0
   Compiling tar v0.4.45
   Compiling rusty-fork v0.3.1
   Compiling num_enum v0.7.6
   Compiling proptest v1.11.0
   Compiling bstr v1.12.1
   Compiling matchers v0.2.0
   Compiling regex v1.12.3
   Compiling tracing-subscriber v0.3.23
   Compiling oci-spec v0.7.1
   Compiling gix-validate v0.11.0
   Compiling gix-utils v0.3.1
   Compiling gix-error v0.2.1
   Compiling gix-packetline v0.21.2
   Compiling gix-path v0.11.2
   Compiling gix-date v0.15.1
   Compiling gix-chunk v0.7.0
   Compiling gix-bitmap v0.3.0
   Compiling gix-quote v0.7.0
   Compiling rustls-platform-verifier v0.6.2
   Compiling ureq v3.3.0
   Compiling gix-features v0.46.2
   Compiling gix-config-value v0.17.1
   Compiling gix-command v0.8.0
   Compiling gix-url v0.35.2
   Compiling gix-actor v0.40.0
   Compiling gix-mailmap v0.32.0
   Compiling gix-prompt v0.14.1
   Compiling gix-hash v0.23.0
   Compiling gix-fs v0.19.2
   Compiling gix-glob v0.24.0
   Compiling tracing-indicatif v0.3.14
   Compiling gix-credentials v0.37.1
   Compiling gix-hashtable v0.13.0
   Compiling gix-commitgraph v0.35.0
   Compiling tokio-util v0.7.18
   Compiling tokio-rustls v0.26.4
   Compiling tower v0.5.3
   Compiling tokio-stream v0.1.18
   Compiling async-compression v0.4.19
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/nix-compat)
   Compiling gix-tempfile v21.0.2
   Compiling gix-attributes v0.31.0
   Compiling gix-ignore v0.19.1
   Compiling gix-object v0.58.0
   Compiling gix-lock v21.0.2
   Compiling fastcdc v3.2.1
   Compiling astral-tokio-tar v0.6.3
   Compiling tower-http v0.6.8
   Compiling gix-pathspec v0.16.1
   Compiling gix-shallow v0.10.0
   Compiling h2 v0.4.13
   Compiling n0-future v0.3.2
   Compiling irpc v0.13.0
   Compiling gix-revwalk v0.29.0
   Compiling gix-filter v0.28.0
   Compiling gix-ref v0.61.0
   Compiling gix-pack v0.68.0
   Compiling gix-traverse v0.55.0
   Compiling gix-revision v0.43.0
   Compiling gix-negotiate v0.29.0
   Compiling gix-index v0.49.0
   Compiling gix-worktree-stream v0.30.0
   Compiling gix-refspec v0.39.0
   Compiling gix-archive v0.30.0
   Compiling gix-discover v0.49.0
   Compiling gix-config v0.54.0
   Compiling crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-nar)
   Compiling crunch-glue v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-glue)
   Compiling gix-worktree v0.50.0
   Compiling gix-odb v0.78.0
   Compiling hyper v1.9.0
   Compiling gix-diff v0.61.0
   Compiling gix-dir v0.23.0
   Compiling gix-worktree-state v0.28.0
   Compiling gix-submodule v0.28.0
   Compiling gix-status v0.28.0
   Compiling gix-blame v0.11.0
   Compiling hyper-util v0.1.20
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.13.2
   Compiling reqwest-middleware v0.5.1
   Compiling object_store v0.14.0
   Compiling gix-transport v0.55.1
   Compiling reqwest-tracing v0.6.0
   Compiling gix-protocol v0.59.0
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-tracing)
   Compiling gix v0.81.0
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-castore)
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-build)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-store)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-store)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-build)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 31.93s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_build-793ece271dec2ecd)

running 23 tests
test fetch_build_service::tests::is_fetch_request_false_for_sandbox ... ok
test fetch_build_service::tests::is_fetch_request_false_for_empty_args ... ok
test fetch_build_service::tests::is_fetch_request_true_for_builtin ... ok
test fetch_build_service::tests::parse_git_fetch ... ok
test fetch_build_service::tests::parse_file_fetch ... ok
test fetch_build_service::tests::parse_executable_fetch ... ok
test fetch_build_service::tests::parse_invalid_url ... ok
test fetch_build_service::tests::parse_rejects_non_fetcher ... ok
test fetch_build_service::tests::parse_git_missing_rev ... ok
test fetch_build_service::tests::parse_missing_url ... ok
test fetch_build_service::tests::ordered_foreign_candidates_preserve_declared_order_and_kind ... ok
test fetch_build_service::tests::parse_tarball_fetch ... ok
test fetch_build_service::tests::ordered_foreign_candidates_reject_malformed_duplicate_and_stale_primary ... ok
test fetch_build_service::tests::parse_unpack_takes_priority_over_executable ... ok
test fetch_build_service::tests::source_override_requires_matching_git_revision ... ok
test fetch_build_service::tests::do_build_rejects_non_fetch ... ok
test fetch_build_service::tests::do_build_rejects_missing_url ... ok
test fetch_build_service::tests::required_source_override_rejects_unmatched_fetch_before_network ... ok
test fetch_build_service::tests::do_build_fetches_local_file ... ok
test fetch_build_service::tests::do_build_executable_sets_mode ... ok
test fetch_build_service::tests::ordered_foreign_candidates_select_first_available_source_state ... ok
test fetch_build_service::tests::do_build_uses_matching_source_override_without_network ... ok
test fetch_build_service::tests::do_build_fetches_tarball ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 641 filtered out; finished in 0.02s

```

## foreign_import_cli

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 44.27s
     Running tests/foreign_import_cli.rs (/home/brittonr/.cargo-target/debug/deps/foreign_import_cli-fe66b0e1541e05b7)

running 17 tests
test produce_backend_fix_and_host_nix_emit_parity_artifacts ... ignored, requires MANTLE_TEST_FIX_BACKEND_BINARY, MANTLE_TEST_NIX_INSTANTIATE_BINARY, and a reachable Nix daemon
test produce_backend_rejects_unknown_backend ... ok
test produce_backend_rejects_conflicting_target_modes ... ok
test foreign_import_cli_rejects_malformed_drv_without_partial_artifacts ... ok
test foreign_import_cli_rejects_drv_dir_missing_reachable_input_without_partial_artifacts ... ok
test foreign_import_cli_produce_aterm_rejects_input_mode_conflicts_without_artifacts ... ok
test foreign_import_cli_does_not_require_foreign_frontend_commands ... ok
test foreign_import_cli_produce_aterm_rejects_bad_bundles_without_partial_artifacts ... ok
test foreign_import_cli_produce_aterm_matches_nix_and_emits_guix_directory_artifacts ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_then_validates_and_plans_without_nix ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_from_drv_files_without_nix ... ok
test foreign_import_cli_produces_nixpkgs_artifacts_from_drv_dir_without_nix ... ok
test foreign_import_cli_produce_aterm_rejects_duplicate_and_bounded_byte_inputs ... ok
test foreign_import_cli_validates_and_plans_checked_fixtures ... ok
test foreign_import_cli_rejects_malformed_json_and_policy_failures ... ok
test foreign_import_cli_reports_fixed_output_mismatch_from_admitted_source_state ... ok
test foreign_import_cli_realizes_two_node_graph_and_reuses_exact_outputs ... ok

test result: ok. 16 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.55s

```

## Post-change binary-target producer check (not baseline)

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 24 tests
test foreign_derivation_import::tests::fixed_output_hash_mode_preserves_modern_and_legacy_recursive_facts ... ok
test foreign_derivation_import::tests::fixed_output_hash_mode_rejects_unknown_and_conflicting_facts ... ok
test foreign_derivation_import::tests::nix_environment_normalization_distinguishes_store_basenames_from_sri_hashes ... ok
test foreign_derivation_import::tests::nix_closure_selection_filters_unreachable_and_requires_inputs ... ok
test foreign_derivation_import::tests::nix_fetch_candidates_normalize_direct_unstructured_and_structured_forms ... ok
test foreign_derivation_import::tests::nix_lowering_emits_canonical_candidates_and_keeps_arbitrary_fixed_outputs_non_downloads ... ok
test foreign_derivation_import::tests::canonical_fetch_candidate_validation_rejects_conflicts_and_private_injection ... ok
test foreign_derivation_import::tests::nix_fetch_candidates_reject_invalid_ambiguous_and_private_forms ... ok
test foreign_derivation_import::tests::nixpkgs_hash_domain_and_frontend_metadata_fail_closed ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_malformed_and_mixed_inputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_normalizes_relative_paths_and_fod_env_outputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_missing_output_path_without_env_fallback ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_non_object_structured_attributes ... ok
test foreign_derivation_import::tests::embedded_source_payload_rewrite_requires_explicit_permission ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_preserves_structured_attributes_as_protocol_json ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_lowering_emits_guix_graph_facts ... ok
test foreign_derivation_import::tests::mantle_adapter_consumes_translated_hello_without_foreign_frontend_invocations ... ok
test foreign_derivation_import::tests::nixpkgs_derivation_json_lowering_preserves_identities_and_substitution_policy ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_parses_nix_and_guix_paths ... ok
test foreign_derivation_import::tests::translation_rejects_unsupported_builtins_references_stale_receipts_indexes_and_features ... ok
test foreign_derivation_import::tests::nix_aterm_derivation_closure_lowering_preserves_graph_facts ... ok
test foreign_derivation_import::tests::cache_and_sandbox_policy_fail_closed_without_trust_or_capability_allowance ... ok
test foreign_derivation_import::tests::translates_guix_and_nix_hello_fixtures_deterministically ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_missing_duplicate_non_utf8_and_limits ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 2305 filtered out; finished in 0.00s

```

## Post-change binary-target compiler check (not baseline)

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 19 tests
test foreign_graph_compiler::tests::exact_path_map_reuses_identical_entries_and_rejects_conflicts ... ok
test foreign_graph_compiler::tests::conflicting_cross_class_path_mapping_is_rejected ... ok
test foreign_graph_compiler::tests::unrelated_or_non_token_store_text_is_not_rewritten ... ok
test foreign_graph_compiler::tests::known_store_dir_placeholders_use_the_active_store_prefix ... ok
test foreign_graph_compiler::tests::dependency_order_rejects_cycles_duplicate_edges_unknown_outputs_and_partial_coverage ... ok
test foreign_graph_compiler::tests::foreign_builtin_lowering_rejects_malformed_hash_empty_candidates_and_digest_domain_substitution ... ok
test foreign_graph_compiler::tests::fixed_output_seed_outside_the_selected_root_is_not_required ... ok
test foreign_graph_compiler::tests::fixed_output_seed_admits_the_recomputed_output_path ... ok
test foreign_graph_compiler::tests::compiler_maps_source_descriptors_and_rewrites_suffixes ... ok
test foreign_graph_compiler::tests::fixed_output_seed_without_a_producer_binding_is_rejected ... ok
test foreign_graph_compiler::tests::cache_only_compiler_preserves_exact_paths_and_rejects_prefix_drift ... ok
test foreign_graph_compiler::tests::compiler_rewrites_store_paths_inside_structured_attribute_json ... ok
test foreign_graph_compiler::tests::compiler_lowers_git_revision_and_export_policy_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::compiler_lowers_ordered_executable_download_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::compiler_preserves_non_path_store_placeholders_but_rejects_valid_unknown_paths ... ok
test foreign_graph_compiler::tests::compiler_rejects_duplicate_maps_output_drift_and_unknown_embedded_paths ... ok
test foreign_graph_compiler::tests::compiler_binds_canonical_nix_candidates_and_order_into_recipe_identity ... ok
test foreign_graph_compiler::tests::execution_profile_changes_target_identity_and_reserved_collision_fails ... ok
test foreign_graph_compiler::tests::dependency_compiler_is_exact_deterministic_and_prefix_sensitive ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 2310 filtered out; finished in 0.01s

```

## Corrected producer baseline from origin/main 0be9ce9c

```text
warning: /home/brittonr/git/OnixResearch/mantle/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/mantle/vendor/fuse-backend-rs)
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/mantle/vendor/snix-castore)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle/vendor/snix-store)
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/mantle/vendor/snix-build)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/mantle/crates/crunch-store)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/mantle/crates/crunch-build)
   Compiling crunch-rust-cache v0.1.0 (/home/brittonr/git/OnixResearch/mantle/crates/crunch-rust-cache)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/mantle/crates/crunch-delta)
   Compiling crunch-rustc-wrapper v0.1.0 (/home/brittonr/git/OnixResearch/mantle/crates/crunch-rustc-wrapper)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/OnixResearch/mantle/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 15s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 20 tests
test foreign_derivation_import::tests::fixed_output_hash_mode_rejects_unknown_and_conflicting_facts ... ok
test foreign_derivation_import::tests::fixed_output_hash_mode_preserves_modern_and_legacy_recursive_facts ... ok
test foreign_derivation_import::tests::nix_environment_normalization_distinguishes_store_basenames_from_sri_hashes ... ok
test foreign_derivation_import::tests::nix_closure_selection_filters_unreachable_and_requires_inputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_missing_output_path_without_env_fallback ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_malformed_and_mixed_inputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_normalizes_relative_paths_and_fod_env_outputs ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_rejects_non_object_structured_attributes ... ok
test foreign_derivation_import::tests::nixpkgs_hash_domain_and_frontend_metadata_fail_closed ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_lowering_emits_guix_graph_facts ... ok
test foreign_derivation_import::tests::versioned_nix_derivation_export_preserves_structured_attributes_as_protocol_json ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_parses_nix_and_guix_paths ... ok
test foreign_derivation_import::tests::embedded_source_payload_rewrite_requires_explicit_permission ... ok
test foreign_derivation_import::tests::nix_aterm_derivation_closure_lowering_preserves_graph_facts ... ok
test foreign_derivation_import::tests::mantle_adapter_consumes_translated_hello_without_foreign_frontend_invocations ... ok
test foreign_derivation_import::tests::nixpkgs_derivation_json_lowering_preserves_identities_and_substitution_policy ... ok
test foreign_derivation_import::tests::cache_and_sandbox_policy_fail_closed_without_trust_or_capability_allowance ... ok
test foreign_derivation_import::tests::translation_rejects_unsupported_builtins_references_stale_receipts_indexes_and_features ... ok
test foreign_derivation_import::tests::translates_guix_and_nix_hello_fixtures_deterministically ... ok
test foreign_derivation_import::tests::prefix_aware_aterm_bundle_rejects_missing_duplicate_non_utf8_and_limits ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2304 filtered out; finished in 0.00s

```

## Corrected compiler baseline from origin/main 0be9ce9c

```text
warning: /home/brittonr/git/OnixResearch/mantle/Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-2f63cdc10b62e328)

running 18 tests
test foreign_graph_compiler::tests::conflicting_cross_class_path_mapping_is_rejected ... ok
test foreign_graph_compiler::tests::exact_path_map_reuses_identical_entries_and_rejects_conflicts ... ok
test foreign_graph_compiler::tests::unrelated_or_non_token_store_text_is_not_rewritten ... ok
test foreign_graph_compiler::tests::known_store_dir_placeholders_use_the_active_store_prefix ... ok
test foreign_graph_compiler::tests::dependency_order_rejects_cycles_duplicate_edges_unknown_outputs_and_partial_coverage ... ok
test foreign_graph_compiler::tests::foreign_builtin_lowering_rejects_malformed_hash_empty_candidates_and_digest_domain_substitution ... ok
test foreign_graph_compiler::tests::fixed_output_seed_outside_the_selected_root_is_not_required ... ok
test foreign_graph_compiler::tests::fixed_output_seed_without_a_producer_binding_is_rejected ... ok
test foreign_graph_compiler::tests::fixed_output_seed_admits_the_recomputed_output_path ... ok
test foreign_graph_compiler::tests::compiler_lowers_git_revision_and_export_policy_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::compiler_maps_source_descriptors_and_rewrites_suffixes ... ok
test foreign_graph_compiler::tests::compiler_lowers_ordered_executable_download_to_mantle_fetch_facts ... ok
test foreign_graph_compiler::tests::compiler_rewrites_store_paths_inside_structured_attribute_json ... ok
test foreign_graph_compiler::tests::cache_only_compiler_preserves_exact_paths_and_rejects_prefix_drift ... ok
test foreign_graph_compiler::tests::compiler_preserves_non_path_store_placeholders_but_rejects_valid_unknown_paths ... ok
test foreign_graph_compiler::tests::compiler_rejects_duplicate_maps_output_drift_and_unknown_embedded_paths ... ok
test foreign_graph_compiler::tests::execution_profile_changes_target_identity_and_reserved_collision_fails ... ok
test foreign_graph_compiler::tests::dependency_compiler_is_exact_deterministic_and_prefix_sensitive ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 2306 filtered out; finished in 0.00s

```
