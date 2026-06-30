# Realization Routing Specification

## Purpose

Defines the `realization-routing` capability.

## Requirements

### Requirement: Mantle plans build realization routes deterministically [r[realization_routing.route_plan_model]]

Mantle MUST plan requested build realization through a deterministic route model before mutating work starts. The route model MUST consume bounded facts about requested roots, local store state, source-bundle readiness, archive candidates, substituter trust, remote-builder capabilities, local executor capability, network policy, and requested evidence strength. It MUST return a selected route or fail-closed route error plus rejected-route reason codes without reading hidden global state in the pure planner.

#### Scenario: Local cache route is selected deterministically [r[realization_routing.route_plan_model.scenario.local-cache]]

- GIVEN every requested output has accepted local PathInfo, content, and required attestation facts
- WHEN Mantle plans realization routes for those roots
- THEN the planner MUST select the local cache route
- AND it MUST record that no remote, archive, or build execution route is required.

#### Scenario: Rejected alternatives have reason codes [r[realization_routing.route_plan_model.scenario.rejected-reasons]]

- GIVEN multiple possible realization routes are ineligible for different reasons
- WHEN Mantle renders the route plan
- THEN the plan MUST include deterministic reason codes for rejected alternatives
- AND those reason codes MUST be stable for equivalent input facts.

### Requirement: Route ranking is deterministic and replayable [r[realization_routing.deterministic_route_ranking]]

Mantle MUST rank eligible realization routes with explicit deterministic tie-breakers. Equivalent route facts MUST select the same route regardless of remote discovery order, map iteration order, latency, wall-clock timing, random identifiers, or previous failed attempts unless those facts are explicitly included in the route input model and report.

#### Scenario: Equal facts choose same route [r[realization_routing.deterministic_route_ranking.scenario.stable]]

- GIVEN two route-planning runs have equivalent requested roots and equivalent local/cache/archive/source/builder/executor facts
- WHEN Mantle ranks eligible routes
- THEN both runs MUST select the same route and rejected-route ordering
- AND the report MUST identify the tie-breaker class when more than one route is otherwise equivalent.

#### Scenario: Latency does not silently choose route [r[realization_routing.deterministic_route_ranking.scenario.no-latency-race]]

- GIVEN multiple remote or archive candidates are eligible
- WHEN one candidate answers discovery faster than another
- THEN Mantle MUST NOT select the route solely because it answered first
- AND any latency-aware preference MUST be an explicit configured fact recorded in the route report.

### Requirement: Offline realization fails closed on network or undeclared input needs [r[realization_routing.offline_fail_closed]]

Mantle MUST treat offline realization mode as a fail-closed policy. In offline mode, a route MUST be ineligible when it requires live network access, remote cache lookup, remote builder connection, source download, VCS fetch, language package-manager access, ambient package-manager or build-cache use, undeclared sibling checkout reads, or output import without configured trust.

#### Scenario: Complete local material permits offline route [r[realization_routing.offline_fail_closed.scenario.local-ready]]

- GIVEN offline mode is selected
- AND every required output or build input is available through accepted local store state, imported source state, or trusted local archive material
- WHEN Mantle plans realization routes
- THEN the planner MAY select a local cache, archive import, or local build route that does not require network access
- AND the report MUST bind the local material facts used for that decision.

#### Scenario: Network-required route is rejected offline [r[realization_routing.offline_fail_closed.scenario.reject-network]]

- GIVEN offline mode is selected
- AND the only otherwise viable route requires a substituter query, remote builder session, VCS fetch, language package-manager access, source download, or undeclared cache read
- WHEN Mantle plans realization routes
- THEN Mantle MUST reject that route before execution
- AND diagnostics MUST identify the network or undeclared-input requirement that blocks offline realization.

### Requirement: Route eligibility respects requested claim strength [r[realization_routing.claim_strength_routing]]

Mantle MUST evaluate realization routes against the requested claim strength. Practical build routes MAY require only ordinary signed PathInfo and artifact attestation evidence, while strong action-correctness or release-facing routes MUST require matching action refs, input/source refs, sandbox policy evidence, network policy evidence, reference-scan evidence, producer policy, and trusted receipts before reuse or remote/offline acceptance.

#### Scenario: Practical route is downgraded for strong claim [r[realization_routing.claim_strength_routing.scenario.downgrade]]

- GIVEN a route can produce or reuse an output with ordinary signed PathInfo but lacks action-correctness receipts or sandbox evidence required by the requested claim strength
- WHEN Mantle plans a strong action-correctness route
- THEN Mantle MUST reject or downgrade that route with a deterministic claim-strength blocker
- AND it MUST NOT report strong correctness eligibility from practical evidence alone.

#### Scenario: Strong route requires matching receipts [r[realization_routing.claim_strength_routing.scenario.strong-match]]

