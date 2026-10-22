# Change: Add the APK build adapter

## Why

Mantle derivations are deterministic commands over declared inputs. An APK is five such commands: `aapt2 compile` and `aapt2 link`, `javac`, `d8`, `zipalign`, and `apksigner`. No new build machinery is required. What is missing is the adapter that composes those steps with pinned inputs.

Gradle is hostile to the sandbox model: daemon processes, network during builds, and mutable caches. This adapter skips Gradle entirely and calls the SDK tools directly, the same manual pipeline that minimal Android builds use.

The composition follows the existing adapter split in this repo: a pure core crate owns plan validation and step lowering; a std adapter crate and a Nickel builder surface own path binding and derivation generation.

## What Changes

- Add `crates/crunch-android-core`: a pure, no-std-capable plan core with typed `ApkPlan`, validation, and lowering to ordered typed derivation-step plans.
- Add `crates/crunch-android`: the std adapter that binds toolchain identities and store paths and generates the five derivation scripts.
- Add `lib/android.ncl`: the Nickel surface with the `mkApk` contract and plan authoring helpers.
- Lower the plan to separate derivations with pinned reproducibility inputs: fixed entry timestamp epoch, fixed locale and timezone, and declared toolchain identities.
- Support an unsigned output path so CI can defer signing.
- Admit keystores only as explicit derivation inputs, never from ambient `HOME`.
- Add offline shape tests with stub tool executables that prove ordering, wiring, and hermeticity without executing real toolchain binaries.

## Non-Goals

- Gradle, Android Gradle Plugin, or any dependency-resolution network path. If Gradle support ever becomes necessary, the recorded escape hatch is Robotnix-style dependency materialization as a fixed-output step before an offline build; that stays a separate change.
- Nix-on-Droid-style on-device operation. The adapter builds on a Linux host with a bwrap sandbox.
- NDK, JNI native library packaging, or ABIs other than the declared host toolchain ABI.
- Android App Bundle (AAB) output.
- Installing, launching, or runtime-verifying the produced APK on a device or emulator.
- Building the toolchain; `adopt-android-prebuilt-toolchain-sources` owns admission.
- Proving the produced APK with real toolchain binaries; `prove-apk-adapter-real-build` owns that evidence.

## Dependencies

- `adopt-android-prebuilt-toolchain-sources` MUST archive first; the adapter consumes its identity records.
- Follows the `crunch-kernelscript` and `crunch-wasm-component` adapter split precedent.

## Impact

- **Affected specs:** `android-adapter`
- **Affected code:** `crates/crunch-android-core` (new), `crates/crunch-android` (new), `lib/android.ncl` (new), root `Cargo.toml` workspace membership, `crunch-eval` embedded stdlib mirror, integration tests
- **Compatibility:** additive; no existing builder or derivation contract changes
