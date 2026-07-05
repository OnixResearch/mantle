## ADDED Requirements

### Requirement: Source-bundle route execution is selectable

r[realization_routing.source_bundle_route_execution] Mantle MUST treat source-bundle input realization as a deterministic build realization route when imported source state can satisfy all missing declared source/fetcher inputs for a selected root. The route planner MUST mark `source-bundle` eligible only from explicit source readiness facts, MUST reject it with stable reason codes when readiness is incomplete, and MUST prefer it over live network fetch routes in offline mode.

#### Scenario: ready source state selects source-bundle route

GIVEN a requested root is missing source/fetcher inputs but not final outputs
AND offline source preflight reports every required source record as ready, pinned, trusted, and identity-matched
WHEN Mantle plans realization routes in offline mode
THEN the route planner MAY select or report the `source-bundle` route for input realization
AND the selected route MUST bind the source-state or source-bundle digest used for the decision.

#### Scenario: source-bundle gaps reject route before network

GIVEN offline mode is selected
AND source-bundle readiness is missing, stale, unpinned, unsupported, untrusted, or network-required for at least one selected root
WHEN Mantle plans routes
THEN the planner MUST reject the `source-bundle` route with deterministic reason codes
AND it MUST also reject live fetch, substituter, and remote-builder routes that would require network access.

#### Scenario: route report does not claim output reuse

GIVEN a route report selects or marks eligible a `source-bundle` route
WHEN a human report, JSON report, task, or evidence file cites that route
THEN the claim MUST be limited to source/input realization eligibility under the supplied facts
AND it MUST NOT claim cached output reuse, substitution acceptance, build execution success, or output trust.
