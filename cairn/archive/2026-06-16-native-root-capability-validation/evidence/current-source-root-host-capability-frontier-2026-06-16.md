# Current source-root host capability frontier (2026-06-16)

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.native_materialization]

## Summary

The current source-root musl provider is intentionally accepted as a target root, but rejected as a host root for the GNU-host Rust provider before host member path collection. No claiming manifest is written.

## Inputs

- Rust provider: `/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider`
- Host root attempted: `/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain`
- Target root attempted: `/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain`
- Output path: `/home/brittonr/git/mantle/target/native-root-capability-validation/frontier/native-toolchain-closure.json`

## Command

```sh
cargo run -p mantle --bin mantle -- bootstrap native-toolchain-closure --rust-source-provider /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider --host-root /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain --target-root /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain --output /home/brittonr/git/mantle/target/native-root-capability-validation/frontier/native-toolchain-closure.json
status=1
```

### stdout

```text
```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
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
8 | pub const FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1: &str = "manifest-kind-allowlist-v1";
  |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_KIND_ALLOWLIST_SCHEMA_V1` is never used
 --> src/frontend_artifact_spec.rs:9:11
  |
9 | pub const FRONTEND_ARTIFACT_KIND_ALLOWLIST_SCHEMA_V1: &str = "mantle-frontend-artifact-kind-allowlist-v1";
  |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_ADMISSION_SIDECAR_SCHEMA` is never used
  --> src/frontend_artifact_spec.rs:10:11
   |
10 | pub const FRONTEND_ARTIFACT_ADMISSION_SIDECAR_SCHEMA: &str = "mantle-frontend-artifact-admission-v1";
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_MISSING_SPEC` is never used
  --> src/frontend_artifact_spec.rs:11:11
   |
11 | pub const FRONTEND_ARTIFACT_DIAG_MISSING_SPEC: &str = "frontend-artifact-missing-spec";
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_EMPTY_FIELD` is never used
  --> src/frontend_artifact_spec.rs:12:11
   |
12 | pub const FRONTEND_ARTIFACT_DIAG_EMPTY_FIELD: &str = "frontend-artifact-empty-field";
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_HASH` is never used
  --> src/frontend_artifact_spec.rs:13:11
   |
13 | pub const FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_HASH: &str = "frontend-artifact-unsupported-hash";
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_INVALID_HASH` is never used
  --> src/frontend_artifact_spec.rs:14:11
   |
14 | pub const FRONTEND_ARTIFACT_DIAG_INVALID_HASH: &str = "frontend-artifact-invalid-hash";
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_HASH_MISMATCH` is never used
  --> src/frontend_artifact_spec.rs:15:11
   |
15 | pub const FRONTEND_ARTIFACT_DIAG_HASH_MISMATCH: &str = "frontend-artifact-hash-mismatch";
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_VALIDATOR` is never used
  --> src/frontend_artifact_spec.rs:16:11
   |
16 | pub const FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_VALIDATOR: &str = "frontend-artifact-unsupported-validator";
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH` is never used
  --> src/frontend_artifact_spec.rs:17:11
   |
17 | pub const FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH: &str = "frontend-artifact-spec-binding-mismatch";
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_HIDDEN_FALLBACK` is never used
  --> src/frontend_artifact_spec.rs:18:11
   |
18 | pub const FRONTEND_ARTIFACT_DIAG_HIDDEN_FALLBACK: &str = "frontend-artifact-hidden-fallback";
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL` is never used
  --> src/frontend_artifact_spec.rs:19:11
   |
19 | pub const FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL: &str = "frontend-artifact-invalid-spec-material";
   |           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: constant `FRONTEND_ARTIFACT_DIAG_KIND_NOT_ALLOWED` is never used
  --> src/frontend_artifact_spec.rs:20:11
   |
20 | pub const FRONTEND_ARTIFACT_DIAG_KIND_NOT_ALLOWED: &str = "frontend-artifact-kind-not-allowed";
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
101 | pub fn admit_frontend_artifact(request: &FrontendArtifactAdmissionRequest<'_>) -> FrontendArtifactAdmissionReport {
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
319 | fn require_non_empty(value: &str, path: &'static str, diagnostics: &mut Vec<FrontendArtifactAdmissionDiagnostic>) {
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
3812 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestLockInputTexts, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3837:4
     |
3837 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInput, String> {
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3846:4
     |
3846 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3859:4
     |
3859 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3890:4
     |
3890 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3920:4
     |
3920 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<NativeManifestLockUnsupportedBlocker> {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3952:4
     |
3952 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3956:4
     |
3956 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> NativeManifestLockUnsupportedBlocker {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5226:4
     |
5226 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest) -> NativeFeatureRoleResolution {
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "mantle") generated 49 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.21s
     Running `/home/brittonr/.cargo-target/debug/mantle bootstrap native-toolchain-closure --rust-source-provider /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider --host-root /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain --target-root /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain --output /home/brittonr/git/mantle/target/native-root-capability-validation/frontier/native-toolchain-closure.json`
error: build failed
source-built native toolchain closure blocked: host-root capability mismatch under /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain: source-root target `x86_64-linux-musl` cannot satisfy host-root for `x86_64-unknown-linux-gnu`
```

## Output check

No manifest was written, as expected for a fail-closed host-root capability mismatch.
