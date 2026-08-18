# Final validation for Git source witness replay

Date: 2026-06-29
Change: `git-source-witness-replay`


## cargo fmt --check -p mantle -p crunch-release-core

```text
```

Exit status: 0

## cargo check -p crunch-release-core --target wasm32-unknown-unknown

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
```

Exit status: 0

## cargo test -p crunch-release-core manifest

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling unicode-ident v1.0.24
   Compiling shlex v1.3.0
   Compiling find-msvc-tools v0.1.9
   Compiling zmij v1.0.21
   Compiling itoa v1.0.18
   Compiling proc-macro2 v1.0.106
   Compiling cc v1.2.59
   Compiling quote v1.0.45
   Compiling blake3 v1.8.2
   Compiling syn v2.0.117
   Compiling serde_json v1.0.149
   Compiling serde_derive v1.0.228
   Compiling serde v1.0.228
   Compiling crunch-release-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-release-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.19s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_release_core-dbe793df6821936f)

running 26 tests
test manifest::tests::extract_full_proof_identity_fields_rejects_wrong_schema ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_malformed_digest ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_accepts_matching_binary ... ok
test manifest::tests::canonical_bytes_are_stable_and_compact ... ok
test manifest::tests::extract_full_proof_identity_fields_accepts_valid_manifest ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_mismatch ... ok
test manifest::tests::provider_fixed_point_release_artifact_binding_rejects_missing_binaries ... ok
test manifest::tests::extract_full_proof_identity_fields_rejects_missing_provider_kind ... ok
test manifest::tests::release_manifest_rejects_unknown_selected_provider_kind ... ok
test manifest::tests::validate_rejects_absolute_member_path ... ok
test manifest::tests::validate_accepts_deterministic_proof_artifacts ... ok
test manifest::tests::validate_accepts_git_source_acquisition ... ok
test manifest::tests::validate_accepts_matching_external_source_acquisition ... ok
test manifest::tests::validate_rejects_credential_bearing_git_source_url ... ok
test manifest::tests::validate_accepts_provider_fixed_point_proof_artifact ... ok
test manifest::tests::validate_rejects_deterministic_proof_with_wrong_path ... ok
test manifest::tests::validate_rejects_deterministic_proof_duplicate_path ... ok
test manifest::tests::validate_rejects_deterministic_proof_with_wrong_role ... ok
test manifest::tests::validate_rejects_git_source_archive_digest_mismatch ... ok
test manifest::tests::validate_rejects_git_source_with_malformed_commit ... ok
test manifest::tests::validate_rejects_prerequisite_inventory_linkage_mismatch ... ok
test manifest::tests::validate_rejects_provider_fixed_point_proof_file_kind ... ok
test manifest::tests::validate_rejects_provider_fixed_point_proof_with_wrong_role ... ok
test manifest::tests::validate_rejects_source_acquisition_digest_mismatch ... ok
test manifest::tests::validate_rejects_stage2_digest_not_present_in_binaries ... ok
test manifest::tests::validate_rejects_unsupported_source_acquisition_url ... ok

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 43 filtered out; finished in 0.00s

```

Exit status: 0

## cargo test -p crunch-release-core source_archive

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_release_core-dbe793df6821936f)

running 6 tests
test source_archive::tests::submodules_are_rejected ... ok
test source_archive::tests::symlink_targets_must_stay_relative ... ok
test source_archive::tests::private_runtime_and_lifecycle_paths_are_excluded ... ok
test manifest::tests::validate_rejects_git_source_archive_digest_mismatch ... ok
test source_archive::tests::unordered_entries_produce_stable_member_order ... ok
test source_archive::tests::unsafe_paths_are_rejected ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 63 filtered out; finished in 0.00s

