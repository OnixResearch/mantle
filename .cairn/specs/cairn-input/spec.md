# Cairn Input Specification

## Purpose

Defines the `cairn-input` capability.

## Requirements

### Requirement: Pinned cairn toolchain
r[mantle.cairn_input.pinned] Mantle MUST provide the `cairn` binary for lifecycle commands through one immutable flake-pinned revision rather than an ambient profile installation.

#### Scenario: Dev shell resolves the pinned binary
r[mantle.cairn_input.pinned.scenario.resolved]
- GIVEN the repository development shell
- WHEN the operator resolves `which cairn`
- THEN the path resolves inside the Nix store build of the pinned revision

### Requirement: Cairn input validation
r[mantle.cairn_input.validation] Lifecycle commands MUST parse the committed generated policy without manual environment overrides.

#### Scenario: Lifecycle commands parse the committed policy
r[mantle.cairn_input.validation.scenario.policy_parses]
- GIVEN the committed `cairn-policy/generated/cairn-policy.json`
- WHEN `cairn validate --root .` runs inside the development shell
- THEN validation succeeds
