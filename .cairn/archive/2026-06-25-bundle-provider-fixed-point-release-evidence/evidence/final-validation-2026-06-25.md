# Final validation: bundle provider fixed-point release evidence

Date: 2026-06-25
Change: bundle-provider-fixed-point-release-evidence

This transcript records focused implementation validation for r[rust_package_planning.bundle_provider_fixed_point_release_evidence].


## env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-release-core --lib manifest::tests::

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_release_core-dbe793df6821936f)

running 11 tests
test manifest::tests::extract_full_proof_identity_fields_rejects_wrong_schema ... ok
test manifest::tests::release_manifest_rejects_unknown_selected_provider_kind ... ok
test manifest::tests::validate_rejects_absolute_member_path ... ok
test manifest::tests::validate_rejects_prerequisite_inventory_linkage_mismatch ... ok
test manifest::tests::canonical_bytes_are_stable_and_compact ... ok
test manifest::tests::validate_rejects_provider_fixed_point_proof_with_wrong_role ... ok
test manifest::tests::validate_accepts_provider_fixed_point_proof_artifact ... ok
test manifest::tests::extract_full_proof_identity_fields_accepts_valid_manifest ... ok
test manifest::tests::validate_rejects_stage2_digest_not_present_in_binaries ... ok
test manifest::tests::extract_full_proof_identity_fields_rejects_missing_provider_kind ... ok
test manifest::tests::validate_rejects_provider_fixed_point_proof_file_kind ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out; finished in 0.00s


exit_status=0
```

## env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --bin crunch release_evidence::tests::

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.31s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 12 tests
test release_evidence::tests::validate_rejects_absolute_member_path ... ok
test release_evidence::tests::validate_rejects_prerequisite_inventory_linkage_mismatch ... ok
test release_evidence::tests::validate_rejects_stage2_digest_not_present_in_binaries ... ok
test release_evidence::tests::canonical_bytes_are_stable_and_compact ... ok
test release_evidence::tests::load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact ... ok
test release_evidence::tests::load_full_self_hosting_proof_identity_accepts_full_proof_bundle ... ok
test release_evidence::tests::create_rejects_invalid_provider_fixed_point_proof_before_manifest ... ok
test release_evidence::tests::verify_rejects_tampered_binary_artifact ... ok
test release_evidence::tests::create_and_verify_release_bundle_round_trip ... ok
test release_evidence::tests::verify_rejects_non_canonical_manifest_json ... ok
test release_evidence::tests::verify_rejects_provider_kind_linkage_mismatch ... ok
test release_evidence::tests::create_and_verify_release_bundle_with_provider_fixed_point_proof ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 830 filtered out; finished in 0.01s


exit_status=0
```

## env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --bin crunch release_cmd::tests::provider_fixed_point_request_absent

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 2 tests
test release_cmd::tests::provider_fixed_point_request_absent_is_optional_by_default ... ok
test release_cmd::tests::provider_fixed_point_request_absent_records_required_blocker ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 840 filtered out; finished in 0.00s


exit_status=0
```

## env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --bin crunch release_create_accepts_provider_fixed_point_proof_flag

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 1 test
test tests::release_create_accepts_provider_fixed_point_proof_flag ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 841 filtered out; finished in 0.00s


exit_status=0
```

## env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --test release_cli provider_fixed_point

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: constant `MUSL_TARGET_GCC_ALIAS` is never used
  --> src/cargo_free_self_build.rs:38:7
   |
38 | const MUSL_TARGET_GCC_ALIAS: &str = "x86_64-linux-musl-gcc";
   |       ^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: constant `FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3` is never used
 --> src/frontend_artifact_spec.rs:7:11
  |
7 | pub const FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3: &str = "blake3";
  |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1` is never used
 --> src/frontend_artifact_spec.rs:8:11
  |
8 | pub const FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1: &str = "manifest-kind-...
  |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_KIND_ALLOWLIST_SCHEMA_V1` is never used
 --> src/frontend_artifact_spec.rs:9:11
  |
