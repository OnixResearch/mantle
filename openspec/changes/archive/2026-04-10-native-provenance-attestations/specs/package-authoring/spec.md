## ADDED Requirements

### Requirement: Builder-layer provenance claims metadata

The package-authoring layer MUST allow package definitions to attach optional
structured provenance claims metadata without changing derivation hashes by
default.

This metadata MUST live in the builder-layer package contract rather than the
minimal core derivation contract. It MUST support structured claim fields for
package identity and publisher intent, such as component name overrides,
version claims, supplier, homepage, license, and source aliases.

#### Scenario: Package defines provenance claims without changing derivation hash

- GIVEN a package definition with optional provenance claims metadata
- WHEN the package is converted into a derivation and built
- THEN crunch records those claims in the final artifact attestation
- AND changing only those claim fields does not change the derivation hash by default

#### Scenario: Core derivation contract stays minimal

- GIVEN the core crunch derivation contract in `lib/`
- WHEN it is inspected after provenance support is added
- THEN the core contract still defines build-engine fields only
- AND builder-level provenance claims are added in the package-authoring layer instead