```

Exit status: 0

## cargo test -p mantle --bin crunch release_evidence::

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling parking_lot_core v0.9.12
   Compiling equivalent v1.0.2
   Compiling version_check v0.9.5
   Compiling thiserror v2.0.18
   Compiling crossbeam-utils v0.8.21
   Compiling fnv v1.0.7
   Compiling cc v1.2.59
   Compiling getrandom v0.4.2
   Compiling rustversion v1.0.22
   Compiling strsim v0.11.1
   Compiling semver v1.0.28
   Compiling serde_json v1.0.149
   Compiling httparse v1.10.1
   Compiling icu_normalizer_data v2.2.0
   Compiling icu_properties_data v2.2.0
   Compiling ident_case v1.0.1
   Compiling autocfg v1.5.0
   Compiling unicode-xid v0.2.6
   Compiling hashbrown v0.16.1
   Compiling generic-array v0.14.7
   Compiling hashbrown v0.15.5
   Compiling indexmap v2.13.1
   Compiling unicode-segmentation v1.13.2
   Compiling cfg_aliases v0.2.1
   Compiling num-traits v0.2.19
   Compiling rustc_version v0.4.1
   Compiling cmake v0.1.58
   Compiling ring v0.17.14
   Compiling blake3 v1.8.2
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling heck v0.5.0
   Compiling anyhow v1.0.102
   Compiling convert_case v0.10.0
   Compiling lzma-sys v0.1.20
   Compiling aws-lc-sys v0.39.1
   Compiling parking_lot v0.12.5
   Compiling tempfile v3.27.0
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling libmimalloc-sys v0.1.44
   Compiling memoffset v0.6.5
   Compiling crossbeam-channel v0.5.15
   Compiling dashmap v6.1.0
   Compiling crossbeam-epoch v0.9.18
   Compiling prodash v31.0.0
   Compiling toml_datetime v1.1.1+spec-1.1.0
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling syn v2.0.117
   Compiling petgraph v0.7.1
   Compiling clap_builder v4.6.0
   Compiling crossbeam-deque v0.8.6
   Compiling string_cache v0.8.9
   Compiling vt100 v0.16.2
   Compiling toml_edit v0.25.10+spec-1.1.0
   Compiling digest v0.10.7
   Compiling arc-swap v1.9.1
   Compiling lalrpop-util v0.22.2
   Compiling curve25519-dalek v4.1.3
   Compiling heapless v0.7.17
   Compiling rayon-core v1.13.0
   Compiling n0-future v0.3.2
   Compiling thiserror v1.0.69
   Compiling sha3 v0.10.8
   Compiling malachite-base v0.6.1
   Compiling sha1 v0.10.6
   Compiling sha2 v0.10.9
   Compiling indicatif v0.18.4
   Compiling proc-macro-crate v3.5.0
   Compiling logos-codegen v0.15.1
   Compiling synstructure v0.13.2
   Compiling serde_derive v1.0.228
   Compiling thiserror-impl v2.0.18
   Compiling tokio-macros v2.7.0
   Compiling futures-macro v0.3.32
   Compiling zerovec-derive v0.11.3
   Compiling sha1-checked v0.10.0
   Compiling displaydoc v0.2.5
   Compiling tracing-attributes v0.1.31
   Compiling darling_core v0.23.0
   Compiling async-trait v0.1.89
   Compiling clap_derive v4.6.0
   Compiling zerofrom-derive v0.1.7
   Compiling yoke-derive v0.8.2
   Compiling derive_more-impl v2.1.1
   Compiling pin-project-internal v1.1.11
   Compiling futures-util v0.3.32
   Compiling tracing v0.1.44
   Compiling lalrpop v0.22.2
   Compiling n0-error-macros v0.1.3
   Compiling zerofrom v0.1.7
   Compiling darling_macro v0.23.0
   Compiling thiserror-impl v1.0.69
   Compiling spez v0.1.2
   Compiling serde v1.0.228
   Compiling async-stream-impl v0.3.6
   Compiling clap v4.6.0
   Compiling pin-project v1.1.11
   Compiling derive_more v2.1.1
   Compiling tracing-subscriber v0.3.23
   Compiling yoke v0.8.2
   Compiling curve25519-dalek-derive v0.1.1
   Compiling darling_core v0.20.11
   Compiling darling v0.23.0
   Compiling proc-macro2-diagnostics v0.10.1
   Compiling n0-error v0.1.3
   Compiling mimalloc v0.1.48
   Compiling clap-verbosity-flag v3.0.4
   Compiling async-stream v0.3.6
   Compiling bzip2 v0.5.2
   Compiling cobs v0.3.0
   Compiling bytes v1.11.1
   Compiling bstr v1.12.1
   Compiling zerovec v0.11.6
   Compiling zerotrie v0.2.4
   Compiling serde_with_macros v3.18.0
   Compiling futures-executor v0.3.32
   Compiling serde_urlencoded v0.7.1
   Compiling tracing-indicatif v0.3.14
   Compiling darling_macro v0.20.11
   Compiling quick-xml v0.39.2
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-attestation-core)
   Compiling chrono v0.4.44
   Compiling malachite-nz v0.6.1
   Compiling tokio v1.51.0
   Compiling http v1.4.0
   Compiling nix v0.24.3
   Compiling tinystr v0.8.3
   Compiling gix-validate v0.11.0
   Compiling gix-utils v0.3.1
   Compiling gix-error v0.2.1
   Compiling potential_utf v0.1.5
   Compiling gix-packetline v0.21.2
   Compiling serde_with v3.18.0
   Compiling futures v0.3.32
   Compiling icu_locale_core v2.2.0
   Compiling gix-path v0.11.2
   Compiling num_enum_derive v0.7.6
   Compiling gix-date v0.15.1
   Compiling gix-chunk v0.7.0
   Compiling http-body v1.0.1
   Compiling gix-quote v0.7.0
   Compiling gix-bitmap v0.3.0
   Compiling irpc-derive v0.10.0
   Compiling icu_collections v2.2.0
   Compiling md-5 v0.10.6
   Compiling rand v0.10.1
   Compiling icu_provider v2.2.0
   Compiling petgraph v0.6.5
   Compiling gix-features v0.46.2
   Compiling http-body-util v0.1.3
   Compiling gix-command v0.8.0
   Compiling gix-actor v0.40.0
   Compiling gix-config-value v0.17.1
   Compiling gix-url v0.35.2
   Compiling num_enum v0.7.6
   Compiling serde_tagged v0.3.0
   Compiling ed25519-dalek v2.2.0
   Compiling postcard v1.1.3
   Compiling darling v0.20.11
   Compiling icu_properties v2.2.0
   Compiling icu_normalizer v2.2.0
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/mantle/vendor/fuse-backend-rs)
   Compiling serde_qs v0.12.0
   Compiling nix-compat-derive v0.1.0 (/home/brittonr/git/mantle/vendor/nix-compat-derive)
   Compiling gix-hash v0.23.0
   Compiling gix-fs v0.19.2
   Compiling gix-glob v0.24.0
   Compiling tokio-util v0.7.18
   Compiling tower v0.5.3
   Compiling tokio-stream v0.1.18
   Compiling auto_impl v1.3.0
   Compiling malachite-q v0.6.1
   Compiling zerocopy v0.8.48
   Compiling codespan-reporting v0.13.1
   Compiling derive_builder_core v0.20.2
   Compiling ouroboros_macro v0.18.5
   Compiling gix-hashtable v0.13.0
   Compiling gix-commitgraph v0.35.0
   Compiling nix-compat v0.1.0 (/home/brittonr/git/mantle/vendor/nix-compat)
   Compiling gix-tempfile v21.0.2
   Compiling idna_adapter v1.2.1
   Compiling gix-attributes v0.31.0
   Compiling gix-ignore v0.19.1
   Compiling tower-http v0.6.8
   Compiling fastcdc v3.2.1
   Compiling h2 v0.4.13
   Compiling astral-tokio-tar v0.6.0
   Compiling malachite-float v0.6.1
   Compiling gix-prompt v0.14.1
   Compiling gix-object v0.58.0
   Compiling logos-derive v0.15.1
   Compiling idna v1.1.0
   Compiling gix-lock v21.0.2
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/mantle/crates/crunch-attestation)
   Compiling gix-pathspec v0.16.1
   Compiling nickel-lang-parser v0.1.1
   Compiling proc-macro-error-attr2 v2.0.0
   Compiling irpc v0.13.0
   Compiling hashlink v0.10.0
   Compiling imara-diff v0.2.0
   Compiling imara-diff v0.1.8
   Compiling paste v1.0.15
   Compiling ouroboros v0.18.5
   Compiling gix-credentials v0.37.1
   Compiling url v2.5.8
   Compiling logos v0.15.1
   Compiling malachite v0.6.1
   Compiling gix-revwalk v0.29.0
   Compiling gix-ref v0.61.0
   Compiling gix-filter v0.28.0
   Compiling saphyr-parser v0.0.6
   Compiling proc-macro-error2 v2.0.1
   Compiling derive_builder_macro v0.20.2
   Compiling hyper v1.9.0
   Compiling codespan v0.13.1
   Compiling nickel-lang-vector v0.1.0
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling nickel-lang-core v0.16.1
   Compiling gix-traverse v0.55.0
   Compiling gix-revision v0.43.0
   Compiling lru v0.16.4
   Compiling clru v0.6.3
   Compiling nix v0.29.0
   Compiling serde_yaml v0.9.34+deprecated
   Compiling derive_builder v0.20.2
   Compiling getset v0.1.6
   Compiling gix-negotiate v0.29.0
   Compiling gix-discover v0.49.0
   Compiling gix-config v0.54.0
   Compiling hyper-util v0.1.20
   Compiling gix-shallow v0.10.0
   Compiling gix-refspec v0.39.0
   Compiling gix-index v0.49.0
   Compiling gix-worktree-stream v0.30.0
   Compiling gix-pack v0.68.0
   Compiling strum_macros v0.26.4
   Compiling maybe-async v0.2.10
   Compiling typed-builder-macro v0.22.0
   Compiling sha-1 v0.10.1
   Compiling indoc v2.0.7
   Compiling gix-mailmap v0.32.0
   Compiling ureq-proto v0.6.0
   Compiling uuid v1.23.0
   Compiling serde_spanned v0.6.9
   Compiling gix-submodule v0.28.0
   Compiling gix-archive v0.30.0
   Compiling toml_datetime v0.6.11
   Compiling crunch-glue v0.1.0 (/home/brittonr/git/mantle/crates/crunch-glue)
   Compiling ppv-lite86 v0.2.21
   Compiling crunch-project-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project-core)
   Compiling crunch-shell-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-shell-core)
   Compiling typed-builder v0.22.0
   Compiling gix-worktree v0.50.0
   Compiling oci-spec v0.7.1
   Compiling float-cmp v0.10.0
   Compiling gix-odb v0.78.0
   Compiling crunch-bootstrap-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-bootstrap-core)
   Compiling crunch-release-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-release-core)
   Compiling toml_edit v0.22.27
   Compiling crunch-shell v0.1.0 (/home/brittonr/git/mantle/crates/crunch-shell)
   Compiling rand_chacha v0.3.1
   Compiling predicates v3.1.4
   Compiling gix-diff v0.61.0
   Compiling gix-dir v0.23.0
   Compiling gix-worktree-state v0.28.0
   Compiling crunch-project v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project)
   Compiling rand v0.8.6
   Compiling toml v0.8.23
   Compiling assert_cmd v2.2.0
   Compiling gix-blame v0.11.0
   Compiling gix-status v0.28.0
   Compiling xz2 v0.1.7
   Compiling zstd-safe v7.2.4
   Compiling zstd v0.13.3
   Compiling async-compression v0.4.19
   Compiling aws-lc-rs v1.16.2
   Compiling rustls v0.23.37
   Compiling rustls-webpki v0.103.12
   Compiling tokio-rustls v0.26.4
   Compiling rustls-platform-verifier v0.6.2
   Compiling ureq v3.3.0
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.13.2
   Compiling reqwest v0.12.28
   Compiling object_store v0.13.2
   Compiling reqwest-middleware v0.5.1
   Compiling gix-transport v0.55.1
   Compiling reqwest-tracing v0.6.0
   Compiling gix-protocol v0.59.0
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/mantle/vendor/snix-tracing)
   Compiling gix v0.81.0
   Compiling snix-castore v0.1.0 (/home/brittonr/git/mantle/vendor/snix-castore)
   Compiling snix-store v0.1.0 (/home/brittonr/git/mantle/vendor/snix-store)
   Compiling snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/mantle/crates/crunch-store)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/mantle/crates/crunch-delta)
   Compiling nickel-lang v2.0.0
   Compiling crunch-eval v0.1.0 (/home/brittonr/git/mantle/crates/crunch-eval)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/mantle/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 52.99s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 18 tests
test release_evidence::tests::canonical_bytes_are_stable_and_compact ... ok
test release_evidence::tests::validate_rejects_absolute_member_path ... ok
test release_evidence::tests::validate_rejects_prerequisite_inventory_linkage_mismatch ... ok
test release_evidence::tests::validate_rejects_stage2_digest_not_present_in_binaries ... ok
test release_evidence::tests::load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact ... ok
test release_evidence::tests::create_rejects_empty_git_source_commit_before_manifest ... ok
test release_evidence::tests::create_rejects_empty_source_acquisition_url_before_manifest ... ok
test release_evidence::tests::create_rejects_conflicting_source_acquisition_modes_before_manifest ... ok
test release_evidence::tests::load_full_self_hosting_proof_identity_accepts_full_proof_bundle ... ok
test release_evidence::tests::create_rejects_invalid_provider_fixed_point_proof_before_manifest ... ok
test release_evidence::tests::create_rejects_provider_fixed_point_proof_for_different_binary ... ok
test release_evidence::tests::create_and_verify_release_bundle_records_git_source_acquisition ... ok
test release_evidence::tests::create_and_verify_release_bundle_round_trip ... ok
test release_evidence::tests::create_and_verify_release_bundle_records_source_acquisition_url ... ok
test release_evidence::tests::verify_rejects_tampered_binary_artifact ... ok
test release_evidence::tests::verify_rejects_non_canonical_manifest_json ... ok
test release_evidence::tests::verify_rejects_provider_kind_linkage_mismatch ... ok
test release_evidence::tests::create_and_verify_release_bundle_with_provider_fixed_point_proof ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 905 filtered out; finished in 0.03s

```