9 | pub const FRONTEND_ARTIFACT_KIND_ALLOWLIST_SCHEMA_V1: &str = "mantle-frontend-a...
  |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_ADMISSION_SIDECAR_SCHEMA` is never used
  --> src/frontend_artifact_spec.rs:10:11
   |
10 | pub const FRONTEND_ARTIFACT_ADMISSION_SIDECAR_SCHEMA: &str = "mantle-frontend-...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_MISSING_SPEC` is never used
  --> src/frontend_artifact_spec.rs:11:11
   |
11 | pub const FRONTEND_ARTIFACT_DIAG_MISSING_SPEC: &str = "frontend-artifact-missi...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_EMPTY_FIELD` is never used
  --> src/frontend_artifact_spec.rs:12:11
   |
12 | pub const FRONTEND_ARTIFACT_DIAG_EMPTY_FIELD: &str = "frontend-artifact-empty-...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_HASH` is never used
  --> src/frontend_artifact_spec.rs:13:11
   |
13 | pub const FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_HASH: &str = "frontend-artifact-u...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_INVALID_HASH` is never used
  --> src/frontend_artifact_spec.rs:14:11
   |
14 | pub const FRONTEND_ARTIFACT_DIAG_INVALID_HASH: &str = "frontend-artifact-inval...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_HASH_MISMATCH` is never used
  --> src/frontend_artifact_spec.rs:15:11
   |
15 | pub const FRONTEND_ARTIFACT_DIAG_HASH_MISMATCH: &str = "frontend-artifact-hash...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_VALIDATOR` is never used
  --> src/frontend_artifact_spec.rs:16:11
   |
16 | pub const FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_VALIDATOR: &str = "frontend-artif...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH` is never used
  --> src/frontend_artifact_spec.rs:17:11
   |
17 | pub const FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH: &str = "frontend-artif...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_HIDDEN_FALLBACK` is never used
  --> src/frontend_artifact_spec.rs:18:11
   |
18 | pub const FRONTEND_ARTIFACT_DIAG_HIDDEN_FALLBACK: &str = "frontend-artifact-hi...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL` is never used
  --> src/frontend_artifact_spec.rs:19:11
   |
19 | pub const FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL: &str = "frontend-artif...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_KIND_NOT_ALLOWED` is never used
  --> src/frontend_artifact_spec.rs:20:11
   |
20 | pub const FRONTEND_ARTIFACT_DIAG_KIND_NOT_ALLOWED: &str = "frontend-artifact-k...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `HEX_CHARS_PER_BYTE` is never used
  --> src/frontend_artifact_spec.rs:22:7
   |
22 | const HEX_CHARS_PER_BYTE: usize = 2;
   |       ^^^^^^^^^^^^^^^^^^

warning: constant `BLAKE3_HEX_LENGTH` is never used
  --> src/frontend_artifact_spec.rs:23:7
   |
23 | const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
   |       ^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactSpecRef` is never constructed
  --> src/frontend_artifact_spec.rs:26:12
   |
26 | pub struct FrontendArtifactSpecRef {
   |            ^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactManifest` is never constructed
  --> src/frontend_artifact_spec.rs:37:12
   |
