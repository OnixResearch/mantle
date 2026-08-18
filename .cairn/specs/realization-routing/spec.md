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

### Requirement: Typed portable client command matrix

r[realization_routing.portable_client_command_matrix] Mantle MUST maintain a typed, versioned command and platform matrix that classifies portable client, Linux local-executor, worker, server, bootstrap, proof, and unsupported operations with explicit filesystem, network, process, trust, and mutation effects.

#### Scenario: Darwin client invokes a supported command

- **GIVEN** the selected Darwin platform and command have a supported portable-client row
- **WHEN** command admission runs
- **THEN** Mantle MUST admit only the effects and dependencies declared by that row
- **AND** diagnostics MUST identify any required remote capability or trust input before execution

#### Scenario: Darwin client invokes a Linux-only command

- **GIVEN** the selected command requires worker, local sandbox, seccomp, bootstrap, self-build, or Linux proof capability
- **WHEN** command admission runs on Darwin
- **THEN** Mantle MUST reject it with the matrix-owned stable blocker before side effects
- **AND** it MUST not discover or invoke Linux execution tools

### Requirement: Portable remote client core

r[realization_routing.portable_client_core] Portable remote request planning, route decisions, upload classification, response-admission inputs, and client report construction MUST be pure deterministic logic over explicit bounded values. Filesystem, network, credential, local-store, and output-write effects MUST remain in a thin portable shell.

#### Scenario: Equivalent client facts produce equivalent plan

- **GIVEN** equivalent concrete build, route, capability, upload, trust, and policy facts
- **WHEN** the portable core plans remote realization
- **THEN** it MUST produce the same route, request, upload summary, and ordered diagnostics
- **AND** client platform path layout MUST not change action or target identity

#### Scenario: Shell fact is missing

- **GIVEN** the shell cannot provide required source, credential, output-trust, or local-store facts
- **WHEN** the core evaluates admission
- **THEN** it MUST return a deterministic blocker
- **AND** the shell MUST not fabricate a default authority or hidden ambient input

### Requirement: Non-Linux remote route selection

r[realization_routing.non_linux_remote_route] Mantle MUST treat local executor capability as one explicit route fact. A non-Linux client MAY select an eligible remote realization route without a local Linux executor, and local executor ineligibility MUST NOT become a process-wide fatal error when another route is eligible.

#### Scenario: Remote route is eligible on macOS

- **GIVEN** local execution is unsupported and concrete inputs, source readiness, upload policy, worker capability, credentials, and output trust admit one remote route
- **WHEN** `mantle build` plans and executes realization
- **THEN** it MUST select the admitted remote route
- **AND** it MUST not require bubblewrap, FUSE, seccomp, user namespaces, or a `/nix` volume on the client

#### Scenario: No route is eligible

- **GIVEN** local execution is unsupported and every cache, import, and remote route has one or more blockers
- **WHEN** realization planning completes
- **THEN** Mantle MUST return the ordered route blockers
- **AND** it MUST not report generic Linux-only failure in place of the specific remote, source, upload, or trust blockers

### Requirement: Portable clients send concrete inputs only

r[realization_routing.portable_client_concrete_inputs] A portable client MUST evaluate project and Nickel inputs locally and MUST send only concrete frontend-neutral build requests, immutable source or object refs, policy identities, selected target facts, and bounded upload plans to remote workers.

#### Scenario: Client and target systems differ

- **GIVEN** a Darwin client selects a Linux target and produces concrete target build inputs
- **WHEN** the remote request is canonicalized
- **THEN** action and route identity MUST bind the selected Linux target independently from the Darwin client platform
- **AND** the worker MUST receive no authority to infer target from the client host

#### Scenario: Request contains raw frontend authority

- **GIVEN** a remote request contains raw Nickel, Onix modules, Nix expressions, flakes, or package-manager resolution instructions
- **WHEN** portable request admission runs
- **THEN** Mantle MUST reject the request before connection or upload
- **AND** it MUST not ask the worker to evaluate or lower the frontend data

### Requirement: No local execution on portable remote route

r[realization_routing.no_local_execution_on_portable_client] A selected portable remote route MUST NOT initialize or invoke the local builder, bubblewrap, FUSE, seccomp, cgroup, protected-exec, worker-server, bootstrap, or proof paths on the client.

#### Scenario: Remote build succeeds

- **GIVEN** local Linux execution seams are instrumented with failure sentinels and an admitted remote worker returns a valid result
- **WHEN** the portable client completes the build
- **THEN** all local execution sentinels MUST remain untouched
- **AND** success MUST derive only from remote result admission and optional local materialization

#### Scenario: Remote preflight fails

- **GIVEN** remote capability, credential, upload, or trust preflight rejects the route
- **WHEN** the portable client reports failure
- **THEN** local execution sentinels MUST remain untouched
- **AND** no fallback to local execution MAY occur on an unsupported platform

### Requirement: Portable output materialization

r[realization_routing.portable_output_materialization] A portable client MUST support report-only remote completion and MAY support local output materialization through the ordinary verified object, PathInfo, signature, prefix, closure, and attestation admission path into an explicit unprivileged physical store.

#### Scenario: Admitted output is materialized

- **GIVEN** the remote result passes current fence, signer, object, PathInfo, prefix, closure, and attestation policy
- **WHEN** the operator requests local materialization
- **THEN** Mantle MUST import accepted state before writing the physical output
- **AND** the logical store identity MUST remain independent from the client physical directory

#### Scenario: Output bytes are corrupt

- **GIVEN** a returned object, NAR, PathInfo, signature, reference, or attestation does not match the admitted result
- **WHEN** the portable client validates materialization
- **THEN** it MUST reject before publishing the physical output
- **AND** report-only completion MUST not be upgraded to locally materialized success

### Requirement: Portable credential and trust boundary

r[realization_routing.portable_client_credentials] Portable clients MUST use explicit caller-owned credential files or supported secure platform handles and MUST keep execution, upload, log, cancellation, output signer, admission, and publication authority separate. Secret material MUST NOT enter plans, reports, logs, arguments, store objects, or diagnostics.

#### Scenario: Authorization and output trust both pass

- **GIVEN** the client has admitted execution authority and separate trust for the returned signer and output facts
- **WHEN** remote result admission runs
- **THEN** Mantle MAY accept the result under current policy
- **AND** reports MUST identify public authority classes without exposing bearer material

#### Scenario: Ticket is valid but signer is not trusted

- **GIVEN** execution authorization succeeds but returned output signer trust is absent or mismatched
- **WHEN** response admission runs
- **THEN** Mantle MUST reject the output
- **AND** the valid ticket MUST NOT count as output trust

### Requirement: Portable client validation

r[realization_routing.portable_client_validation] Portable client support MUST include native Darwin and Linux positive and negative fixtures, supplemental cross-target checks, dependency-boundary checks, no-local-execution sentinels, protocol and trust tests, output-materialization tests, and secret scans.

#### Scenario: Native platform matrix passes

- **GIVEN** supported native Darwin and Linux runners with the declared client command matrix
- **WHEN** portable validation runs
- **THEN** supported commands MUST produce their expected plans and outcomes
- **AND** unsupported commands MUST fail before their prohibited effects

#### Scenario: Cross-target check passes without native evidence

- **GIVEN** Darwin targets compile from a Linux cross-target check but native command fixtures are absent or stale
- **WHEN** support status is summarized
- **THEN** Mantle MUST report compilation evidence separately from native platform evidence
- **AND** it MUST not claim checked Darwin support from cross-compilation alone
