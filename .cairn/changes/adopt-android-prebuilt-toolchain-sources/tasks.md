# Tasks

## 1. Baseline and trust decision

- [x] [serial] 1.1 Confirm no existing Android toolchain surface with a repository grep and record the result. r[android_adapter.prebuilt_non_claims]
- [x] [serial] 1.2 Write `adr/0089-admit-prebuilt-android-toolchain-through-fixed-output-sources.md` recording the fixed-output admission decision, the rejected source-build-now alternative, and the non-claims. r[android_adapter.prebuilt_non_claims]
- [x] [serial] 1.3 Select the initial component cohort (JDK 17 LTS, SDK command-line tools, one build-tools release, one platform `android.jar`) with exact versions for the manifest. r[android_adapter.prebuilt_source_admission]

## 2. Source manifest contract

- [x] [serial] 2.1 Add the typed Nickel contract for component records in `lib/android/sources.ncl` with all required fields and platform admission. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 2.2 Implement record identity normalization so record identity is deterministic over normalized content. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 2.3 Lower each admitted record to one fixed-output `builtin:fetchurl` derivation with the pinned SHA-256. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 2.4 Add positive manifest contract tests: complete record admitted, deterministic identity across field reorder. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 2.5 Add negative manifest contract tests: missing URL, missing SHA-256, placeholder digest, empty version, unsupported platform, duplicate component. r[android_adapter.prebuilt_source_admission]

## 3. Toolchain identity binding

- [x] [serial] 3.1 Add the identity binding helper so derivations declare a toolchain identity as a required input. r[android_adapter.toolchain_identity_binding]
- [x] [serial] 3.2 Fail closed on digest drift before tool execution with a diagnostic naming the component and both digests. r[android_adapter.toolchain_identity_binding]
- [x] [serial] 3.3 Add positive identity tests: matching identity resolves to the fetch output. r[android_adapter.toolchain_identity_binding]
- [x] [serial] 3.4 Add negative identity tests: drifted digest rejected before execution, missing identity rejected at lowering. r[android_adapter.toolchain_identity_binding]

## 4. Offline replay and stdlib wiring

- [x] [serial] 4.1 Keep admitted records compatible with `SourceFetchOverridePlan` kind and URL matching. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 4.2 Add an offline replay test using the existing `file://` fixture pattern that fails closed on unmatched requests. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 4.3 Include the new `lib/android/` modules in the `crunch-eval` embedded stdlib mirror and verify with `CRUNCH_FORCE_EMBEDDED_STDLIB=1`. r[android_adapter.prebuilt_source_admission]

## 5. Validate

- [x] [serial] 5.1 Run focused Nickel contract tests for the manifest and identity binding. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 5.2 Run `cargo test -p mantle --lib` for shell integration surfaces touched by the new lib modules. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 5.3 Run `cargo fmt --check -p mantle -v` for the root package scope. r[android_adapter.prebuilt_source_admission]
- [x] [serial] 5.4 Run Cairn validation and the proposal, design, and tasks gates for this change. r[android_adapter.prebuilt_non_claims]
