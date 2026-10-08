# Tasks

## 1. Baseline and trust decision

- [x] [serial] 1.1 Confirm no existing Android toolchain surface with a repository grep and record the result in evidence (prior mentions were proposals, not code). r[android_adapter.prebuilt_non_claims]
- [ ] [serial] 1.2 Review proposed `adr/0110-separate-reviewed-android-metadata-from-source-identity.md` for the reviewed prebuilt-record-v1 identity, rejected source-build-now alternative, and bounded non-claims. The separate named API and reviewed-to-source-v1 bridge are implemented, but their positive/drift fixtures, actual fixed-output byte-authority path, and merged-source gates must be exercised before accepting the ADR or checking this task. Published ADR 0089 remains the distinct source-v1 authority; the historical draft 0087 and its reviewed digests are not renumbered. r[android_adapter.prebuilt_non_claims]
- [x] [serial] 1.3 Select the initial component cohort (Temurin 17.0.17+10, Android command-line tools 19.0 revision 13114758, build-tools 35.0.0, platform android-35 r02 with `android.jar`) with directly downloaded upstream SHA-256 pins. r[android_adapter.prebuilt_source_admission]

## 2. Source manifest contract

- [x] [serial] 2.1 Add the typed Nickel contract for component records in `lib/android/sources.ncl` with all required fields and platform admission. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 2.2 Implement canonical record preimage with external BLAKE3 freshness check; Nickel cannot itself hash BLAKE3. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 2.3 Lower each admitted record to one fixed-output `builtin:fetchurl` derivation with the pinned SHA-256. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 2.4 Add positive manifest contract tests: complete record admitted, deterministic identity across field reorder. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 2.5 Add negative manifest contract tests: missing URL, missing SHA-256/identity, placeholder digest, empty version, unsupported platform, duplicate component, unsafe archive root, overcount. r[android_adapter.prebuilt_source_admission]

## 3. Toolchain identity binding

- [x] [serial] 3.1 Add the identity binding helper selecting the reviewed record and declaring the fixed-output fetch as a consumer input. r[android_adapter.toolchain_identity_binding]
- [x] [serial] 3.2 Fail closed on metadata digest drift before tool execution with a diagnostic naming the component and expected/observed digests; fetched-byte drift remains the SHA-256 fetcher's responsibility. r[android_adapter.toolchain_identity_binding]
- [x] [serial] 3.3 Prove a matching reviewed `android-build-tools` identity resolves to its actual original-HTTPS SHA-256-fixed store output in a native `android.bind_tool` consumer build; isolated signed PathInfo/NAR verification and byte-identical physical consumer output are recorded in the implementation receipt. r[android_adapter.toolchain_identity_binding]
- [x] [serial] 3.4 Add negative identity tests: drifted digest rejected before lowering, missing identity rejected at lowering. r[android_adapter.toolchain_identity_binding]

## 4. Offline replay and stdlib wiring

- [x] [serial] 4.1 Keep admitted records compatible with `SourceFetchOverridePlan` kind and URL matching; real fixture matched after deleting source file. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 4.2 Add an offline replay test using the existing `file://` fixture pattern that fails closed on unmatched requests at source preflight. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 4.3 Include new `lib/android/` modules in `crunch-eval` embedded stdlib and verify CLI subprocess with `CRUNCH_FORCE_EMBEDDED_STDLIB=1`. r[android_adapter.prebuilt_source_admission]

## 5. Validate

- [x] [serial] 5.1 Run focused Nickel contract/identity tests via `cargo test -p mantle --test android_sources` in isolated origin/main overlay (7/7). r[android_adapter.prebuilt_source_admission]
- [x] [serial] 5.2 Run focused `cargo test -p mantle --test android_sources` in isolated origin/main overlay (7/7) and current shared checkout after a fresh root check (`--locked --offline`, 7/7). r[android_adapter.prebuilt_source_admission]
- [x] [serial] 5.3 Run `cargo fmt --check -p mantle -p crunch-eval` in isolated origin/main Android overlay (exit 0). r[android_adapter.prebuilt_source_admission]
- [x] [serial] 5.4 Run Cairn validation and proposal/design/tasks gates for this change (gates PASS; global validation reports unrelated thin-cli task-order issue). r[android_adapter.prebuilt_non_claims]
