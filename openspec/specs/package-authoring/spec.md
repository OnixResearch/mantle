# Package Authoring Specification

## Purpose

Defines requirements for the builder-layer Nickel interfaces and generated
project inputs that package authors use to describe Mantle packages.

## Requirements

### Requirement: Locked project inputs are importable from generated state

Package Nickel code MUST be able to import locked project inputs from the
project layer's generated file, `.mantle/inputs.ncl`, instead of hand-writing
source records or seed files for every pinned external input.

#### Scenario: Package imports generated locked inputs

- GIVEN a project with a current `mantle.lock`
- AND `.mantle/inputs.ncl` generated from that lock
- WHEN a package Nickel file imports `.mantle/inputs.ncl`
- THEN the package can reference locked inputs from that file
- AND those inputs correspond to the current lock state

#### Scenario: Generated locked inputs replace hand-maintained source records

- GIVEN a package that previously repeated pinned source metadata in local
  Nickel code
- WHEN the package is updated to import `.mantle/inputs.ncl`
- THEN the package no longer needs duplicate hand-maintained source records for
  those locked project inputs

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
- THEN mantle records those claims in the final artifact attestation
- AND changing only those claim fields does not change the derivation hash by default

#### Scenario: Core derivation contract stays minimal

- GIVEN the core mantle derivation contract in `lib/`
- WHEN it is inspected after provenance support is added
- THEN the core contract still defines build-engine fields only
- AND builder-level provenance claims are added in the package-authoring layer instead
