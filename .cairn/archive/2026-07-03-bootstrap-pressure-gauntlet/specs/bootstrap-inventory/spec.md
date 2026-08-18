## ADDED Requirements

### Requirement: Bootstrap gauntlet inventory pressure

r[bootstrap_inventory.bootstrap_gauntlet_inventory_pressure] Mantle bootstrap pressure profiles that claim no-host-tools or protected-exec coverage MUST bind every permitted protected-phase executable to a declared inventory entry.

#### Scenario: declared executable inventory is complete

GIVEN a bootstrap pressure profile enters a protected phase
WHEN protected exec observes an executable path
THEN the path MUST match a host prerequisite, pinned fetched artifact, or Mantle-built output declared in the profile inventory
AND the audit report MUST bind the inventory digest and observed executable digest.

#### Scenario: missing inventory entry fails closed

GIVEN protected exec observes an executable that is not declared by the active profile inventory
WHEN the bootstrap pressure profile evaluates the phase
THEN the profile MUST fail closed before recording no-host-tools success
AND the diagnostic MUST identify the undeclared executable class without promoting the profile.
