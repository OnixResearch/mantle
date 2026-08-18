# Rust provider wrapper rejection

Task-ID: rust-provider-wrapper-rejection-2026-06-10
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now rejects text-like `rustc` and `cargo` provider artifacts that contain disallowed Rust-provider markers such as Nix/rustup/prebuilt references, even when provider metadata and receipt digests have been updated to match the wrapper bytes. This prevents a provider directory from relabeling a shell wrapper around `/nix/store/.../rustc` as source-built.

This does not materialize a real source-built Rust provider. It tightens the existing import/validation boundary so the remaining materialization and proof tasks stay fail-closed unless a real provider is supplied.

## Commands and output

### provider wrapper rejection tests

Command:

```sh
cargo test -p mantle --bin mantle rust_source_provider
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 34 tests
test cargo_free_self_build::tests::rust_source_provider_selection_prefers_validated_provider_and_keeps_fallback ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_provenance ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_source_name ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_missing_target_rustlib_role ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_provider_receipt_digest_link_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_digest_mismatch_from_shell_observation ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_bad_receipt_schema ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_digest_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_artifact_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_rustlib_outside_lib_dir ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_with_prebuilt_step ... ok
test rust_source_provider::tests::materializer_fails_closed_without_claiming_prebuilt_rust ... ok
test source_toolchain_closure::tests::rust_source_provider_accepts_matching_receipt_payload ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_metadata_yields_stable_digest ... ok
test rust_source_provider::tests::directory_validator_rejects_nix_rustc_wrapper_even_with_matching_digest ... ok
test tests::bootstrap_rust_source_provider_fails_closed_without_output ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test rust_source_provider::tests::directory_validator_accepts_complete_fake_provider ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok
test rust_source_provider::tests::import_provider_rejects_existing_output_without_overwrite ... ok
test rust_source_provider::tests::directory_validator_rejects_artifact_digest_mismatch ... ok
test rust_source_provider::tests::import_provider_rejects_prebuilt_metadata_without_output ... ok
test rust_source_provider::tests::import_provider_rejects_malformed_receipt_without_output ... ok
test rust_source_provider::tests::smoke_provider_rejects_invalid_metadata_before_launch ... ok
test tests::self_build_cli_rejects_rust_source_provider_without_cargo_free ... ok
test tests::self_build_cli_accepts_cargo_free_rust_source_provider ... ok
test tests::bootstrap_rust_source_provider_action_parses_smoke_evidence_dir ... ok
test tests::bootstrap_rust_source_provider_action_parses_import_dir ... ok
test tests::bootstrap_rust_source_provider_rejects_smoke_evidence_without_smoke ... ok
test tests::bootstrap_rust_source_provider_action_parses ... ok
test rust_source_provider::tests::smoke_provider_runs_synthetic_rustc_after_validation ... ok
test rust_source_provider::tests::import_provider_copies_validated_provider ... ok
test rust_source_provider::tests::smoke_evidence_rejects_tampered_smoke_output ... ok
test rust_source_provider::tests::smoke_evidence_persists_summary_output_and_metadata ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 718 filtered out; finished in 0.02s


```

### source toolchain closure marker tests

Command:

```sh
cargo test -p mantle --bin mantle source_toolchain_closure
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 36 tests
test source_toolchain_closure::tests::absent_closure_status_preserves_current_non_claim ... ok
test source_toolchain_closure::tests::enforcement_rejects_undeclared_host_rustc_path ... ok
test source_toolchain_closure::tests::enforcement_rejects_undeclared_host_pkg_config_path ... ok
test source_toolchain_closure::tests::enforcement_rejects_declared_path_with_digest_mismatch ... ok
test source_toolchain_closure::tests::enforcement_rejects_undeclared_host_linker_path ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_bad_receipt_schema ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_digest_mismatch_from_shell_observation ... ok
test source_toolchain_closure::tests::json_fixture_parses_seed_exception_and_counts_source_built_members ... ok
test source_toolchain_closure::tests::enforcement_accepts_declared_observed_toolchain_inputs ... ok
test source_toolchain_closure::tests::enforced_closure_status_still_keeps_claim_disabled_until_real_proof_lands ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_missing_target_rustlib_role ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_provenance ... ok
test source_toolchain_closure::tests::rust_source_provider_accepts_matching_receipt_payload ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_source_name ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_provider_receipt_digest_link_mismatch ... ok
test source_toolchain_closure::tests::policy_digest_changes_when_seed_exception_reason_changes ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_rustlib_outside_lib_dir ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_artifact_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_digest_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_with_prebuilt_step ... ok
test source_toolchain_closure::tests::validator_rejects_duplicate_member_identity ... ok
test source_toolchain_closure::tests::validator_rejects_invalid_digest_shape ... ok
test source_toolchain_closure::tests::validator_rejects_invalid_optional_receipt_on_seed_member ... ok
test source_toolchain_closure::tests::validated_closure_status_keeps_claim_disabled_until_enforcement_lands ... ok
test source_toolchain_closure::tests::validator_accepts_explicit_seed_exception ... ok
test source_toolchain_closure::tests::validator_allows_seed_reason_with_marker_letters_inside_larger_word ... ok
test source_toolchain_closure::tests::validator_rejects_invalid_optional_source_on_seed_member ... ok
test source_toolchain_closure::tests::seed_exception_policy_digest_is_order_independent_and_accounted ... ok
test source_toolchain_closure::tests::validator_rejects_missing_required_role ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_metadata_yields_stable_digest ... ok
test source_toolchain_closure::tests::validator_rejects_placeholder_seed_exception_reason ... ok
test source_toolchain_closure::tests::valid_manifest_yields_stable_order_independent_policy_digest ... ok
test source_toolchain_closure::tests::validator_rejects_seed_member_without_seed_exception ... ok
test source_toolchain_closure::tests::validator_rejects_source_built_member_without_receipt ... ok
test source_toolchain_closure::tests::validator_rejects_source_built_rustc_with_prebuilt_source_marker ... ok
test source_toolchain_closure::tests::validator_rejects_unverified_seed_exception_name ... ok

test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 716 filtered out; finished in 0.00s


```

### cairn validate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . 
```

Exit status: `0`

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}

```

### cairn tasks gate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root . 
```

Exit status: `0`

```text
{
  "change": "source-built-rust-seed-closure",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "73890128cc00428e8dbc99fee7b8754d239ba5a31c732567065f7f5ca028ba1f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "5309aff76590edb0d9e8a30b6e45b0ab9fd85a2f28bde9786a54c11269f3b1a8",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

