# Release Provenance Specification

## Purpose

Define a stable Mantle-owned function-address release binding receipt schema.

## Requirements

### Requirement: Function-address binding receipt schema is versioned
r[mantle.release_provenance.function_address_binding_schema.contract] Mantle MUST define a versioned function-address release binding receipt schema for Cairn readiness consumption.

#### Scenario: Binding receipt has required fields
r[mantle.release_provenance.function_address_binding_schema.positive]
- GIVEN Mantle validates function-address release evidence successfully
- WHEN it renders the binding receipt
- THEN the receipt MUST include validity, verdict, receipt hash, claim scope, sidecar digest, Valence receipt digest, source archive digest, release binary digest, optional Kamacite digest, and non-claim boundaries.

### Requirement: Binding receipt maps to Cairn readiness input
r[mantle.release_provenance.function_address_binding_schema.cairn_mapping] Mantle SHOULD document how each binding receipt field maps to Cairn function-address readiness validation.

#### Scenario: Cairn extracts fields without wrapper glue
r[mantle.release_provenance.function_address_binding_schema.validation]
- GIVEN a Mantle function-address binding receipt follows the versioned schema
- WHEN Cairn release-readiness consumes it
- THEN Cairn SHOULD read the receipt directly without a smoke-specific wrapper.

### Requirement: Malformed binding receipts fail closed
r[mantle.release_provenance.function_address_binding_schema.negative] Mantle MUST reject or mark invalid binding receipts with missing required fields, malformed BLAKE3 hashes, stale Valence/Kamacite digests, unsupported claim scope, or weakened non-claim boundaries.

#### Scenario: Stable renderer preserves release verification
r[mantle.release_provenance.function_address_binding_schema.render]
- GIVEN Mantle renders a binding receipt
- WHEN an operator inspects it
- THEN the receipt MUST preserve the underlying release verification disposition and diagnostics without adding semantic correctness claims.
