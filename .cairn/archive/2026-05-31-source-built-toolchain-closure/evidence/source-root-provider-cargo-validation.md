# Source-root provider cargo validation

Task-ID: I4-cargo-validation
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

This evidence records the focused Rust validation commands that exercise the changed source-root provider code after positive materialization evidence landed.

## Environment

```text
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
```

## Focused source-root provider tests

Command:

```text
cargo test -p mantle --bin mantle source_root_provider -- --nocapture 
```

Output:

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-b7a954f06b8bd29f)

running 23 tests
test source_root_provider::tests::available_parallelism_returns_bounded_value ... ok
test self_build::tests::source_root_provider_fails_stagex_eligibility ... ok
test source_root_provider::tests::extract_kind_coverage ... ok
test bootstrap_parity::tests::stagex_lineage_receipt_rejects_source_root_provider ... ok
test source_root_provider::tests::index_artifacts_rejects_missing_required ... ok
test source_root_provider::tests::directory_blake3_digest_changes_with_content ... ok
test release_cmd::tests::stagex_profile_rejects_source_root_provider ... ok
test source_root_provider::tests::retained_unprefixed_tools_are_recognized ... ok
test source_root_provider::tests::source_root_and_legacy_share_contract_metadata_constants ... ok
test source_root_provider::tests::directory_blake3_digest_is_deterministic ... ok
test source_root_provider::tests::normalize_drops_unwanted_components ... ok
test source_root_provider::tests::legacy_and_source_root_provider_json_share_required_fields ... ok
test source_root_provider::tests::validate_seed_contract_layout_rejects_missing_compiler ... ok
test source_root_provider::tests::source_root_has_source_root_metadata_field ... ok
test source_root_provider::tests::discover_emitted_roles_finds_all_contract_parts ... ok
test tests::source_root_provider_store_name_rejects_bad_digest ... ok
test source_root_provider::tests::source_root_retained_tools_match_required_roles ... ok
test source_root_provider::tests::source_root_provider_json_has_matching_metadata_fields ... ok
test source_root_provider::tests::source_root_provider_json_does_not_contain_legacy_raw_artifact ... ok
test tests::source_root_provider_store_name_uses_nix_store_hash_shape ... ok
test source_root_provider::tests::validate_seed_contract_layout_rejects_missing_sysroot_headers ... ok
test source_root_provider::tests::source_root_materialization_rejects_declared_patches_before_fetch ... ok
test tests::bootstrap_source_root_provider_reaches_materializer_and_fails_closed_on_patches ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 616 filtered out; finished in 0.00s


```

Exit status: `0`

## Mantle binary build

Command:

```text
cargo build -p mantle --bin mantle 
```

Output:

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: struct `NativeTextInput` is never constructed
    --> src/rust_plan.rs:1008:8
     |
1008 | struct NativeTextInput {
     |        ^^^^^^^^^^^^^^^
     |
     = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: struct `NativeManifestLockInputTexts` is never constructed
    --> src/rust_plan.rs:1014:8
     |
1014 | struct NativeManifestLockInputTexts {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockUnsupportedBlocker` is never constructed
    --> src/rust_plan.rs:1020:8
     |
1020 | struct NativeManifestLockUnsupportedBlocker {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolutionRequest` is never constructed
    --> src/rust_plan.rs:1044:8
     |
1044 | struct NativeFeatureRoleResolutionRequest {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolution` is never constructed
    --> src/rust_plan.rs:1051:8
     |
1051 | struct NativeFeatureRoleResolution {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `collect_native_manifest_lock_texts` is never used
    --> src/rust_plan.rs:3628:4
     |
3628 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestLockInputTexts, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3653:4
     |
3653 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInput, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3662:4
     |
3662 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3675:4
     |
3675 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3706:4
     |
3706 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3736:4
     |
3736 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3768:4
     |
3768 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3772:4
     |
3772 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> NativeManifestLockUnsupportedBlocker {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5042:4
     |
5042 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest) -> NativeFeatureRoleResolution {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "mantle") generated 14 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s

```

Exit status: `0`

## Cairn validation

Command:

```text
/home/brittonr/.cargo-target/debug/cairn validate --root . 
```

Output:

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}

```

Exit status: `0`

## Cairn tasks gate

Command:

```text
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root . 
```

Output:

```text
{
  "change": "source-built-toolchain-closure",
  "input_hash": "ba612bbb6ba4ae56813bd313113f8c582ae2ff725a2b1d3cbb42cbb942cc597e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0400cff3e3ab4adbe74ee82584fc420f33e3e6c41385e173d961c64775c0c039",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit status: `0`

## Post-task-pointer Cairn rerun

Command:

```text
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Output:

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "input_hash": "8667ed1beac374496ab07aec27408511d21445d6d6bd4f922c8e00c8383a385a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "552bc7147147b92bba9455268b1ccfbce964aa754f5b28289dd116525cab6e54",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```
