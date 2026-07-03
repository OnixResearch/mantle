# Foreign Derivation Import Specification

## Purpose

Defines CLI requirements for exposing adapter-neutral foreign derivation import artifacts to operators.

## Requirements

### Requirement: Operator CLI surface for foreign imports

r[foreign_derivation_import.operator_cli_surface] Mantle MUST provide a thin operator CLI for validating and planning from foreign derivation import artifacts without requiring the foreign frontend at consumption time.

#### Scenario: Operator validates lowered artifacts

GIVEN a foreign derivation graph artifact, optional package index, and translation policy file
WHEN an operator runs the validation command
THEN Mantle MUST validate the artifacts through the import core and report deterministic diagnostics
AND it MUST NOT require Guix, Nix, flake evaluation, Nix expression evaluation, overlays, or package-module evaluation.

#### Scenario: Operator emits an adapter plan

GIVEN valid lowered artifacts and a target package/root selection
WHEN an operator runs the planning command
THEN Mantle MUST emit a receipt-bound adapter plan JSON document
AND the plan MUST identify translated roots, policy digests, source acquisition hints, sandbox audit events, and explicit non-claims.

### Requirement: CLI file boundary stays outside the core

r[foreign_derivation_import.cli_file_boundary] Foreign import CLI integration MUST keep filesystem reads, JSON decoding, stdout/stderr rendering, and process exit behavior in the imperative shell while preserving the translation core as pure in-memory logic.

#### Scenario: Core remains file-system independent

GIVEN the CLI reads artifact files from disk
WHEN it invokes translation or admission logic
THEN it MUST pass owned in-memory data into the core
AND the core MUST NOT read files, inspect environment variables, execute processes, or render CLI JSON directly.

#### Scenario: CLI failure reports core diagnostics

GIVEN an input artifact is invalid
WHEN the CLI reports the failure
THEN it MUST preserve the core diagnostic class and path in machine-readable output
AND it SHOULD bound human-readable summaries.

### Requirement: Checked fixtures exercise the CLI contract

r[foreign_derivation_import.checked_fixtures] Mantle MUST ship small checked-in Guix-like and Nix-like foreign import fixtures that can validate the CLI contract without live foreign frontends.

#### Scenario: Guix-like fixture plans without Guix

GIVEN a checked-in Guix-like hello fixture lowered into the foreign import IR
WHEN the CLI validates and plans the fixture
THEN the command MUST succeed without executing `guix`
AND the emitted receipt MUST bind the fixture graph and policy digests.

#### Scenario: Nix-like fixture plans without Nix

GIVEN a checked-in Nix-like hello fixture lowered from `.drv` or derivation-JSON facts
WHEN the CLI validates and plans the fixture
THEN the command MUST succeed without executing `nix`, `nix-store`, flake evaluation, Nix expression evaluation, or overlay application
AND the emitted receipt MUST bind the fixture graph and policy digests.
