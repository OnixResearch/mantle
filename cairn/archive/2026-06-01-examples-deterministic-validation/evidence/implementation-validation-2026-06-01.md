# Implementation validation

Change: `examples-deterministic-validation`

This transcript validates catalog-driven eval coverage, negative eval/project diagnostics, fast build smoke tests, output inspection, multi-output fixture layout, project check fixture output, and heavy-example ignored-test preservation.

## rustfmt --check tests/examples_eval.rs tests/examples_build.rs

```text

```

Exit: `0`

## cargo test -p mantle --test examples_eval --test examples_build -- --nocapture

```text
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

running 8 tests
test build_crate_crc64_example_builds_binary ... ignored, heavy bootstrap build; run explicitly when validating the real crate example
test project_missing_selector_fails_before_build_success ... ok
test fail_example_reports_expected_failure ... ok
test hello_example_builds_flat_output ... ok
test multi_step_example_builds_structured_output ... ok
test project_check_fixture_builds_result_output ... ok
test local_multi_output_fixture_builds_named_layout ... ok
test fetch_crate_crc64_example_builds ... ok

test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.21s

     Running tests/examples_eval.rs (/home/brittonr/.cargo-target/debug/deps/examples_eval-e76ea8f3fd84f122)

running 7 tests
test bootstrap_no_nix_example_uses_store_env_glob ... ok
test malformed_example_fails_before_export ... ok
test seed_dependent_example_without_seed_fails_loudly ... ok
test eval_fetch_crate_crc64_example ... ok
test eval_bootstrap_no_nix_example_uses_shared_seed ... ok
test eval_build_crate_crc64_example ... ok
test cataloged_eval_examples_export_json ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s


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
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 9,
  "valid": true
}

```

Exit: `0`

## cairn gate tasks examples-deterministic-validation --root .

```text
{
  "change": "examples-deterministic-validation",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "d9cb6210457c8e361432cc0117578f028f861f75c6a7f8a38a52e2e5489387d9",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "db6973efbfe519621d2318f5c6fa838ca56e628dc7c6e9afb156fb3097eb7b06",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## post-task cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 9,
  "valid": true
}

```

Exit: `0`

## post-task cairn gate tasks examples-deterministic-validation --root .

```text
{
  "change": "examples-deterministic-validation",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ed6e73ea8e1f93a8ebf62c732da9a768bc0b93b30702724ec44b95f1a8f89a4d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "963f6b62293b76df2b534075e2c1986826f0af03f893d7220980e8b9d87c6d5d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

