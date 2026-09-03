# Realization Routing Planning-Core Delta

## ADDED Requirements

### Requirement: Route planning consumes explicit observations only

r[realization_routing.explicit_observation_boundary] Mantle MUST make local output, source readiness, substituter, archive, remote candidate, doctor, platform, trust, network, claim-strength, and executor facts explicit bounded inputs to route policy. The route core MUST NOT read stores, files, environment state, keys, clocks, remote services, or provider state.

#### Scenario: Complete observations select a route

GIVEN the shell supplies complete admitted route observations within named bounds
WHEN the route core evaluates eligibility and preference
THEN it MUST return one selected route or a typed fail-closed blocker plus ordered rejected-route reasons
AND equivalent facts MUST produce the same result regardless of observation order or host timing.

#### Scenario: Required observation is missing

GIVEN a route needs source, trust, store, executor, network, or remote capability facts that the shell did not supply
WHEN the route core evaluates the request
THEN it MUST return a stable missing-fact blocker
AND it MUST NOT inspect ambient state or fabricate a permissive default.

### Requirement: Route plans remain separate from execution

r[realization_routing.plan_execution_separation] A route decision MUST return a bounded typed effect plan and MUST NOT open a remote session, redeem credentials, query a substituter, mutate a store, execute a build, or publish an output. The shell MUST execute the accepted plan and record observations separately.

#### Scenario: Remote route is selected

GIVEN explicit facts make one remote route eligible and preferred
WHEN route planning completes
THEN the result MUST identify the route, rejected alternatives, trust basis, upload summary, and required effects
AND it MUST not claim session, transfer, execution, or output-admission success.

#### Scenario: Bound facts drift before mutation

GIVEN the shell accepted a route plan and a bound trust, source, store, or capability fact changes before a mutating effect
WHEN execution rechecks the plan preimage
THEN it MUST reject the stale plan with a typed drift blocker where policy requires freshness
AND it MUST not silently re-plan or execute a different route.

### Requirement: Build-planning blockers are typed

r[realization_routing.typed_planning_blockers] Mantle MUST preserve store absence, source gaps, output-trust gaps, unsupported platform, offline network need, executor absence, and stale-plan state as separate route blocker variants until the presentation boundary.

#### Scenario: Several routes fail for different reasons

GIVEN local cache, substitution, remote, and local execution each have different blockers
WHEN route planning fails
THEN the result MUST retain each stable blocker class in deterministic route order
AND a generic shell or CLI error string MUST NOT replace the typed planning result.
