# Realization Routing Portable Client Delta

## ADDED Requirements

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
