# Tasks

## 1. Core plan model

- [ ] [serial] 1.1 Add `crates/crunch-android-core` as a pure workspace crate with `serde` in alloc-only mode, following the existing `-core` precedent. r[android_adapter.apk_plan_functional_core]
- [ ] [serial] 1.2 Define the typed `ApkPlan`: module identity, application id, version, manifest, resources, Java sources, toolchain identity references, signing configuration, and reproducibility policy. r[android_adapter.apk_plan_functional_core]
- [ ] [serial] 1.3 Define typed rejection categories for missing manifest member, unbound toolchain identity, unpinned timestamp policy, keystore path escape, unknown signature scheme, and empty source set. r[android_adapter.apk_plan_functional_core]
- [ ] [serial] 1.4 Implement validation and lowering to ordered typed derivation-step plans with per-step inputs, toolchain identity, and output. r[android_adapter.apk_plan_functional_core]
- [ ] [serial] 1.5 Add positive core tests: valid plan lowers to the ordered step list; unsigned plans omit the signing step. r[android_adapter.apk_plan_functional_core]
- [ ] [serial] 1.6 Add negative core tests: table-driven coverage of every rejection category and boundary values. r[android_adapter.apk_plan_functional_core]
- [ ] [serial] 1.7 Verify core purity with a `cargo check -p crunch-android-core --target wasm32-unknown-unknown` leg. r[android_adapter.apk_plan_functional_core]

## 2. Std adapter and derivation generation

- [ ] [serial] 2.1 Add `crates/crunch-android` as the std adapter that binds toolchain identity records to fetch outputs and checks digest binding before script generation. r[android_adapter.apk_pipeline_lowering]
- [ ] [serial] 2.2 Generate one derivation per step: `aapt2 compile`, `aapt2 link`, `javac`, `d8`, `zipalign`, and `apksigner` when signing is configured. r[android_adapter.apk_pipeline_lowering]
- [ ] [serial] 2.3 Pin the reproducibility environment in every generated derivation: entry timestamp epoch, `LC_ALL=C`, fixed `TZ`, deterministic entry ordering, and fixed JVM user properties. r[android_adapter.apk_pipeline_lowering]
- [ ] [serial] 2.4 Admit keystores only as explicit derivation inputs and prove no generated step references ambient user configuration paths. r[android_adapter.apk_pipeline_lowering]
- [ ] [serial] 2.5 Emit the unsigned zipaligned output when signing is not configured. r[android_adapter.apk_pipeline_lowering]

## 3. Nickel surface

- [ ] [serial] 3.1 Add `lib/android.ncl` with the `mkApk` contract and plan authoring helpers, composing with `lib/android/sources.ncl`. r[android_adapter.apk_pipeline_lowering]
- [ ] [serial] 3.2 Add the new modules to the `crunch-eval` embedded stdlib mirror and verify with `CRUNCH_FORCE_EMBEDDED_STDLIB=1`. r[android_adapter.apk_pipeline_lowering]
- [ ] [serial] 3.3 Add Nickel contract tests for the `mkApk` surface with positive and negative fixtures. r[android_adapter.apk_pipeline_lowering]

## 4. Offline shape tests

- [ ] [serial] 4.1 Add stub tool executables that record argv and environment to a declared output. r[android_adapter.apk_offline_shape_tests]
- [ ] [serial] 4.2 Add the offline shape test asserting step ordering, input wiring, pinned environment values, and stub-only execution. r[android_adapter.apk_offline_shape_tests]
- [ ] [serial] 4.3 Add offline rejection tests proving each typed category fails before any stub invocation. r[android_adapter.apk_offline_shape_tests]

## 5. Validate

- [ ] [serial] 5.1 Run `cargo test -p crunch-android-core -p crunch-android` with positive and negative coverage. r[android_adapter.apk_plan_functional_core]
- [ ] [serial] 5.2 Run the offline shape integration tests through the pipeline test harness with `can_build()` gating on non-Linux hosts. r[android_adapter.apk_offline_shape_tests]
- [ ] [serial] 5.3 Run first-package `cargo fmt --check` and focused Clippy for the new crates with `-D warnings`. r[android_adapter.apk_plan_functional_core]
- [ ] [serial] 5.4 Update the first-party Clippy exclusion lists and `deny.toml` handling if the new crates need entries. r[android_adapter.apk_plan_functional_core]
- [ ] [serial] 5.5 Run Cairn validation and the proposal, design, and tasks gates for this change. r[android_adapter.apk_offline_shape_tests]
