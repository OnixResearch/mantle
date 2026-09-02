# Nickel Toolchain Specification

## Purpose

Defines the `nickel-toolchain` capability.

## Requirements

### Requirement: Mantle pins the complete Nickel 1.17 cohort

r[mantle.nickel_toolchain.cohort] Mantle MUST pin Nickel CLI `1.17.0`, `nickel-lang 2.2.0`, and `nickel-lang-core 0.18.0` from the reviewed upstream release.

#### Scenario: Embedded and command-line pins match

- GIVEN Mantle builds its evaluator and its configuration checks
- WHEN dependency resolution completes
- THEN all Nickel surfaces MUST belong to the declared cohort

#### Scenario: A mixed cohort is present

- GIVEN any embedded or command-line Nickel surface resolves to an older cohort
- WHEN the cohort guard runs
- THEN the guard MUST fail

### Requirement: Vendored Nickel source has exact provenance

r[mantle.nickel_toolchain.vendor] Mantle MUST refresh vendored Nickel source through its repository-owned importer. The result MUST bind commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`, a file manifest, checksums, and licenses.

#### Scenario: The vendor refresh is complete

- GIVEN the importer reads the reviewed upstream source
- WHEN it writes the vendored snapshot
- THEN the snapshot MUST reproduce its declared manifest
- AND preserved license checks MUST pass

#### Scenario: A vendored file changes outside the importer

- GIVEN vendored Nickel content differs from its recorded manifest
- WHEN source admission runs
- THEN admission MUST fail

### Requirement: Evaluation compatibility remains fail-closed

r[mantle.nickel_toolchain.compatibility] Mantle MUST test parsing, contracts, imports, direct deserialization, diagnostic redaction, and evaluation budgets with positive and negative fixtures.

#### Scenario: A valid derivation evaluates

- GIVEN a supported Mantle derivation fixture
- WHEN the new evaluator processes it
- THEN the decoded Mantle value MUST retain its supported meaning

#### Scenario: Invalid or excessive input is evaluated

- GIVEN malformed source, a failed contract, a missing import, an oversized input, or a budget breach
- WHEN evaluation runs
- THEN Mantle MUST return a stable bounded error
- AND no build effect MUST start

### Requirement: Nickel API changes remain in the adapter

r[mantle.nickel_toolchain.boundary] Mantle MUST confine upstream API changes to evaluator adapters. Build, store, scheduler, and evidence cores MUST NOT depend on upstream runtime types.

#### Scenario: Upstream diagnostics change

- GIVEN Nickel changes an internal diagnostic representation
- WHEN Mantle adapts the new library
- THEN Mantle MUST normalize it at the evaluator boundary
- AND repository-owned stable error classes MUST remain authoritative

### Requirement: Evidence binds the evaluator cohort

r[mantle.nickel_toolchain.evidence] Mantle bootstrap and release evidence MUST record the exact source commit, crate versions, CLI version, Rust requirement, and vendor manifest identity.

#### Scenario: Evidence is reviewed

- GIVEN all focused compatibility and source checks pass
- WHEN release evidence is generated
- THEN the evaluator cohort MUST be explicit
- AND evidence MUST retain Mantle's bootstrap and correctness non-claims

### Requirement: The repository validation rail passes

r[mantle.nickel_toolchain.validation] The change MUST pass vendor checks, focused positive and negative tests, formatting, Clippy, Cairn gates, and relevant Nix checks.

#### Scenario: Validation completes

- GIVEN the source pin, vendor snapshot, adapters, fixtures, and evidence are current
- WHEN the validation rail runs
- THEN every required check MUST pass or report one exact blocker
