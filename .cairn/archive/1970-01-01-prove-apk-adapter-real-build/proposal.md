# Change: Prove the APK adapter with a real toolchain build

## Why

Offline adapter tests prove typed rejection before SDK effects and declared-input-sensitive output identities; they do not execute SDK stages or produce a working APK. The adapter's real-build claim needs execution with the admitted prebuilt JDK and Android SDK: build a minimal checked-in APK, verify its structure, and prove rebuild determinism.

This proof is execution-gated. It needs the network once to fetch the admitted toolchain components, or a prefetched source bundle. After that, replay is offline. The change records its environment prerequisites and its blockers rather than claiming unexecuted results.

## What Changes

- Add `examples/android-minimal.ncl`: a minimal signed APK plan (one activity, one XML string resource, one Java source, fixed id/version, test-only signing).
- Add an evidence rail that builds the example with the admitted toolchain and writes a receipt binding toolchain identities and the output digest.
- Prove rebuild determinism: two clean rebuilds in fresh stores produce the same output BLAKE3.
- Add bounded structural verification of the produced APK: deterministic zip entry ordering, compiled manifest presence, DEX presence, signing block presence when signed, and an `apksigner verify` capture.
- Add negative controls: a perturbed Java source changes the digest; a tampered APK byte fails verification.
- Record explicit non-claims: no install, launch, or runtime behavior on any device or emulator.

## Non-Goals

- Device or emulator testing, install verification, or launch verification.
- Performance measurement of the toolchain or the adapter.
- Coverage of resource-heavy applications, native libraries, or AAB output.
- Extending trust in the prebuilt binaries beyond digest match; the admission change owns that boundary.

## Dependencies

- `add-apk-build-adapter` MUST archive first.
- Network access for the first toolchain fetch, or a prefetched source bundle produced through source-bundle export.

## Impact

- **Affected specs:** `android-adapter`
- **Affected code:** `examples/android-minimal.ncl`, `lib/android.ncl`'s reviewed-to-published fetch bridge, the checked-in evidence rail and ignored real-SDK test, and `crates/crunch-android/src/lib.rs` signing-root admission and derivation inputs
- **Compatibility:** direct signed `PreparedApk::bind` callers must supply a keystore member beneath a distinct signing store object; shared app-source signing paths now fail before graph generation. Unsigned plans and the separate pinned-source API are unchanged.
