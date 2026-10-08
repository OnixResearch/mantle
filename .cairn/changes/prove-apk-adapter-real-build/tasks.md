# Tasks

## 1. Example and prerequisites

- [x] [serial] 1.1 Add `examples/android-minimal.ncl` with one activity, the manifest, one Java source, fixed application id and version, and signing configured. r[android_adapter.real_apk_build_evidence]
- [x] [serial] 1.2 Record the proof environment prerequisites: one connected fetch or a prefetched source bundle, sandbox shell, and disk bounds. r[android_adapter.real_apk_build_evidence]
- [x] [serial] 1.3 Prefetch the admitted toolchain components into a source bundle and verify offline replay through `SourceFetchOverridePlan`. r[android_adapter.real_apk_build_evidence]

## 2. Evidence rail

- [ ] [serial] 2.1 Add the checked-in evidence rail that builds the example and writes the receipt binding toolchain identities, digests, step derivations, output BLAKE3, and environment facts. r[android_adapter.real_apk_build_evidence]
- [x] [serial] 2.2 Fail closed with a blocker record when prerequisites are absent, and write no success receipt. r[android_adapter.real_apk_build_evidence]
- [ ] [serial] 2.3 Add structural verification: deterministic zip entry ordering, compiled manifest presence, DEX presence, signing block presence when signed, and captured `apksigner verify`. r[android_adapter.apk_structure_verification]
- [ ] [serial] 2.4 Add the tamper negative control: a flipped byte in a signed region MUST fail `apksigner verify`. r[android_adapter.apk_structure_verification]

## 3. Determinism proof

- [ ] [serial] 3.1 Run two clean rebuilds in fresh store and state directories and compare output BLAKE3 digests. r[android_adapter.apk_rebuild_determinism]
- [ ] [serial] 3.2 Run the perturbed-source negative control and assert the digest changes. r[android_adapter.apk_rebuild_determinism]
- [ ] [serial] 3.3 Run the real builds detached with pueue under the repo long-build conventions and capture full logs. r[android_adapter.real_apk_build_evidence]

## 4. Evidence and validation

- [ ] [serial] 4.1 Capture the pueue task output, `test result:` lines, and receipt digests into this change's evidence directory. r[android_adapter.real_apk_build_evidence]
- [x] [serial] 4.2 Record the non-claims: no install, launch, or runtime behavior on any device or emulator. r[android_adapter.real_apk_build_evidence]
- [ ] [serial] 4.3 Run Cairn validation and the proposal, design, and tasks gates for this change. r[android_adapter.real_apk_build_evidence]