37 | pub struct FrontendArtifactManifest {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactAdmissionRequest` is never constructed
  --> src/frontend_artifact_spec.rs:50:12
   |
50 | pub struct FrontendArtifactAdmissionRequest<'a> {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactAdmissionDiagnostic` is never constructed
  --> src/frontend_artifact_spec.rs:59:12
   |
59 | pub struct FrontendArtifactAdmissionDiagnostic {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactAdmissionSidecar` is never constructed
  --> src/frontend_artifact_spec.rs:83:12
   |
83 | pub struct FrontendArtifactAdmissionSidecar {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactKindAllowlistSpec` is never constructed
  --> src/frontend_artifact_spec.rs:89:8
   |
89 | struct FrontendArtifactKindAllowlistSpec {
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactAdmissionReport` is never constructed
  --> src/frontend_artifact_spec.rs:95:12
   |
95 | pub struct FrontendArtifactAdmissionReport {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `admit_frontend_artifact` is never used
   --> src/frontend_artifact_spec.rs:101:8
    |
101 | pub fn admit_frontend_artifact(request: &FrontendArtifactAdmissionRequest<'_>...
    |        ^^^^^^^^^^^^^^^^^^^^^^^

warning: function `render_frontend_artifact_admission_sidecar` is never used
   --> src/frontend_artifact_spec.rs:142:8
    |
142 | pub fn render_frontend_artifact_admission_sidecar(
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `report` is never used
   --> src/frontend_artifact_spec.rs:151:4
    |
151 | fn report(
    |    ^^^^^^

warning: function `validate_spec_ref` is never used
   --> src/frontend_artifact_spec.rs:162:4
    |
162 | fn validate_spec_ref(
    |    ^^^^^^^^^^^^^^^^^

warning: function `validate_blake3_hash` is never used
   --> src/frontend_artifact_spec.rs:186:4
    |
186 | fn validate_blake3_hash(
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `execute_declared_validator` is never used
   --> src/frontend_artifact_spec.rs:217:4
    |
217 | fn execute_declared_validator(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `execute_kind_allowlist_validator` is never used
   --> src/frontend_artifact_spec.rs:235:4
    |
235 | fn execute_kind_allowlist_validator(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `validate_manifest_binding` is never used
   --> src/frontend_artifact_spec.rs:278:4
    |
278 | fn validate_manifest_binding(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `require_non_empty` is never used
   --> src/frontend_artifact_spec.rs:319:4
    |
319 | fn require_non_empty(value: &str, path: &'static str, diagnostics: &mut Vec<F...
    |    ^^^^^^^^^^^^^^^^^

warning: function `is_blake3_hex` is never used
   --> src/frontend_artifact_spec.rs:329:4
    |
329 | fn is_blake3_hex(value: &str) -> bool {
    |    ^^^^^^^^^^^^^

warning: function `diagnostic` is never used
   --> src/frontend_artifact_spec.rs:333:4
    |
333 | fn diagnostic(
    |    ^^^^^^^^^^

warning: function `validate_native_root_capability` is never used
   --> src/native_toolchain_closure.rs:231:4
    |
231 | fn validate_native_root_capability(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `GNU_COMPILER_HOST_TRIPLE` is never used
  --> src/rust_bootstrap_patch_plan.rs:13:18
   |
13 | pub(crate) const GNU_COMPILER_HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu";
   |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: method `contains_operation` is never used
   --> src/rust_bootstrap_patch_plan.rs:166:19
    |
144 | impl RustBootstrapPatchPlan {
    | --------------------------- method in this implementation
...
166 |     pub(crate) fn contains_operation(&self, kind: RustBootstrapPatchOperation...
    |                   ^^^^^^^^^^^^^^^^^^

warning: struct `NativeTextInput` is never constructed
    --> src/rust_plan.rs:1192:8
     |
1192 | struct NativeTextInput {
     |        ^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockInputTexts` is never constructed
    --> src/rust_plan.rs:1198:8
     |
1198 | struct NativeManifestLockInputTexts {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockUnsupportedBlocker` is never constructed
    --> src/rust_plan.rs:1204:8
     |
1204 | struct NativeManifestLockUnsupportedBlocker {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolutionRequest` is never constructed
    --> src/rust_plan.rs:1228:8
     |
1228 | struct NativeFeatureRoleResolutionRequest {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolution` is never constructed
    --> src/rust_plan.rs:1235:8
     |
1235 | struct NativeFeatureRoleResolution {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `collect_native_manifest_lock_texts` is never used
    --> src/rust_plan.rs:3812:4
     |
3812 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestL...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3837:4
     |
3837 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInpu...
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3846:4
     |
3846 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3859:4
     |
3859 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<Nati...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3890:4
     |
3890 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3920:4
     |
3920 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<Nati...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3952:4
     |
3952 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3956:4
     |
3956 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> N...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5226:4
     |
5226 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest)...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "crunch") generated 52 warnings
warning: `mantle` (bin "mantle") generated 52 warnings (52 duplicates)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running tests/release_cli.rs (/home/brittonr/.cargo-target/debug/deps/release_cli-adc6653fd83c9572)

running 5 tests
test release_create_rejects_invalid_provider_fixed_point_proof ... ok
test release_create_can_package_provider_fixed_point_proof ... ok
test release_verify_required_provider_fixed_point_fails_when_missing ... ok
test release_verify_required_provider_fixed_point_uses_bundled_proof ... ok
test release_verify_provider_fixed_point_external_override_reports_source ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out; finished in 0.03s


exit_status=0
```

## env SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo build -p mantle --bin mantle

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: constant `MUSL_TARGET_GCC_ALIAS` is never used
  --> src/cargo_free_self_build.rs:38:7
   |
38 | const MUSL_TARGET_GCC_ALIAS: &str = "x86_64-linux-musl-gcc";
   |       ^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: constant `FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3` is never used
 --> src/frontend_artifact_spec.rs:7:11
  |
7 | pub const FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3: &str = "blake3";
  |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1` is never used
 --> src/frontend_artifact_spec.rs:8:11
  |
8 | pub const FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1: &str = "manifest-kind-...
  |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_KIND_ALLOWLIST_SCHEMA_V1` is never used
 --> src/frontend_artifact_spec.rs:9:11
  |
9 | pub const FRONTEND_ARTIFACT_KIND_ALLOWLIST_SCHEMA_V1: &str = "mantle-frontend-a...
  |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_ADMISSION_SIDECAR_SCHEMA` is never used
  --> src/frontend_artifact_spec.rs:10:11
   |
10 | pub const FRONTEND_ARTIFACT_ADMISSION_SIDECAR_SCHEMA: &str = "mantle-frontend-...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_MISSING_SPEC` is never used
  --> src/frontend_artifact_spec.rs:11:11
   |
11 | pub const FRONTEND_ARTIFACT_DIAG_MISSING_SPEC: &str = "frontend-artifact-missi...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_EMPTY_FIELD` is never used
  --> src/frontend_artifact_spec.rs:12:11
   |
12 | pub const FRONTEND_ARTIFACT_DIAG_EMPTY_FIELD: &str = "frontend-artifact-empty-...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_HASH` is never used
  --> src/frontend_artifact_spec.rs:13:11
   |
13 | pub const FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_HASH: &str = "frontend-artifact-u...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_INVALID_HASH` is never used
  --> src/frontend_artifact_spec.rs:14:11
   |
14 | pub const FRONTEND_ARTIFACT_DIAG_INVALID_HASH: &str = "frontend-artifact-inval...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_HASH_MISMATCH` is never used
  --> src/frontend_artifact_spec.rs:15:11
   |
15 | pub const FRONTEND_ARTIFACT_DIAG_HASH_MISMATCH: &str = "frontend-artifact-hash...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_VALIDATOR` is never used
  --> src/frontend_artifact_spec.rs:16:11
   |
16 | pub const FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_VALIDATOR: &str = "frontend-artif...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH` is never used
  --> src/frontend_artifact_spec.rs:17:11
   |
17 | pub const FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH: &str = "frontend-artif...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_HIDDEN_FALLBACK` is never used
  --> src/frontend_artifact_spec.rs:18:11
   |
18 | pub const FRONTEND_ARTIFACT_DIAG_HIDDEN_FALLBACK: &str = "frontend-artifact-hi...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL` is never used
  --> src/frontend_artifact_spec.rs:19:11
   |
19 | pub const FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL: &str = "frontend-artif...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_KIND_NOT_ALLOWED` is never used
  --> src/frontend_artifact_spec.rs:20:11
   |
20 | pub const FRONTEND_ARTIFACT_DIAG_KIND_NOT_ALLOWED: &str = "frontend-artifact-k...
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `HEX_CHARS_PER_BYTE` is never used
  --> src/frontend_artifact_spec.rs:22:7
   |
22 | const HEX_CHARS_PER_BYTE: usize = 2;
   |       ^^^^^^^^^^^^^^^^^^

warning: constant `BLAKE3_HEX_LENGTH` is never used
  --> src/frontend_artifact_spec.rs:23:7
   |
23 | const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
   |       ^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactSpecRef` is never constructed
  --> src/frontend_artifact_spec.rs:26:12
   |
26 | pub struct FrontendArtifactSpecRef {
   |            ^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactManifest` is never constructed
  --> src/frontend_artifact_spec.rs:37:12
   |
37 | pub struct FrontendArtifactManifest {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactAdmissionRequest` is never constructed
  --> src/frontend_artifact_spec.rs:50:12
   |
50 | pub struct FrontendArtifactAdmissionRequest<'a> {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactAdmissionDiagnostic` is never constructed
  --> src/frontend_artifact_spec.rs:59:12
   |
59 | pub struct FrontendArtifactAdmissionDiagnostic {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactAdmissionSidecar` is never constructed
  --> src/frontend_artifact_spec.rs:83:12
   |
83 | pub struct FrontendArtifactAdmissionSidecar {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactKindAllowlistSpec` is never constructed
  --> src/frontend_artifact_spec.rs:89:8
   |
89 | struct FrontendArtifactKindAllowlistSpec {
   |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `FrontendArtifactAdmissionReport` is never constructed
  --> src/frontend_artifact_spec.rs:95:12
   |
95 | pub struct FrontendArtifactAdmissionReport {
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `admit_frontend_artifact` is never used
   --> src/frontend_artifact_spec.rs:101:8
    |
101 | pub fn admit_frontend_artifact(request: &FrontendArtifactAdmissionRequest<'_>...
    |        ^^^^^^^^^^^^^^^^^^^^^^^

warning: function `render_frontend_artifact_admission_sidecar` is never used
   --> src/frontend_artifact_spec.rs:142:8
    |
142 | pub fn render_frontend_artifact_admission_sidecar(
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `report` is never used
   --> src/frontend_artifact_spec.rs:151:4
    |
151 | fn report(
    |    ^^^^^^

warning: function `validate_spec_ref` is never used
   --> src/frontend_artifact_spec.rs:162:4
    |
162 | fn validate_spec_ref(
    |    ^^^^^^^^^^^^^^^^^

warning: function `validate_blake3_hash` is never used
   --> src/frontend_artifact_spec.rs:186:4
    |
186 | fn validate_blake3_hash(
    |    ^^^^^^^^^^^^^^^^^^^^

warning: function `execute_declared_validator` is never used
   --> src/frontend_artifact_spec.rs:217:4
    |
217 | fn execute_declared_validator(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `execute_kind_allowlist_validator` is never used
   --> src/frontend_artifact_spec.rs:235:4
    |
235 | fn execute_kind_allowlist_validator(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `validate_manifest_binding` is never used
   --> src/frontend_artifact_spec.rs:278:4
    |
278 | fn validate_manifest_binding(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `require_non_empty` is never used
   --> src/frontend_artifact_spec.rs:319:4
    |
319 | fn require_non_empty(value: &str, path: &'static str, diagnostics: &mut Vec<F...
    |    ^^^^^^^^^^^^^^^^^

warning: function `is_blake3_hex` is never used
   --> src/frontend_artifact_spec.rs:329:4
    |
329 | fn is_blake3_hex(value: &str) -> bool {
    |    ^^^^^^^^^^^^^

warning: function `diagnostic` is never used
   --> src/frontend_artifact_spec.rs:333:4
    |
333 | fn diagnostic(
    |    ^^^^^^^^^^

warning: function `validate_native_root_capability` is never used
   --> src/native_toolchain_closure.rs:231:4
    |
231 | fn validate_native_root_capability(
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `GNU_COMPILER_HOST_TRIPLE` is never used
  --> src/rust_bootstrap_patch_plan.rs:13:18
   |
13 | pub(crate) const GNU_COMPILER_HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu";
   |                  ^^^^^^^^^^^^^^^^^^^^^^^^

warning: method `contains_operation` is never used
   --> src/rust_bootstrap_patch_plan.rs:166:19
    |
144 | impl RustBootstrapPatchPlan {
    | --------------------------- method in this implementation
...
166 |     pub(crate) fn contains_operation(&self, kind: RustBootstrapPatchOperation...
    |                   ^^^^^^^^^^^^^^^^^^

warning: struct `NativeTextInput` is never constructed
    --> src/rust_plan.rs:1192:8
     |
1192 | struct NativeTextInput {
     |        ^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockInputTexts` is never constructed
    --> src/rust_plan.rs:1198:8
     |
1198 | struct NativeManifestLockInputTexts {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockUnsupportedBlocker` is never constructed
    --> src/rust_plan.rs:1204:8
     |
1204 | struct NativeManifestLockUnsupportedBlocker {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolutionRequest` is never constructed
    --> src/rust_plan.rs:1228:8
     |
1228 | struct NativeFeatureRoleResolutionRequest {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolution` is never constructed
    --> src/rust_plan.rs:1235:8
     |
1235 | struct NativeFeatureRoleResolution {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `collect_native_manifest_lock_texts` is never used
    --> src/rust_plan.rs:3812:4
     |
3812 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestL...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3837:4
     |
3837 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInpu...
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3846:4
     |
3846 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3859:4
     |
3859 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<Nati...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3890:4
     |
3890 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3920:4
     |
3920 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<Nati...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3952:4
     |
3952 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3956:4
     |
3956 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> N...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5226:4
     |
5226 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest)...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "mantle") generated 52 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s

exit_status=0
```

## git diff --check

```text

exit_status=0
```

## nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle

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

exit_status=0
```

## nix run path:/home/brittonr/git/cairn#cairn -- gate proposal bundle-provider-fixed-point-release-evidence --root /home/brittonr/git/mantle

```text
{
  "change": "bundle-provider-fixed-point-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "404cb37383fe4169f1f4068b833450a563decf03942365b94ee5092b621531e0",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "52192405c5273e816669700f7dda7a10677c4629e3c9c53f4ab8742069befe9c",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

exit_status=0
```

## nix run path:/home/brittonr/git/cairn#cairn -- gate design bundle-provider-fixed-point-release-evidence --root /home/brittonr/git/mantle

```text
{
  "change": "bundle-provider-fixed-point-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b2e6c22c47137f811da8cac6f74688787509ea5b25574e8cdfa0dfe510568218",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ea5260e3d9e3f1c1974c5fa559fde3c9fdb7bb01dd74ac6acee2680db9910ba4",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

exit_status=0
```

## nix run path:/home/brittonr/git/cairn#cairn -- gate tasks bundle-provider-fixed-point-release-evidence --root /home/brittonr/git/mantle

```text
{
  "change": "bundle-provider-fixed-point-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "29e76adb0265673ca09819d6c2ebe1cff52522038759a8eb75e60430c8969ec1",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "d660426453ee1ab6f391097d8a45fd8a7c755780ec1b7b9506424a5c98f3e0b6",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

exit_status=0
```

## Post-archive: nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle

```text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 5,
  "valid": true
}

exit_status=0
```

## Post-archive: nix run path:/home/brittonr/git/cairn#cairn -- change list --root /home/brittonr/git/mantle

```text
{
  "changes": [],
  "layout": "cairn",
  "root": "/home/brittonr/git/mantle"
}

exit_status=0
```

## Archived-task gate with completed tasks (temporary restored copy)

```text
{
  "change": "bundle-provider-fixed-point-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "93ffdb4ad523fbf4aeb79e95d1deb59dd514f85fb7a609922c3fc1b0dc1bad28",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "152b244f93f9ed8860a0704c348bfd0cbbf46f1a279660ecc0dbad781243e4d6",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

exit_status=0
```

## Final post-archive: nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle

```text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 5,
  "valid": true
}

exit_status=0
```

## Final post-archive: nix run path:/home/brittonr/git/cairn#cairn -- change list --root /home/brittonr/git/mantle

```text
{
  "changes": [],
  "layout": "cairn",
  "root": "/home/brittonr/git/mantle"
}

exit_status=0
```