Exit status: 0

## cargo test -p mantle --bin crunch release_source::

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 7 tests
test release_source::tests::source_archive_rejects_symlink_targets_outside_tree ... ok
test release_source::tests::source_archive_rejects_git_submodule_entries ... ok
test release_source::tests::source_archive_uses_current_worktree_verified_vendor_and_tracked_package_files ... ok
test release_source::tests::git_source_archive_rejects_wrong_commit_before_archiving ... ok
test release_source::tests::git_source_archive_rejects_missing_ref_policy ... ok
test release_source::tests::git_source_archive_reconstruction_matches_release_source_archive ... ok
test release_source::tests::git_source_archive_rejects_ref_policy_mismatch ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 916 filtered out; finished in 0.04s

```

Exit status: 0

## cargo test -p mantle --bin crunch witness_rebuild::

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 35 tests
test witness_rebuild::tests::bootstrap_divergence_diagnostic_reports_convergence ... ok
test witness_rebuild::tests::default_witness_scratch_dir_appends_work_suffix ... ok
test witness_rebuild::tests::launched_workflow_command_records_proof_mode_arguments ... ok
test witness_rebuild::tests::parse_request_relative_path_rejects_parent_components ... ok
test witness_rebuild::tests::resolve_proof_bundle_artifact_path_rejects_parent_escape ... ok
test witness_rebuild::tests::resolve_proof_bundle_artifact_path_anchors_relative_paths ... ok
test witness_rebuild::tests::source_acquisition_for_plan_accepts_metadata_when_flagged ... ok
test witness_rebuild::tests::source_acquisition_for_plan_requires_git_kind_when_flagged ... ok
test witness_rebuild::tests::source_acquisition_for_plan_requires_metadata_when_flagged ... ok
test witness_rebuild::tests::validate_source_replay_flags_rejects_conflicting_strict_modes ... ok
test witness_rebuild::tests::workflow_args_preserve_non_nix_proof_mode_and_reject_unknown_modes ... ok
test witness_rebuild::tests::validate_supported_workflow_identity_rejects_unknown_pair ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_escaping_proof_binary_path ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_absolute_proof_binary_path ... ok
test witness_rebuild::tests::git_source_success_audit_records_derivation_details ... ok
test witness_rebuild::tests::source_acquisition_prelaunch_failure_audit_records_error ... ok
test witness_rebuild::tests::failure_audit_reports_gcc_bootstrap_divergence_root ... ok
test witness_rebuild::tests::failure_audit_reports_digest_mismatch_diagnostics ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_symlink_root ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_file_helper_owned_cargo_target_entry ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_symlinked_helper_owned_tmp_entry ... ok
test witness_rebuild::tests::validate_rebuilt_output_digests_rejects_stripped_equivalent_but_different_binary ... ok
test witness_rebuild::tests::validate_existing_scratch_root_rejects_unexpected_entries ... ok
test witness_rebuild::tests::source_acquisition_success_audit_records_verified_fetch ... ok
test witness_rebuild::tests::source_acquisition_rejects_digest_mismatch_before_extraction ... ok
test witness_rebuild::tests::validate_existing_scratch_root_allows_helper_owned_dirs_only ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_missing_expected_digest ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_invalid_provider_fixed_point_proof ... ok
test witness_rebuild::tests::prepare_scratch_fetches_independent_source_archive_when_required ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_rejects_provider_fixed_point_stage_escape ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_preserves_one_output_legacy_manifest ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_maps_two_expected_outputs_by_digest ... ok
test witness_rebuild::tests::collect_rebuilt_output_paths_uses_provider_fixed_point_candidate_by_digest ... ok
test witness_rebuild::tests::git_source_acquisition_rejects_digest_mismatch_before_extraction ... ok
test witness_rebuild::tests::prepare_scratch_fetches_git_source_archive_when_required ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 888 filtered out; finished in 0.04s

```

