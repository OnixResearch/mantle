# Android Adapter Specification

## Purpose

Defines the `android-adapter` capability.

## Requirements

### Requirement: android_adapter.prebuilt_source_admission

r[android_adapter.prebuilt_source_admission]

Prebuilt Android toolchain components MUST enter Mantle only through fixed-output fetch derivations declared in a typed source manifest. Each component record MUST carry the component name, exact version, upstream URL, SHA-256 digest, BLAKE3 record identity, unpack shape, and declared platform. The manifest MUST reject incomplete records. For the `android.reviewed` adapter, the SHA-256 pin is over archive bytes and the distinct reviewed BLAKE3 identity is over canonical normalized metadata excluding the identity field. The reviewed cohort's metadata identities MUST be recomputed by a deterministic freshness rail before accepting changes; Nickel does not provide BLAKE3 hashing. Its binder MUST check the selected published source-v1 fixed-output record against the finite reviewed-to-published cohort mapping.

#### Scenario: Complete pinned record is admitted

- **GIVEN** a manifest record contains component name, exact version, upstream URL, SHA-256, BLAKE3 identity, unpack shape, and platform
- **WHEN** Mantle validates the source manifest
- **THEN** it MUST admit the record and plan one fixed-output fetch derivation
- **AND** the record identity MUST be deterministic over the normalized record content

#### Scenario: Incomplete record is rejected

- **GIVEN** a record is missing the URL, the SHA-256, the version, or carries a placeholder digest
- **WHEN** Mantle validates the source manifest
- **THEN** it MUST reject the manifest with ordered diagnostics
- **AND** it MUST NOT plan a fetch derivation or contact the network

### Requirement: android_adapter.toolchain_identity_binding

r[android_adapter.toolchain_identity_binding]

Every derivation that executes a prebuilt Android toolchain component MUST declare the matching toolchain identity record as an input. If the resolved component bytes do not match the recorded digest, the build MUST fail before the tool executes. For derivations produced by the `android.reviewed` adapter, the matching reviewed identity and published source-v1 fixed-output fetch MUST both be declared as inputs. Metadata drift MUST be refused before lowering the consumer; if acquired bytes differ from the SHA-256 pin, the fetch MUST fail before any consumer tool executes. BLAKE3 at Nickel lowering is a checked-in metadata binding, not an independent runtime recomputation.

#### Scenario: Identity matches admitted bytes

- **GIVEN** a derivation declares a toolchain identity and the fetched component matches its recorded digest
- **WHEN** the derivation is built
- **THEN** execution MUST proceed with only the declared inputs available in the sandbox

#### Scenario: Digest drift fails closed

- **GIVEN** a derivation declares a toolchain identity and the available component bytes differ from the recorded digest
- **WHEN** the derivation is scheduled
- **THEN** the build MUST fail before tool execution
- **AND** the failure MUST name the component and the mismatched digest

### Requirement: android_adapter.prebuilt_non_claims

r[android_adapter.prebuilt_non_claims]

Documentation, receipts, and the admission ADR MUST record that prebuilt Android toolchain components are not source-built, carry no bootstrap-chain claim, and are proven only to the extent of content digest match. Their execution MUST remain confined to sandbox derivations.

#### Scenario: Receipt carries the non-claim boundary

- **GIVEN** a build consumed a prebuilt toolchain component
- **WHEN** the build receipt is written
- **THEN** the receipt MUST identify the consumed prebuilt component identities
- **AND** it MUST NOT claim source-built provenance for those components

#### Scenario: Prebuilt component runs outside a sandbox derivation

- **GIVEN** a caller attempts to execute an admitted prebuilt component outside a sandboxed derivation boundary
- **WHEN** the operation is evaluated
- **THEN** Mantle MUST NOT provide an execution path for that request

### Requirement: android_adapter.apk_plan_functional_core

r[android_adapter.apk_plan_functional_core]

The APK build adapter MUST provide a pure functional core that validates a typed APK plan and lowers it to ordered typed derivation-step plans. The core MUST NOT perform filesystem, network, clock, environment, or store access, and MUST return typed rejection categories for invalid plans.

#### Scenario: Valid plan lowers to ordered steps

- **GIVEN** a complete APK plan with manifest, resources, Java sources, bound toolchain identities, and a reproducibility policy
- **WHEN** the core validates and lowers the plan
- **THEN** it MUST return the ordered steps for `aapt2 compile`, `aapt2 link`, `javac`, `d8`, `zipalign`, and, when signing is configured, `apksigner`
- **AND** each step MUST name its inputs, its toolchain identity, and its output

