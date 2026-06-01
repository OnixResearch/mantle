# Implementation validation

Change: `examples-workflow-gallery`

This transcript validates the progressive examples gallery slice: local named-output layout, project workflow docs, trust/provenance non-claim wording, JSON artifact-attestation shape, and module-boundary guards for `examples/`.

## Focused examples workflow tests

Command source: pueue task `280` (`examples-workflow-gallery-tests-rerun`).

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on artifact directory
warning: struct `NativeTextInput` is never constructed
    --> src/rust_plan.rs:1013:8
     |
1013 | struct NativeTextInput {
     |        ^^^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: struct `NativeManifestLockInputTexts` is never constructed
    --> src/rust_plan.rs:1019:8
     |
1019 | struct NativeManifestLockInputTexts {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockUnsupportedBlocker` is never constructed
    --> src/rust_plan.rs:1025:8
     |
1025 | struct NativeManifestLockUnsupportedBlocker {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolutionRequest` is never constructed
    --> src/rust_plan.rs:1049:8
     |
1049 | struct NativeFeatureRoleResolutionRequest {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolution` is never constructed
    --> src/rust_plan.rs:1056:8
     |
1056 | struct NativeFeatureRoleResolution {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `collect_native_manifest_lock_texts` is never used
    --> src/rust_plan.rs:3633:4
     |
3633 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestLockInputTexts, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3658:4
     |
3658 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInput, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3667:4
     |
3667 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3680:4
     |
3680 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3711:4
     |
3711 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3741:4
     |
3741 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3773:4
     |
3773 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3777:4
     |
3777 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> NativeManifestLockUnsupportedBlocker {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5047:4
     |
5047 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest) -> NativeFeatureRoleResolution {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "crunch") generated 14 warnings
warning: `mantle` (bin "mantle") generated 14 warnings (14 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.88s
     Running tests/examples_eval.rs (/home/brittonr/.cargo-target/debug/deps/examples_eval-e76ea8f3fd84f122)

running 7 tests
test bootstrap_no_nix_example_uses_store_env_glob ... ok
test malformed_example_fails_before_export ... ok
test seed_dependent_example_without_seed_fails_loudly ... ok
test eval_fetch_crate_crc64_example ... ok
test eval_bootstrap_no_nix_example_uses_shared_seed ... ok
test eval_build_crate_crc64_example ... ok
test cataloged_eval_examples_export_json ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: struct `NativeTextInput` is never constructed
    --> src/rust_plan.rs:1013:8
     |
1013 | struct NativeTextInput {
     |        ^^^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: struct `NativeManifestLockInputTexts` is never constructed
    --> src/rust_plan.rs:1019:8
     |
1019 | struct NativeManifestLockInputTexts {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockUnsupportedBlocker` is never constructed
    --> src/rust_plan.rs:1025:8
     |
1025 | struct NativeManifestLockUnsupportedBlocker {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolutionRequest` is never constructed
    --> src/rust_plan.rs:1049:8
     |
1049 | struct NativeFeatureRoleResolutionRequest {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolution` is never constructed
    --> src/rust_plan.rs:1056:8
     |
1056 | struct NativeFeatureRoleResolution {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `collect_native_manifest_lock_texts` is never used
    --> src/rust_plan.rs:3633:4
     |
3633 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestLockInputTexts, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3658:4
     |
3658 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInput, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3667:4
     |
3667 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3680:4
     |
3680 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3711:4
     |
3711 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3741:4
     |
3741 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3773:4
     |
3773 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3777:4
     |
3777 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> NativeManifestLockUnsupportedBlocker {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5047:4
     |
5047 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest) -> NativeFeatureRoleResolution {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "crunch") generated 14 warnings
warning: `mantle` (bin "mantle") generated 14 warnings (14 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running tests/examples_build.rs (/home/brittonr/.cargo-target/debug/deps/examples_build-c329e6ab5fd8de21)

running 1 test
test local_output_layout_example_builds_named_outputs ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.20s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: struct `NativeTextInput` is never constructed
    --> src/rust_plan.rs:1013:8
     |
1013 | struct NativeTextInput {
     |        ^^^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: struct `NativeManifestLockInputTexts` is never constructed
    --> src/rust_plan.rs:1019:8
     |
1019 | struct NativeManifestLockInputTexts {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockUnsupportedBlocker` is never constructed
    --> src/rust_plan.rs:1025:8
     |
1025 | struct NativeManifestLockUnsupportedBlocker {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolutionRequest` is never constructed
    --> src/rust_plan.rs:1049:8
     |
1049 | struct NativeFeatureRoleResolutionRequest {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolution` is never constructed
    --> src/rust_plan.rs:1056:8
     |
1056 | struct NativeFeatureRoleResolution {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `collect_native_manifest_lock_texts` is never used
    --> src/rust_plan.rs:3633:4
     |
3633 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestLockInputTexts, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3658:4
     |
3658 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInput, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3667:4
     |
3667 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3680:4
     |
3680 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3711:4
     |
3711 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3741:4
     |
3741 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3773:4
     |
3773 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3777:4
     |
3777 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> NativeManifestLockUnsupportedBlocker {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5047:4
     |
5047 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest) -> NativeFeatureRoleResolution {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "mantle") generated 14 warnings
warning: `mantle` (bin "crunch") generated 14 warnings (14 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running tests/examples_build.rs (/home/brittonr/.cargo-target/debug/deps/examples_build-c329e6ab5fd8de21)

running 1 test
test hello_json_build_report_exposes_artifact_attestation_shape ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.21s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: struct `NativeTextInput` is never constructed
    --> src/rust_plan.rs:1013:8
     |
1013 | struct NativeTextInput {
     |        ^^^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: struct `NativeManifestLockInputTexts` is never constructed
    --> src/rust_plan.rs:1019:8
     |
1019 | struct NativeManifestLockInputTexts {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockUnsupportedBlocker` is never constructed
    --> src/rust_plan.rs:1025:8
     |
1025 | struct NativeManifestLockUnsupportedBlocker {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolutionRequest` is never constructed
    --> src/rust_plan.rs:1049:8
     |
1049 | struct NativeFeatureRoleResolutionRequest {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolution` is never constructed
    --> src/rust_plan.rs:1056:8
     |
1056 | struct NativeFeatureRoleResolution {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `collect_native_manifest_lock_texts` is never used
    --> src/rust_plan.rs:3633:4
     |
3633 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestLockInputTexts, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3658:4
     |
3658 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInput, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3667:4
     |
3667 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3680:4
     |
3680 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3711:4
     |
3711 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3741:4
     |
3741 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3773:4
     |
3773 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3777:4
     |
3777 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> NativeManifestLockUnsupportedBlocker {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5047:4
     |
5047 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest) -> NativeFeatureRoleResolution {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "crunch") generated 14 warnings
warning: `mantle` (bin "mantle") generated 14 warnings (14 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running tests/examples_inventory.rs (/home/brittonr/.cargo-target/debug/deps/examples_inventory-f099cf4a3bfa12d9)

running 10 tests
test examples_catalog_rejects_invalid_tier_and_silent_skip ... ok
test documented_crunch_ncl_identifier_is_allowed ... ok
test examples_catalog_rejects_stale_crunch_branding_in_user_docs ... ok
test examples_catalog_rejects_fetchers_without_offline_and_negative_rails ... ok
test examples_catalog_rejects_duplicate_ids_and_paths ... ok
test examples_catalog_rejects_readme_omissions_and_stale_paths ... ok
test examples_catalog_rejects_readme_lane_order_and_missing_project_outputs ... ok
test examples_catalog_rejects_trust_docs_without_non_claims ... ok
test examples_catalog_supports_lane_inventory_for_docs ... ok
test examples_catalog_covers_checked_in_user_facing_examples ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
warning: struct `NativeTextInput` is never constructed
    --> src/rust_plan.rs:1013:8
     |
1013 | struct NativeTextInput {
     |        ^^^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: struct `NativeManifestLockInputTexts` is never constructed
    --> src/rust_plan.rs:1019:8
     |
1019 | struct NativeManifestLockInputTexts {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockUnsupportedBlocker` is never constructed
    --> src/rust_plan.rs:1025:8
     |
1025 | struct NativeManifestLockUnsupportedBlocker {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolutionRequest` is never constructed
    --> src/rust_plan.rs:1049:8
     |
1049 | struct NativeFeatureRoleResolutionRequest {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolution` is never constructed
    --> src/rust_plan.rs:1056:8
     |
1056 | struct NativeFeatureRoleResolution {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `collect_native_manifest_lock_texts` is never used
    --> src/rust_plan.rs:3633:4
     |
3633 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestLockInputTexts, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3658:4
     |
3658 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInput, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3667:4
     |
3667 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3680:4
     |
3680 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3711:4
     |
3711 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3741:4
     |
3741 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3773:4
     |
3773 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3777:4
     |
3777 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> NativeManifestLockUnsupportedBlocker {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5047:4
     |
5047 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest) -> NativeFeatureRoleResolution {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "mantle") generated 14 warnings
warning: `mantle` (bin "crunch") generated 14 warnings (14 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.43s
     Running tests/removed_system_cli.rs (/home/brittonr/.cargo-target/debug/deps/removed_system_cli-6a5f74961fe9cfca)

running 6 tests
test system_eval_is_not_a_supported_subcommand ... ok
test help_succeeds_without_system_subcommand ... ok
test public_docs_and_stdlib_do_not_reference_system_eval_surface ... ok
test raw_module_inventory_is_rejected_as_build_plan_input ... ok
test external_frontend_can_hand_mantle_build_shaped_input ... ok
test implementation_surface_does_not_gain_module_layer_coupling ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s


```

Exit: `0`


## rustfmt --check tests/examples_build.rs tests/examples_inventory.rs tools/tracey_refs.rs

```text

```

Exit: `0`

## git diff --check

```text

```

Exit: `0`

## cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}

```

Exit: `0`

## cairn gate tasks examples-workflow-gallery --root .

```text
{
  "change": "examples-workflow-gallery",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ec2461a5ef3f4e3a68a5a9ef619c2e1da34ac2eda5e0e72c6a4ff0beb38017b0",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "bc3ad3d26e51b68dc8596cda24ba618707de11be308e4a1d574eef71b0d1877d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## post-final-task cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}

```

Exit: `0`

## post-final-task cairn gate tasks examples-workflow-gallery --root .

```text
{
  "change": "examples-workflow-gallery",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f9bde5341179446b156ce6f4273f66245f5763b40b54a9fdd6e15a5e09501d25",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b18761734593bf0e43612edc9334a9f44c7466552a08650461cc65ba9941a991",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`
