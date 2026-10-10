# Android Adapter Delta

## ADDED Requirements

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
