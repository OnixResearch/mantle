# Change: Prove the APK adapter with a real toolchain build

## Why

Offline shape tests prove ordering and wiring, not a working APK. The adapter's honest completion needs one execution with the real prebuilt JDK and Android SDK: build a minimal checked-in APK, verify its structure, and prove rebuild determinism.

This proof is execution-gated. It needs the network once to fetch the admitted toolchain components, or a prefetched source bundle. After that, replay is offline. The change records its environment prerequisites and its blockers rather than claiming unexecuted results.

## What Changes

- Add `examples/android-minimal.ncl`: a minimal APK plan (one activity, no resources beyond the manifest, one Java source).
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
- **Affected code:** `examples/android-minimal.ncl` (new), evidence rail script or integration test, receipts
- **Compatibility:** additive; evidence only
