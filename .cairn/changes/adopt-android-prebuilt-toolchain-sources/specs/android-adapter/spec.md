# Android Adapter Delta

## ADDED Requirements

### Requirement: android_adapter.prebuilt_source_admission

r[android_adapter.prebuilt_source_admission]

Prebuilt Android toolchain components MUST enter the Android adapter through fixed-output fetch derivations declared in a typed source manifest. Each component record MUST carry the component name, exact version, upstream URL, SHA-256 digest of archive bytes, BLAKE3 record identity over canonical normalized metadata excluding that identity field, unpack shape, and declared platform. The manifest MUST reject incomplete records. The reviewed cohort's BLAKE3 metadata identities MUST be recomputed by a deterministic freshness rail before accepting changes; Nickel does not provide BLAKE3 hashing.

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

Every derivation produced by the Android adapter for execution of a prebuilt toolchain component MUST declare the matching reviewed toolchain identity and fixed-output fetch as an input. Metadata drift MUST be refused before lowering the consumer; if acquired bytes differ from the SHA-256 pin, the fetch MUST fail before any consumer tool executes. BLAKE3 at Nickel lowering is a checked-in metadata binding, not an independent runtime recomputation.

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
