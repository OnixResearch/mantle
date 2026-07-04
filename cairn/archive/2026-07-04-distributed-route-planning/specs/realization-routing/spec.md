## ADDED Requirements

### Requirement: Build plans include remote-builder route eligibility

r[realization_routing.remote_route_plan_cli] Mantle MUST include P2P/stdio remote-builder route eligibility in deterministic build plan reports before mutating work starts. The planner MUST consume only explicit bounded facts about concrete build inputs, builder capabilities, source/input readiness, upload budgets, network policy, output trust, and requested claim strength. Planning MUST NOT open remote sessions, redeem tickets, upload bytes, or claim remote execution success.

#### Scenario: compatible remote route is reported eligible

GIVEN the requested roots have concrete Mantle build inputs
AND a configured remote builder advertises compatible system, sandbox, network, transfer, queue, and resource capabilities
AND source/input upload is feasible within configured limits
AND the client has explicit output trust for the builder signing key or attestation authority
WHEN Mantle renders a build plan
THEN the remote-builder route MAY be reported eligible
AND the report MUST identify the capability, upload, and trust facts without revealing bearer ticket secrets.

#### Scenario: remote route blockers are deterministic

GIVEN a remote route lacks concrete inputs, output trust, source readiness, upload budget, network permission, or a matching capability
WHEN Mantle renders a build plan
THEN the remote-builder route MUST be rejected before dispatch
AND the report MUST include stable rejected-route reason codes for the blockers.

#### Scenario: route planning is a non-claim

GIVEN Mantle reports a remote-builder route as selected or eligible during plan mode
WHEN a human report, JSON report, task, or evidence file cites that plan
THEN the claim MUST be limited to route eligibility under the supplied facts
AND it MUST NOT claim remote session success, input upload, build execution, output transfer, output trust admission, or reproducibility.
