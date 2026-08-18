# Rust provider cargo-free threading

Task-ID: rust-provider-cargo-free-threading-2026-06-10
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now accepts `self-build --cargo-free --rust-source-provider <dir>`. The cargo-free self-build shell validates the provider directory with the existing source-built Rust provider validator, derives `bin/rustc` from validated provider metadata, and feeds that rustc into both one-shot and fixed-point cargo-free execution. JSON summaries and fixed-point preflight now include a `rust_source_provider` binding record with metadata and policy digests.

This does not materialize a real Rust-from-source provider and does not remove `not-source-built-toolchain-closure`; it only wires the future validated provider boundary into cargo-free proof commands.

## Commands and output

### provider binding tests

Command:

```sh
cargo test -p mantle --bin mantle rust_source_provider_binding
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 2 tests
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 744 filtered out; finished in 0.00s


```

### fixed-point cargo-free summary tests

Command:

```sh
cargo test -p mantle --bin mantle fixed_point
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 12 tests
test cargo_free_self_build::tests::fixed_point_plan_rejects_bundle_inside_source_root ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_relative_source_root ... ok
test cargo_free_self_build::tests::fixed_point_plan_threads_targets_into_stage_commands ... ok
test cargo_free_self_build::tests::fixed_point_status_accepts_matching_policy_digest ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_presence_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_plan_describes_stage_paths_and_commands ... ok
test rust_plan::tests::native_feature_resolver_reaches_fixed_point_for_defaults_explicit_and_optional_dependencies ... ok
test tests::self_build_cli_rejects_fixed_point_without_cargo_free ... ok
test tests::self_build_cli_accepts_cargo_free_fixed_point_out_dir ... ok
test tests::cargo_free_fixed_point_rejects_legacy_options ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 734 filtered out; finished in 0.00s


```

### self-build CLI provider option tests

Command:

```sh
cargo test -p mantle --bin mantle self_build_cli_
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 9 tests
test tests::self_build_cli_rejects_rust_source_provider_without_cargo_free ... ok
test tests::self_build_cli_rejects_fixed_point_without_cargo_free ... ok
test tests::self_build_cli_rejects_toolchain_closure_without_cargo_free ... ok
test tests::self_build_cli_accepts_impure_mode ... ok
test tests::self_build_cli_accepts_cargo_free_toolchain_closure_manifest ... ok
test tests::self_build_cli_accepts_cargo_free_out_dir ... ok
test tests::self_build_cli_accepts_cargo_free_target_triple ... ok
test tests::self_build_cli_accepts_cargo_free_rust_source_provider ... ok
test tests::self_build_cli_accepts_cargo_free_fixed_point_out_dir ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 737 filtered out; finished in 0.00s


```

### provider validation and smoke tests

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

running 28 tests
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_provenance ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_source_name ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_missing_target_rustlib_role ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_provider_receipt_digest_link_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_bad_receipt_schema ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_digest_mismatch_from_shell_observation ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_rustlib_outside_lib_dir ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_digest_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_artifact_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_with_prebuilt_step ... ok
test source_toolchain_closure::tests::rust_source_provider_accepts_matching_receipt_payload ... ok
test rust_source_provider::tests::materializer_fails_closed_without_claiming_prebuilt_rust ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_metadata_yields_stable_digest ... ok
test tests::bootstrap_rust_source_provider_fails_closed_without_output ... ok
test rust_source_provider::tests::directory_validator_rejects_artifact_digest_mismatch ... ok
test rust_source_provider::tests::import_provider_rejects_existing_output_without_overwrite ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok
test rust_source_provider::tests::import_provider_rejects_prebuilt_metadata_without_output ... ok
test rust_source_provider::tests::directory_validator_accepts_complete_fake_provider ... ok
test rust_source_provider::tests::import_provider_rejects_malformed_receipt_without_output ... ok
test tests::self_build_cli_rejects_rust_source_provider_without_cargo_free ... ok
test tests::self_build_cli_accepts_cargo_free_rust_source_provider ... ok
test tests::bootstrap_rust_source_provider_action_parses_import_dir ... ok
test tests::bootstrap_rust_source_provider_action_parses ... ok
test rust_source_provider::tests::smoke_provider_rejects_invalid_metadata_before_launch ... ok
test rust_source_provider::tests::smoke_provider_runs_synthetic_rustc_after_validation ... ok
test rust_source_provider::tests::import_provider_copies_validated_provider ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 718 filtered out; finished in 0.02s


```

## Lifecycle validation after task update

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
  "input_hash": "a5500f6089cccbd714ac5de470a6fb7de7b6eab24b78a0638b3938874503f0d1",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c8d60e9879faf45575897f39cee4ea309e47c6cf84aba48ad5b97ac560f93d15",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

## Post-refactor focused validation

### provider binding tests after fixture refactor

Command:

```sh
cargo test -p mantle --bin mantle rust_source_provider_binding
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 2 tests
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 744 filtered out; finished in 0.00s


