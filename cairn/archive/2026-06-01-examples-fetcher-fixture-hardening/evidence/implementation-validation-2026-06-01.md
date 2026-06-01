# Implementation validation

Change: `examples-fetcher-fixture-hardening`

This transcript validates generated offline fixtures for `fetchurl`, `fetchTarball`, and `fetchGit`; deterministic wrong-hash failures; `--fix` repair against a temp fixture; and catalog/docs pairing between real-network examples and offline validation rails.

## Focused examples fetcher tests

Command source: pueue task `255` (`examples-fetcher-offline-tests`).

```text
┌─────────────────────────────────────┐
│ Task 255:    completed successfully │
└─────────────────────────────────────┘
Command: export PATH="/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin" 
         export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=cc                                                                                                                                                                                                                                                                                                                                               
         export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig"                                                                                                                                                                                                                                                                                                 
         export SNIX_BUILD_SANDBOX_SHELL="/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox"                                                                                                                                                                                                                                                            
         cargo test -p mantle --test examples_build offline_fetch -- --nocapture && \                                                                                                                                                                                                                                                                                                                         
         cargo test -p mantle --test examples_build fix_flag_updates_temp_fetchurl_fixture_hash -- --nocapture && \                                                                                                                                                                                                                                                                                           
         cargo test -p mantle --test examples_inventory -- --nocapture                                                                                                                                                                                                                                                                                                                                        
   Path: /home/brittonr/git/mantle                                                                                                                                                                                                                                                                                                                                                                            
  Label: examples-fetcher-offline-tests                                                                                                                                                                                                                                                                                                                                                                       
  Start: Mon, 1 Jun 2026 18:07:39 -0400                                                                                                                                                                                                                                                                                                                                                                       
    End: Mon, 1 Jun 2026 18:07:43 -0400                                                                                                                                                                                                                                                                                                                                                                       

output: (last 15 lines)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.76s
     Running tests/examples_inventory.rs (/home/brittonr/.cargo-target/debug/deps/examples_inventory-f099cf4a3bfa12d9)

running 8 tests
test examples_catalog_rejects_stale_crunch_branding_in_user_docs ... ok
test examples_catalog_rejects_fetchers_without_offline_and_negative_rails ... ok
test documented_crunch_ncl_identifier_is_allowed ... ok
test examples_catalog_rejects_readme_omissions_and_stale_paths ... ok
test examples_catalog_rejects_duplicate_ids_and_paths ... ok
test examples_catalog_rejects_invalid_tier_and_silent_skip ... ok
test examples_catalog_supports_lane_inventory_for_docs ... ok
test examples_catalog_covers_checked_in_user_facing_examples ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


```

Exit: `0`

## rustfmt --check tests/examples_build.rs tests/examples_inventory.rs

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
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

```

Exit: `0`

## cairn gate tasks examples-fetcher-fixture-hardening --root .

```text
{
  "change": "examples-fetcher-fixture-hardening",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "52ebe4b13f30dbc217c1fa97bd324667192b350ce6aeda3374ccf8c7fc7ac8c7",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b375fcffe8d9df8fb8eef830e363b9e1a3811f9d6b839df12bf2a81bfe2b0c91",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`


## Full pueue log for focused examples fetcher tests

```text
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.59s
     Running tests/examples_build.rs (/home/brittonr/.cargo-target/debug/deps/examples_build-c329e6ab5fd8de21)

running 4 tests
test offline_fetchurl_fixture_builds_without_network ... ok
test offline_fetch_tarball_fixture_builds_without_network ... ok
test offline_fetchgit_fixture_builds_without_network ... ok
test offline_fetcher_wrong_hashes_fail_closed ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.68s

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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.35s
     Running tests/examples_build.rs (/home/brittonr/.cargo-target/debug/deps/examples_build-c329e6ab5fd8de21)

running 1 test
test fix_flag_updates_temp_fetchurl_fixture_hash ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.32s

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

   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
warning: `mantle` (bin "crunch") generated 14 warnings
warning: `mantle` (bin "mantle") generated 14 warnings (14 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.76s
     Running tests/examples_inventory.rs (/home/brittonr/.cargo-target/debug/deps/examples_inventory-f099cf4a3bfa12d9)

running 8 tests
test examples_catalog_rejects_stale_crunch_branding_in_user_docs ... ok
test examples_catalog_rejects_fetchers_without_offline_and_negative_rails ... ok
test documented_crunch_ncl_identifier_is_allowed ... ok
test examples_catalog_rejects_readme_omissions_and_stale_paths ... ok
test examples_catalog_rejects_duplicate_ids_and_paths ... ok
test examples_catalog_rejects_invalid_tier_and_silent_skip ... ok
test examples_catalog_supports_lane_inventory_for_docs ... ok
test examples_catalog_covers_checked_in_user_facing_examples ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


```

Exit: `0`

## post-final-task cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

```

Exit: `0`

## post-final-task cairn gate tasks examples-fetcher-fixture-hardening --root .

```text
{
  "change": "examples-fetcher-fixture-hardening",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "bf4a38430a9736a2a91cb88cb95595da10b9385b74a88cb4c2f3c51c0752d660",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "01625f7c7ddd33752877722bc559f0d6d1e52c1f3905bfb840463d4bf09b3cd2",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## final pre-commit rustfmt --check tests/examples_build.rs tests/examples_inventory.rs tools/tracey_refs.rs

```text

```

Exit: `0`

## final pre-commit git diff --check

```text

```

Exit: `0`

## final pre-commit cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

```

Exit: `0`

## final pre-commit cairn gate tasks examples-fetcher-fixture-hardening --root .

```text
{
  "change": "examples-fetcher-fixture-hardening",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "bf4a38430a9736a2a91cb88cb95595da10b9385b74a88cb4c2f3c51c0752d660",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "01625f7c7ddd33752877722bc559f0d6d1e52c1f3905bfb840463d4bf09b3cd2",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`
