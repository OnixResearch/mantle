## ADDED Requirements

### Requirement: Nix required system features are classified [r[foreign_derivation_import.required_system_features]]

The Nix producer adapter MUST parse every required system feature from each supported concrete derivation input. It MUST map each value through a closed supported-feature table or return a typed unsupported-feature blocker. Nix feature names and encodings MUST remain inside the Nix adapter and its fixtures.

#### Scenario: A derivation needs no special builder service

- GIVEN a supported Nix 2.36 derivation declares no required system feature outside the adapter's supported table
- WHEN the Nix producer lowers the derivation
- THEN it MUST preserve the existing frontend-neutral graph projection
- AND consumption MUST NOT require Nix, a daemon connection, or a builder-facing protocol

#### Scenario: A derivation requires builder-rpc-v0

- GIVEN a concrete Nix derivation requires `builder-rpc-v0`
- WHEN the Nix producer classifies required system features
- THEN it MUST return a stable `nix-required-system-feature-unsupported` blocker that names the bounded source feature
- AND it MUST NOT publish a successful graph or package index for that request

#### Scenario: A required feature is unknown or malformed

- GIVEN a required system feature is unknown, duplicated inconsistently, not valid text, or malformed in structured attributes
- WHEN the adapter parses and classifies the feature set
- THEN it MUST fail closed with a deterministic feature diagnostic
- AND it MUST NOT omit the requirement or treat the derivation as an ordinary local build

#### Scenario: Core input contains a Nix protocol type

- GIVEN Nix daemon, Varlink, descriptor-passing, or builder-RPC types enter a frontend-neutral graph, plan, profile, receipt, or core command
- WHEN the compatibility architecture guard runs
- THEN it MUST reject the provider type and name the owning Nix adapter
- AND the core contract MUST remain expressed with Mantle-owned execution facts