- GIVEN a route supplies matching action refs, input refs, output object refs, sandbox/network policy evidence, reference-scan evidence, producer policy, and trusted receipt signatures
- WHEN Mantle plans a strong route for the requested roots
- THEN Mantle MAY mark that route strong-eligible
- AND the report MUST identify the receipt and trust-basis facts used for the decision.

### Requirement: Remote-builder route eligibility is explicit [r[realization_routing.remote_builder_eligibility]]

Mantle MUST mark a P2P remote-builder route eligible only when the client has concrete evaluated build inputs, the builder capabilities match the requested platform/profile/hermeticity needs, missing input sync is feasible within configured limits, and the client has an explicit output trust path for returned PathInfo and attestations. Builder resource authorization MUST NOT count as output trust.

#### Scenario: Compatible trusted builder is eligible [r[realization_routing.remote_builder_eligibility.scenario.eligible]]

- GIVEN the client has concrete frontend-neutral build inputs for the requested roots
- AND a configured builder advertises compatible platform, hermeticity, transfer, queue, and resource capabilities
- AND source/input upload is feasible within limits
- AND the client trusts the builder output signing key or required attestation authority
- WHEN Mantle plans realization routes
- THEN the planner MAY mark the remote-builder route as eligible
- AND the report MUST identify the capability and trust facts without revealing bearer ticket secrets.

#### Scenario: Resource access without output trust is ineligible [r[realization_routing.remote_builder_eligibility.scenario.no-output-trust]]

- GIVEN the client has a valid remote-builder ticket or trusted-client authorization
- AND the client lacks configured trust for the builder's returned output signing key or attestation authority
- WHEN Mantle plans remote-builder eligibility
- THEN Mantle MUST reject the remote-builder route as an output-trust blocker
- AND it MUST NOT treat resource authorization as sufficient to accept remote outputs.

#### Scenario: Raw frontend evaluation request is not routable [r[realization_routing.remote_builder_eligibility.scenario.no-remote-eval]]

- GIVEN a requested remote route would require the builder to evaluate raw Nickel source, Onix module semantics, or frontend-specific configuration
- WHEN Mantle plans remote-builder eligibility
- THEN Mantle MUST reject the route before remote dispatch
- AND diagnostics MUST state that remote builders require concrete Mantle build inputs.

### Requirement: Remote upload planning is privacy-safe [r[realization_routing.remote_upload_privacy]]

Mantle MUST summarize and enforce remote-upload privacy policy before selecting a remote-builder route. Upload planning MUST identify source, store, proof, and secret-descriptor classes, byte counts, and object counts in bounded form, and it MUST reject routes that would upload disallowed classes or raw secret material.

#### Scenario: Upload summary precedes remote dispatch [r[realization_routing.remote_upload_privacy.scenario.summary]]

- GIVEN a remote-builder candidate requires missing input upload
- WHEN Mantle plans the route
- THEN the route report MUST summarize upload classes, object counts, and byte counts before dispatch
- AND it MUST omit bearer tickets, private key paths, decrypted secret bytes, raw environment values, and unbounded path lists.

#### Scenario: Upload privacy policy rejects route [r[realization_routing.remote_upload_privacy.scenario.reject]]

- GIVEN a remote-builder candidate would upload a source, store, proof, or secret-descriptor class disallowed by configured policy
- WHEN Mantle evaluates route eligibility
- THEN Mantle MUST reject the remote-builder route before opening the session or uploading bytes
- AND diagnostics MUST identify the disallowed upload class without revealing the protected content.

### Requirement: Route reports are bounded diagnostic evidence [r[realization_routing.route_report]]

Mantle MUST render route plans as bounded diagnostic evidence. Human and JSON reports MUST identify the selected route, rejected alternatives, source gaps, trust blockers, archive candidates, remote-builder candidates, and requested policy modes. Route reports MUST redact bearer tickets, private key paths, raw environment values, and full argument strings, and they MUST NOT claim build success or output trust before downstream evidence exists.

#### Scenario: JSON route report remains parseable and redacted [r[realization_routing.route_report.scenario.json-redacted]]

- GIVEN route planning uses substituters, archives, source bundles, remote builder tickets, trusted keys, or environment-derived configuration
- WHEN Mantle emits a JSON route report
- THEN the report MUST remain valid JSON with stable field names
- AND it MUST omit bearer secrets, private key paths, raw environment values, and unbounded log or argv payloads.

#### Scenario: Route report does not overclaim [r[realization_routing.route_report.scenario.non-claim]]

- GIVEN Mantle reports a selected route or eligible remote/archive/substitution candidate
- WHEN a task, evidence file, status reply, or human output cites that route report
- THEN the claim MUST be limited to planning eligibility under the supplied facts
- AND it MUST NOT claim successful build execution, archive import, remote execution, output trust, substitution acceptance, or reproducibility without separate evidence.