Exit status: 0

## cargo test -p mantle --test release_cli release_create_records_git_source_metadata

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.16s
     Running tests/release_cli.rs (/home/brittonr/.cargo-target/debug/deps/release_cli-adc6653fd83c9572)

running 1 test
test release_create_records_git_source_metadata ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 105 filtered out; finished in 0.07s

```

Exit status: 0

## cargo test -p mantle --test release_cli release_create_rejects_git_source_url_without_commit

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running tests/release_cli.rs (/home/brittonr/.cargo-target/debug/deps/release_cli-adc6653fd83c9572)

running 1 test
test release_create_rejects_git_source_url_without_commit ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 105 filtered out; finished in 0.02s

```

Exit status: 0

## cargo test -p mantle --test release_cli release_create_rejects_conflicting_source_acquisition_flags

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running tests/release_cli.rs (/home/brittonr/.cargo-target/debug/deps/release_cli-adc6653fd83c9572)

running 1 test
test release_create_rejects_conflicting_source_acquisition_flags ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 105 filtered out; finished in 0.02s

```

Exit status: 0

## git diff --check

```text
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate proposal git-source-witness-replay --root /home/brittonr/git/mantle

```text
{
  "change": "git-source-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "945a2e11d05e020e34ee216bd9152ea9284989c1d76be384c25c3bdfb6d91e5c",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ff8282a712aa543ea2024b9d4e7dc0534db59be4d0c0246177c76990586e6549",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate design git-source-witness-replay --root /home/brittonr/git/mantle

```text
{
  "change": "git-source-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "c7e9a68d501dd25788c304a6d7c59813d2456d6559eb63f77e26e80fedbf2908",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "bfcd701cb34edb1df97bed6e2e78604962ec12a7cc8434cc56acc6d820d0eef4",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- gate tasks git-source-witness-replay --root /home/brittonr/git/mantle

```text
{
  "change": "git-source-witness-replay",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1436659d206e514c74c47373c72bebb508b561cd3f8df4c268a59c3d23811aac",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b212c87167275ecee8ac6af53e110fff66ba1e59e31d4bb7cfbfaaa9a3cd8940",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

Exit status: 0
