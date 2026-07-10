# Release Provenance Specification

## Purpose

Add a Mantle CLI surface for function-address release evidence binding.

## Requirements

### Requirement: CLI binds function-address evidence to release artifacts
r[mantle.release_provenance.function_address_binding_cli.command] Mantle MUST provide a CLI command that binds Valence/Kamacite function-address evidence to release source and binary artifact identities.

#### Scenario: Required binding passes
r[mantle.release_provenance.function_address_binding_cli.positive]
- GIVEN a release manifest has matching source archive and release binary digests, Valence function-address evidence, and Kamacite receipt metadata
- WHEN Mantle validates function-address release evidence in required mode
- THEN the CLI MUST emit a passing binding receipt.

### Requirement: Binding CLI shell owns filesystem effects
r[mantle.release_provenance.function_address_binding_cli.shell] Mantle CLI file reads, path resolution, stdout/stderr, and receipt writes MUST remain outside `crunch-release-core` validation logic.

#### Scenario: Core receives typed release evidence
r[mantle.release_provenance.function_address_binding_cli.receipt]
- GIVEN the CLI has loaded release artifact metadata and external evidence metadata
- WHEN validation runs
- THEN `crunch-release-core` MUST evaluate typed in-memory release evidence and return deterministic diagnostics.

### Requirement: Invalid binding fails closed
r[mantle.release_provenance.function_address_binding_cli.negative] Mantle MUST reject function-address binding inputs with missing evidence, stale digest, wrong role, wrong schema, unsupported claim scope, source/binary mismatch, or overclaiming non-claim text.

#### Scenario: Stack smoke uses Mantle-owned receipt
r[mantle.release_provenance.function_address_binding_cli.validation]
- GIVEN Octet, Kamacite, and Valence produce function-address evidence
- WHEN Mantle binds the evidence to release artifacts
- THEN the binding receipt MUST remain identity/linkage-only and suitable for Cairn readiness input.
