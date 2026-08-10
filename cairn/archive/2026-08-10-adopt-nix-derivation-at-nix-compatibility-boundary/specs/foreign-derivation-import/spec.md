# Foreign Derivation Import Delta

## ADDED Requirements

### Requirement: Reviewed `nix-derivation` dependency

r[foreign_derivation_import.reviewed_nix_derivation_dependency] Mantle MUST admit `nix-derivation` only from an exact reviewed package whose packaged source, upstream commit, checksum, license, and supported Nix version match recorded lifecycle evidence.

#### Scenario: Exact reviewed package is admitted

- GIVEN the exact crate version, crates.io checksum, reviewed upstream commit, license, and packaged source comparison all match the adoption record
- WHEN Mantle validates the dependency
- THEN the dependency MAY enter the Nix compatibility adapter
- AND the accepted record MUST identify every matched source fact

#### Scenario: Package or source identity drifts

- GIVEN the crate version, checksum, upstream source, packaged files, license, or supported Nix version differs from the adoption record
- WHEN dependency admission runs
- THEN Mantle MUST reject production cutover
- AND it MUST NOT treat a mutable branch or package name as equivalent evidence

### Requirement: Reviewed Nix derivation adapter

r[foreign_derivation_import.reviewed_nix_derivation_adapter] Mantle MUST use one private compatibility adapter for production `nix-derivation` parsing and MUST preserve existing foreign graph, package-index, receipt, CLI, limit, and publication contracts.

#### Scenario: Concrete Nix derivation is admitted

- GIVEN a bounded concrete `/nix/store` `.drv` closure and a valid logical root identity
- WHEN the adapter parses and projects the closure
- THEN it MUST emit the existing bounded foreign graph facts without invoking Nix, a daemon, or an evaluator
- AND public artifact schemas MUST remain unchanged

#### Scenario: Unreviewed direct import appears

- GIVEN production code imports `nix-derivation` outside the private adapter
- WHEN the dependency boundary guard runs
- THEN validation MUST fail with a deterministic direct-import finding
- AND the bypass MUST NOT qualify for adoption evidence

### Requirement: Nix derivation projection boundary

r[foreign_derivation_import.nix_derivation_projection_boundary] The adapter MUST separate syntax parsing, semantic validation, unsupported-feature classification, and foreign-IR projection. It MUST derive the out-of-band derivation name from the logical `.drv` identity and MUST handle every parsed output and dynamic-input form explicitly.

#### Scenario: Supported derivation projects without loss

- GIVEN a supported parsed Nix derivation with valid outputs, inputs, structured attributes, and byte-valued environment entries that the foreign IR can represent
- WHEN projection runs
- THEN the adapter MUST preserve all required graph facts and ordered structured candidate data
- AND Nix identities MUST remain distinct from Mantle BLAKE3 identities

#### Scenario: Parsed value cannot enter the current IR

- GIVEN a non-UTF-8 environment value, unsupported output variant, unsupported version, excessive dynamic depth, invalid logical name, or unrepresentable structured value
- WHEN projection runs
- THEN the adapter MUST return a stable unsupported or rejection class
- AND it MUST NOT use lossy conversion, flatten the value silently, or publish a partial graph

### Requirement: Positive and negative Nix parity gate

r[foreign_derivation_import.nix_derivation_adapter_parity] Mantle SHALL cut over a Nix `.drv` compatibility path only after positive and negative dual-run evidence agrees with the pinned Nix behavior for canonical bytes, derivation hashes, store paths, graph facts, structured attributes, and rejection classes.

#### Scenario: Recorded parity corpus passes

- GIVEN the pinned Nix corpus and Mantle compatibility fixtures cover accepted traditional and versioned forms plus required negative cases
- WHEN both implementations and the reference expectations are compared
- THEN Mantle MAY cut over the covered `/nix/store` path
- AND evidence MUST bind the exact corpus, adapter, dependency, and Nix version identities

#### Scenario: Required parity case differs

- GIVEN canonical bytes, a Nix hash, a store path, a graph fact, structured data, or a required rejection differs
- WHEN cutover readiness is evaluated
- THEN the old production path MUST remain active for that surface
- AND the mismatch MUST receive an explicit adopted, adapted, deferred, or rejected disposition

### Requirement: Coupled adapter rollback

r[foreign_derivation_import.nix_derivation_adapter_rollback] Mantle MUST record a rollback that restores the prior compatibility adapter and dependency state together without changing accepted foreign artifact schemas.

#### Scenario: Adoption regression appears

- GIVEN a post-cutover parser, hash, path, projection, or diagnostic regression
- WHEN rollback executes
- THEN Mantle MUST restore the recorded prior adapter and dependency state
- AND existing foreign artifact readers MUST remain compatible

#### Scenario: Partial rollback is proposed

- GIVEN a rollback removes only the dependency or restores only call sites
- WHEN rollback validation runs
- THEN validation MUST reject the incomplete rollback
- AND production code MUST NOT retain an adapter and dependency mismatch
