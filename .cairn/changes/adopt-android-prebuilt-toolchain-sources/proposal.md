# Change: Admit prebuilt Android toolchain through fixed-output sources

## Why

The Mantle bootstrap chain ends at GCC 10.5.0, binutils 2.41, and musl 1.2.5. It contains no JVM, no JDK, and no Android SDK component. Building an APK requires JVM-hosted tools (`javac`, `d8`, `apksigner`) plus the prebuilt `aapt2` binary.

Source-building OpenJDK and the Android SDK from the current chain is a separate large project. Nix demonstrates the pragmatic boundary: admit the prebuilt JDK and SDK command-line tools as pinned fixed-output sources. Mantle already owns this machinery (`builtin:fetchurl`, `FetchBuildService`, source bundles, and the offline fixed-fetch override handoff).

This change admits those sources with explicit identity binding and explicit non-claims. It does not extend the source-built bootstrap story.

## What Changes

- Add a typed Nickel source manifest for prebuilt Android toolchain components (JDK, SDK command-line tools, build-tools, platform `android.jar`).
- Pin every component by upstream URL, SHA-256, and BLAKE3 record identity.
- Bind every derivation that executes a prebuilt tool to an explicit toolchain identity record. Digest drift fails closed before execution.
- Record the trust decision in `adr/0079-admit-prebuilt-android-toolchain-through-fixed-output-sources.md`.
- Keep offline replay working: admitted records MUST flow through the existing `SourceFetchOverridePlan` handoff so proofs need no live network after first fetch.
- Add positive and negative contract tests for the manifest and the identity binding.

## Non-Goals

- Source-building OpenJDK, Gradle, or any Android SDK component.
- Bootstrapping a JVM into the source-built chain.
- Any APK build step; that is `add-apk-build-adapter`.
- Verifying prebuilt binary behavior beyond content digest match.
- Cross-component trust, signing-key discovery, or SDK license automation.
- Building Android OS images, device images, or verified-boot artifacts; that is Robotnix scope.
- Running Mantle on Android hosts. Mantle's bwrap sandbox needs user namespaces on a Linux host; on-device operation through proot, the Nix-on-Droid approach, is a separate platform question.

## Dependencies

- None. This change is the first of the Android adapter family and owns the new `android-adapter` accepted spec.
- `add-apk-build-adapter` MUST archive after this change.

## Impact

- **Affected specs:** `android-adapter` (new)
- **Affected code:** `lib/android/sources.ncl` (new), `lib/android.ncl` (new), `crunch-eval` embedded stdlib inclusion, contract tests, `adr/0079`
- **Compatibility:** additive; no existing derivation or fetch behavior changes
