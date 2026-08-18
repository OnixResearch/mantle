## ADDED Requirements

### Requirement: Host-tool attestation inventory

r[build_correctness.host_tool_attestation_inventory] Mantle MUST require any host executable used by strict proof or sandbox setup paths to be declared in a host-tool inventory with role, absolute path, BLAKE3 digest, bounded version evidence, and provenance before that executable can affect proof evidence.

#### Scenario: declared host tool is accepted

GIVEN a host-tool inventory declares an executable role, absolute path, BLAKE3 digest, bounded version evidence, and provenance note
WHEN Mantle validates the inventory and observes that executable in a strict proof path
THEN the observation MUST match the accepted inventory record
AND proof reports MUST bind the inventory digest and accepted tool role.

#### Scenario: undeclared execution is denied

GIVEN protected execution observes an executable path with no accepted inventory record or Mantle-built sandbox transition record
WHEN the strict proof path evaluates the observation
THEN Mantle MUST deny execution or fail the proof before accepting evidence
AND the diagnostic MUST identify the undeclared executable class.

#### Scenario: digest drift blocks proof

GIVEN an inventory record names a host executable but its current bytes, path kind, or bounded version evidence no longer match
WHEN Mantle validates the inventory before strict proof execution
THEN validation MUST fail closed with a deterministic inventory-drift diagnostic
AND Mantle MUST NOT fall back to name-only or path-only trust.
