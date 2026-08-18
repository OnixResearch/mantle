# Final validation: optional snix sandbox shell compile env

Task-ID: V4
Covers: r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]

## Environment

```text
cwd: /home/brittonr/git/mantle
cargo: /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo
PKG_CONFIG_PATH: /nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
```

## unset-env-snix-build-tests

```text
command: env -u SNIX_BUILD_SANDBOX_SHELL cargo test -p snix-build choose_sandbox_shell --lib
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/snix_build-cac873d4af23e8c0)

running 5 tests
test buildservice::bwrap::tests::choose_sandbox_shell_prefers_explicit_env ... ok
test buildservice::bwrap::tests::choose_sandbox_shell_ignores_missing_compile_default ... ok
test buildservice::bwrap::tests::choose_sandbox_shell_keeps_placeholder_without_static_candidate ... ok
test buildservice::bwrap::tests::choose_sandbox_shell_uses_discovered_static_for_placeholder ... ok
test buildservice::bwrap::tests::choose_sandbox_shell_keeps_existing_compile_default ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.01s


exit_status: 0
```

## unset-env-snix-build-check

```text
command: env -u SNIX_BUILD_SANDBOX_SHELL cargo check -p snix-build --lib
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s

exit_status: 0
```

## unset-env-mantle-bin-check

```text
command: env -u SNIX_BUILD_SANDBOX_SHELL cargo check -p mantle --bin mantle
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: unused import: `std::collections::BTreeSet`
 --> src/cargo_import.rs:1:5
  |
1 | use std::collections::BTreeSet;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

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

warning: function `find_project_root` is never used
  --> src/project_build.rs:96:8
   |
96 | pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
   |        ^^^^^^^^^^^^^^^^^

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
    --> src/rust_plan.rs:1214:8
     |
1214 | struct NativeTextInput {
     |        ^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockInputTexts` is never constructed
    --> src/rust_plan.rs:1220:8
     |
1220 | struct NativeManifestLockInputTexts {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeManifestLockUnsupportedBlocker` is never constructed
    --> src/rust_plan.rs:1226:8
     |
1226 | struct NativeManifestLockUnsupportedBlocker {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolutionRequest` is never constructed
    --> src/rust_plan.rs:1250:8
     |
1250 | struct NativeFeatureRoleResolutionRequest {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: struct `NativeFeatureRoleResolution` is never constructed
    --> src/rust_plan.rs:1257:8
     |
1257 | struct NativeFeatureRoleResolution {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `collect_native_manifest_lock_texts` is never used
    --> src/rust_plan.rs:3849:4
     |
3849 | fn collect_native_manifest_lock_texts(root: &Path) -> Result<NativeManifestL...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `read_native_text_input` is never used
    --> src/rust_plan.rs:3874:4
     |
3874 | fn read_native_text_input(path: &Path, label: &str) -> Result<NativeTextInpu...
     |    ^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_unsupported_blockers` is never used
    --> src/rust_plan.rs:3883:4
     |
3883 | fn native_manifest_lock_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_unsupported_blockers` is never used
    --> src/rust_plan.rs:3896:4
     |
3896 | fn native_manifest_unsupported_blockers(input: &NativeTextInput) -> Vec<Nati...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_target_table_unsupported_blockers` is never used
    --> src/rust_plan.rs:3927:4
     |
3927 | fn native_target_table_unsupported_blockers(
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `native_lockfile_unsupported_blockers` is never used
    --> src/rust_plan.rs:3957:4
     |
3957 | fn native_lockfile_unsupported_blockers(input: &NativeTextInput) -> Vec<Nati...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `lock_source_supported` is never used
    --> src/rust_plan.rs:3989:4
     |
3989 | fn lock_source_supported(source: &str) -> bool {
     |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `native_manifest_lock_blocker` is never used
    --> src/rust_plan.rs:3993:4
     |
3993 | fn native_manifest_lock_blocker(path: &str, class: &str, message: &str) -> N...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `resolve_native_feature_roles` is never used
    --> src/rust_plan.rs:5263:4
     |
5263 | fn resolve_native_feature_roles(request: NativeFeatureRoleResolutionRequest)...
     |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `mantle` (bin "mantle") generated 54 warnings (run `cargo fix --bin "mantle" -p mantle` to apply 1 suggestion)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s

exit_status: 0
```

## rustfmt-check

```text
command: cargo fmt --check -p mantle -p snix-build

exit_status: 0
```

## diff-whitespace-check

```text
command: git diff --check

exit_status: 0
```

## cairn-validate

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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

exit_status: 0
```

## cairn-gate-proposal

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- gate proposal source-built-provider-snix-sandbox-shell --root .
{
  "change": "source-built-provider-snix-sandbox-shell",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e4d41c995f77ff92e12989e27440044d6e1186305ee1923dd253c4f65d676a10",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "003f25fb4262d124ab16e4692e46af5fbb6049c92e049add8db6cbd9fb098974",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```

## cairn-gate-design

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- gate design source-built-provider-snix-sandbox-shell --root .
{
  "change": "source-built-provider-snix-sandbox-shell",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "719df4e2a0e12a6d0d1887692b690de51993b81f583654030cf173ea16f5ea95",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "58e5ba5a87a79ecff1cf90ff68b6c0e05e8fc1c8804c1d881f35a1b3190e7f4b",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```

## cairn-gate-tasks

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-provider-snix-sandbox-shell --root .
{
  "change": "source-built-provider-snix-sandbox-shell",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a99e8209b419fdfc5bd0c45ec836c734c6d1f6359ef915c1cd446476bb73d0d6",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ace4162379a44832fe18131e1e215d845b7361ef49a6a782d8f62f85026d3be3",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```


## post-task-update-cairn-gate-tasks

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-provider-snix-sandbox-shell --root .
{
  "change": "source-built-provider-snix-sandbox-shell",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "d202bed49d0d49c21952bf987c2d7e0ebdd041fcc0c0ff2804b431b88a670f0e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e06e1f343f007d977dcc91bee24a09f5ba63b7f298652f51f21eb3e7d90556ba",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```

## post-task-update-diff-whitespace-check

```text
command: git diff --check

exit_status: 0
```

## post-task-update-cairn-validate

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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

exit_status: 0
```

## post-archive-cairn-validate

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}

exit_status: 0
```