#### Scenario: Invalid plan is rejected with a typed category

- **GIVEN** a plan with a missing manifest member, an unbound toolchain identity, an unpinned timestamp policy, a keystore path escape, an unknown signature scheme, or an empty source set
- **WHEN** the core validates the plan
- **THEN** it MUST return the matching typed rejection category
- **AND** it MUST NOT produce any step plan

### Requirement: android_adapter.apk_pipeline_lowering

r[android_adapter.apk_pipeline_lowering]

The adapter MUST lower a validated plan to separate Mantle derivations for resource compilation and linking, Java compilation, dexing, zip alignment, and signing. Every derivation MUST pin a fixed entry timestamp epoch, a fixed locale and timezone, and deterministic entry ordering, and MUST declare its toolchain identities as inputs. Keystores MUST be admitted only as explicit derivation inputs.

#### Scenario: Steps are separate derivations with pinned reproducibility inputs

- **GIVEN** a validated plan and resolved toolchain components matching their recorded digests
- **WHEN** the adapter generates derivations
- **THEN** each step MUST be its own derivation with declared inputs
- **AND** each derivation environment MUST pin the entry timestamp epoch, locale, and timezone from the plan policy

#### Scenario: Signing input stays explicit

- **GIVEN** a plan configured for signing with a keystore input
- **WHEN** the adapter generates the signing derivation
- **THEN** the keystore MUST appear as a declared derivation input
- **AND** no ambient user configuration path MUST be referenced by any generated step

#### Scenario: Unsigned path defers signing

- **GIVEN** a plan without signing configuration
- **WHEN** the adapter generates derivations
- **THEN** the output MUST be the zipaligned unsigned APK
- **AND** no signing step MUST be generated

### Requirement: android_adapter.apk_offline_behavior_tests

r[android_adapter.apk_offline_behavior_tests]

The adapter MUST be covered by offline tests for typed rejection, declared-input-sensitive output identities, and same-run extraction admission. Fixed-output fixture archives MUST be fetched through the builder and extracted in its real sandbox before testing provenance rejection. Synthetic SDK scripts, echoed invocations, and placeholder APK bytes MUST NOT be used as evidence that an SDK stage ran or produced a usable APK.

#### Scenario: Equivalent declarations retain their output identity

- **GIVEN** a reviewed APK plan with declared source, toolchain, runtime, and optional signing inputs
- **WHEN** the adapter lowers equivalent plans, a reordered resource list, a changed declared resource, and a changed signing identity
- **THEN** equivalent plans and resource orderings MUST retain the same final output identity
- **AND** changed declared resource or signing inputs MUST change that identity

#### Scenario: Unsafe extraction outcomes stop before an SDK stage

- **GIVEN** local fixed-output fixture archives fetched by the builder and extracted in the sandbox during the same run
- **WHEN** an extraction outcome is swapped, cached, missing a trusted store signature, or bound to a drifted runtime digest
- **THEN** the adapter MUST reject the extraction outcome before admitting a tool tree or generating an SDK stage

#### Scenario: Rejection paths are exercised offline

- **GIVEN** plans that trigger each typed rejection category or an archive whose bytes differ from its declared digest
- **WHEN** the offline tests evaluate them
- **THEN** each plan MUST fail with its typed category, or the archive MUST fail with digest drift
- **AND** no derivation graph MUST be generated for a rejected input

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

### Requirement: android_adapter.apk_offline_shape_tests

r[android_adapter.apk_offline_shape_tests]

The adapter MUST be covered by offline integration tests that use stub tool executables instead of real toolchain binaries. These tests MUST prove step ordering, input wiring, pinned environment values, and rejection paths without network access or prebuilt binary execution.

#### Scenario: Shape test proves ordering and wiring

- **GIVEN** stub executables that record their argv and environment
- **WHEN** the offline shape test builds a plan
- **THEN** the recorded invocations MUST appear in the required order with the declared inputs and pinned environment values

#### Scenario: Rejection paths are exercised offline

- **GIVEN** plans that trigger each typed rejection category
- **WHEN** the offline tests evaluate them
- **THEN** each MUST fail with its typed category before any step executes
- **AND** no stub tool MUST be invoked for a rejected plan
