# Android Adapter Delta

## ADDED Requirements

### Requirement: android_adapter.real_apk_build_evidence

r[android_adapter.real_apk_build_evidence]

The adapter MUST provide an execution-gated evidence rail that builds the checked-in minimal APK example with the admitted prebuilt toolchain and writes a receipt binding the consumed toolchain identities, the step derivation identities, the output digest, and the affecting environment facts. The rail MUST fail closed with a blocker record when prerequisites are absent.

#### Scenario: Real build produces a bound receipt

- **GIVEN** the admitted toolchain components are available through fetch or prefetch and the adapter is archived
- **WHEN** the evidence rail builds the minimal example
- **THEN** it MUST write a receipt binding toolchain identities, digests, step derivations, and the output BLAKE3
- **AND** the receipt MUST NOT claim install, launch, or runtime correctness

#### Scenario: Prerequisites are absent

- **GIVEN** no network access and no prefetched source bundle
- **WHEN** the evidence rail starts
- **THEN** it MUST fail closed with a recorded blocker
- **AND** it MUST NOT write a success receipt

### Requirement: android_adapter.apk_rebuild_determinism

r[android_adapter.apk_rebuild_determinism]

Two clean rebuilds of the same plan in fresh stores MUST produce the same output digest, and a perturbed source input MUST change the output digest.

#### Scenario: Clean rebuilds are byte-identical

- **GIVEN** two fresh store and state directories and the same plan and toolchain identities
- **WHEN** both builds complete
- **THEN** the output BLAKE3 digests MUST match
- **AND** a digest mismatch MUST be reported with the producing step

#### Scenario: Perturbed input changes the digest

- **GIVEN** the same plan with one Java source perturbed
- **WHEN** the build completes
- **THEN** the output digest MUST differ from the unperturbed build

### Requirement: android_adapter.apk_structure_verification

r[android_adapter.apk_structure_verification]

The evidence rail MUST perform bounded structural verification of the produced APK: deterministic zip entry ordering, compiled manifest presence, DEX presence, signing block presence when signed, and a captured `apksigner verify` result. A tampered APK MUST fail verification.

#### Scenario: Built APK passes structural checks

- **GIVEN** a completed real build with signing configured
- **WHEN** structural verification runs
- **THEN** entries MUST parse in deterministic order with the compiled manifest and DEX present
- **AND** the APK Signing Block MUST be present and `apksigner verify` MUST succeed

#### Scenario: Tampered APK fails verification

- **GIVEN** a signed APK with one byte flipped in a signed region
- **WHEN** verification runs
- **THEN** `apksigner verify` MUST fail
- **AND** the failure MUST be recorded as evidence that verification observes content