```

### fixed-point tests after fixture refactor

Command:

```sh
cargo test -p mantle --bin mantle fixed_point
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 12 tests
test cargo_free_self_build::tests::fixed_point_plan_rejects_relative_source_root ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_bundle_inside_source_root ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_presence_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_plan_threads_targets_into_stage_commands ... ok
test cargo_free_self_build::tests::fixed_point_status_accepts_matching_policy_digest ... ok
test cargo_free_self_build::tests::fixed_point_plan_describes_stage_paths_and_commands ... ok
test cargo_free_self_build::tests::fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches ... ok
test rust_plan::tests::native_feature_resolver_reaches_fixed_point_for_defaults_explicit_and_optional_dependencies ... ok
test tests::self_build_cli_accepts_cargo_free_fixed_point_out_dir ... ok
test tests::cargo_free_fixed_point_rejects_legacy_options ... ok
test tests::self_build_cli_rejects_fixed_point_without_cargo_free ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 734 filtered out; finished in 0.00s


```

## Final lifecycle validation

### final cairn validate

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

### final cairn tasks gate

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
  "input_hash": "b5008bfd9488a237231694640c00b055641093a512ab795ff59a3be0e7f73514",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "1a50a8c1379932f7351c2291dbb1515d205f6411cd871479c67612ef8fc7f806",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

## Final focused validation after provider-selection test

### final provider and CLI/provider validation tests

Command:

```sh
cargo test -p mantle --bin mantle rust_source_provider
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 29 tests
test cargo_free_self_build::tests::rust_source_provider_selection_prefers_validated_provider_and_keeps_fallback ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_provenance ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_prebuilt_source_name ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_missing_target_rustlib_role ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_provider_receipt_digest_link_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_bad_receipt_schema ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_digest_mismatch_from_shell_observation ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_digest_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_artifact_mismatch ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_receipt_with_prebuilt_step ... ok
test rust_source_provider::tests::materializer_fails_closed_without_claiming_prebuilt_rust ... ok
test source_toolchain_closure::tests::rust_source_provider_rejects_rustlib_outside_lib_dir ... ok
test source_toolchain_closure::tests::rust_source_provider_accepts_matching_receipt_payload ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_metadata_yields_stable_digest ... ok
test tests::bootstrap_rust_source_provider_fails_closed_without_output ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_rejects_prebuilt_provider_metadata ... ok
test rust_source_provider::tests::import_provider_rejects_existing_output_without_overwrite ... ok
test rust_source_provider::tests::directory_validator_rejects_artifact_digest_mismatch ... ok
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok
test rust_source_provider::tests::directory_validator_accepts_complete_fake_provider ... ok
test rust_source_provider::tests::import_provider_rejects_malformed_receipt_without_output ... ok
test rust_source_provider::tests::smoke_provider_rejects_invalid_metadata_before_launch ... ok
test tests::bootstrap_rust_source_provider_action_parses ... ok
test tests::self_build_cli_accepts_cargo_free_rust_source_provider ... ok
test tests::bootstrap_rust_source_provider_action_parses_import_dir ... ok
test tests::self_build_cli_rejects_rust_source_provider_without_cargo_free ... ok
test rust_source_provider::tests::import_provider_rejects_prebuilt_metadata_without_output ... ok
test rust_source_provider::tests::smoke_provider_runs_synthetic_rustc_after_validation ... ok
test rust_source_provider::tests::import_provider_copies_validated_provider ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 718 filtered out; finished in 0.02s


```

### final fixed-point proof plumbing tests

Command:

```sh
cargo test -p mantle --bin mantle fixed_point
```

Exit status: `0`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 12 tests
test cargo_free_self_build::tests::fixed_point_plan_rejects_bundle_inside_source_root ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_relative_source_root ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_presence_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_status_accepts_matching_policy_digest ... ok
test cargo_free_self_build::tests::fixed_point_plan_describes_stage_paths_and_commands ... ok
test cargo_free_self_build::tests::fixed_point_plan_threads_targets_into_stage_commands ... ok
test cargo_free_self_build::tests::fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches ... ok
test rust_plan::tests::native_feature_resolver_reaches_fixed_point_for_defaults_explicit_and_optional_dependencies ... ok
test tests::self_build_cli_rejects_fixed_point_without_cargo_free ... ok
test tests::self_build_cli_accepts_cargo_free_fixed_point_out_dir ... ok
test tests::cargo_free_fixed_point_rejects_legacy_options ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 735 filtered out; finished in 0.00s


```

### final cairn validate after focused tests

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

### final cairn tasks gate after focused tests

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
  "input_hash": "b5008bfd9488a237231694640c00b055641093a512ab795ff59a3be0e7f73514",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "1a50a8c1379932f7351c2291dbb1515d205f6411cd871479c67612ef8fc7f806",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

