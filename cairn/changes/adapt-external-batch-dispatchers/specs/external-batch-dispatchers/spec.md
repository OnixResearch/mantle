# External Batch Dispatchers Specification

## Purpose

Allow Mantle to place fenced remote build attempts through existing external batch schedulers while preserving provider-neutral planning, CAS-based transfer, current worker authority, secret boundaries, and ordinary output admission.

## Requirements

### Requirement: External dispatch uses a bounded canonical protocol [r[external_batch_dispatchers.protocol]]

Mantle MUST communicate with an external batch dispatcher through a versioned bounded protocol for submit, observe, cancel, and reconcile operations. Canonical requests and responses MUST bind operation identity, dispatcher profile, realization/job/attempt/fence refs, normalized provider state, bounded locator metadata, and non-claims; Mantle-owned protocol record identities MUST use domain-separated BLAKE3.

#### Scenario: Equivalent submit requests are identical

- GIVEN two submit requests contain equivalent canonical attempt, dispatcher, bootstrap, capability, resource, deadline, and policy facts in different map order
- WHEN Mantle canonicalizes them
- THEN both MUST produce the same request ref
- AND clocks, host paths, transport order, raw provider output, and mutable queue position MUST NOT affect that ref.

#### Scenario: Adapter response is malformed or stale

- GIVEN a response has an unsupported schema, wrong operation id, stale attempt/fence, invalid state transition, oversized field, malformed locator, or conflicting duplicate content
- WHEN protocol validation runs
- THEN Mantle MUST reject or quarantine it with a stable reason
- AND it MUST NOT mutate current leases, worker authority, transfer state, or output admission.

### Requirement: Resource projection is provider-neutral and secret-free [r[external_batch_dispatchers.resource_projection]]

Mantle MUST project only already-validated CPU, memory-byte, scratch-byte, accelerator, and bounded named-token requirements into dispatch requests. It MUST preserve semantic-versus-scheduling classification and checked arithmetic. License credentials, token bytes, secret environment values, private endpoints, and provider authentication material MUST NOT enter action identity, public adapter records, logs, or reports.

#### Scenario: Compatible resource request is submitted

- GIVEN an action has an admitted bounded scheduling resource vector and current reservation lease
- WHEN Mantle constructs a dispatch request
- THEN the request MUST carry normalized quantities and opaque named-token labels sufficient for provider translation
- AND scheduling-only availability or reservation ids MUST NOT silently change the action ref.

#### Scenario: Secret license material is supplied

- GIVEN provider operation requires a license credential, bearer token, private endpoint, or secret environment value
- WHEN dispatch configuration and reporting are constructed
- THEN Mantle MUST pass only an approved shell-owned handle through the explicit environment policy or reject the profile
- AND canonical requests, responses, logs, evidence, and CAS objects MUST omit the secret value.

### Requirement: Adapter execution is exact and confined [r[external_batch_dispatchers.adapter_confinement]]

Mantle MUST load typed Nickel dispatcher policy that identifies an exact immutable adapter executable, its BLAKE3 identity, protocol support, allowed operations, provider class, timeout/output limits, environment handles, and bootstrap policy. The shell MUST invoke the executable directly with typed argv and canonical stdin under a clean allowlisted environment; configured shell command strings, PATH lookup, and unbounded stdout/stderr MUST NOT be accepted.

#### Scenario: Exact adapter handles a request

- GIVEN an admitted dispatcher profile and matching executable bytes are available
- WHEN Mantle invokes the adapter
- THEN it MUST verify executable identity, enforce operation/time/output/environment policy, and parse one bounded canonical response
- AND reports MUST bind the adapter/profile identities without exposing raw command or environment data.

#### Scenario: Adapter identity or output drifts

- GIVEN executable bytes differ, PATH resolves another program, the profile contains shell syntax, output exceeds bounds, or the process emits malformed machine data
- WHEN adapter preflight or execution runs
- THEN Mantle MUST fail the dispatch operation with a stable diagnostic
- AND it MUST NOT fall back to ambient tools or interpret untrusted text as scheduler authority.

### Requirement: External jobs remain bound to current fenced attempts [r[external_batch_dispatchers.fenced_lifecycle]]

Every external job submission, observation, cancellation, and reconciliation fact MUST be bound to the current Mantle realization, durable job, attempt, fence generation, adapter operation, and resource lease. External provider job locators and queue states MUST remain scheduling metadata and MUST NOT authorize worker registration, transfer, output admission, or result publication by themselves.

#### Scenario: Current allocation is observed and reconciled

- GIVEN the coordinator restarts with a durable current dispatch request, provider locator, attempt/fence, and lease
- WHEN reconciliation observes a matching active provider job
- THEN Mantle MUST retain the lease and resume bounded observation or worker-registration waiting idempotently
- AND duplicate equivalent observations MUST NOT create another attempt or reservation.

#### Scenario: Stale job reports completion

- GIVEN an earlier attempt’s external job reports running, completion, cancellation, or output after the fence advanced
- WHEN Mantle processes the report
- THEN it MUST reject it as stale or retain it only as bounded diagnostics
- AND it MUST NOT release the current lease, register current authority, import output, or publish an action result.

#### Scenario: Provider state is unknown

- GIVEN observe, cancel, or reconcile times out, contradicts durable state, or returns an unknown provider state
- WHEN lifecycle planning runs
- THEN Mantle MUST preserve safety conservatively and emit a stable blocked/degraded state
- AND it MUST NOT claim physical termination, exactly-once execution, resource release, or successful build without separate current evidence.

### Requirement: Allocations hand off to ordinary Mantle workers [r[external_batch_dispatchers.worker_handoff]]

A scheduled allocation MUST register or connect an ordinary Mantle remote worker under current coordinator authority before it receives action inputs. Declared source/input objects and outputs MUST move through existing receiver-driven CAS transfer, fenced execution reporting, PathInfo, signature, attestation, reference-scan, and output-admission paths. Provider shared filesystem presence MUST NOT substitute for object completeness or trust.

#### Scenario: Worker joins without shared storage

- GIVEN an external scheduler allocates a compatible node with no shared project filesystem
- WHEN the allocation starts the declared worker bootstrap
- THEN the worker MUST register for the exact current attempt/fence and request missing declared objects through Mantle transfer
- AND accepted outputs MUST return through ordinary fenced transfer and admission before any success or action result is published.

#### Scenario: Shared path contains apparent inputs or outputs

- GIVEN a provider-visible directory contains files with expected names or store-like paths
- WHEN the worker or coordinator plans execution or import
- THEN those files MUST remain unavailable unless represented by declared verified object inputs or admitted outputs
- AND path presence, provider ownership, or queue success MUST NOT establish action or output correctness.

### Requirement: Slurm compatibility remains a shell adapter [r[external_batch_dispatchers.slurm_adapter]]

Mantle SHOULD provide an optional `slurm-cli-v1` adapter outside scheduler and protocol cores. It MUST translate canonical operations into bounded typed `sbatch`, `squeue` or `sacct`, and `scancel` argv; any bootstrap script MUST use a fixed reviewed template with escaped data fields. Provider output MUST be parsed as bounded untrusted data, and ordinary tests MUST use exact fake executables rather than a real cluster.

#### Scenario: Slurm fixture completes the lifecycle

- GIVEN fake Slurm executables implement the expected bounded machine-output contract
- WHEN Mantle submits, observes, registers a worker, completes or cancels, and reconciles the fixture job
- THEN the adapter MUST return canonical protocol responses and preserve current attempt/fence bindings
- AND the fixture MUST run without ambient Slurm services, PATH discovery, or shared filesystem authority.

#### Scenario: Slurm command or output is unsafe

- GIVEN a request field contains shell syntax/control bytes, provider output is malformed/oversized, job ids conflict, or cancellation races with completion
- WHEN the Slurm adapter translates or parses the operation
- THEN it MUST reject or classify the condition with bounded diagnostics
- AND it MUST NOT execute interpolated shell text, accept stale authority, or fabricate terminal success.

### Requirement: Dispatcher diagnostics are bounded non-authoritative evidence [r[external_batch_dispatchers.diagnostics]]

Mantle MUST expose bounded redacted plan, status, build, and immutable-attempt diagnostic facts for dispatcher profile/class, adapter identity, provider job locator digest or safe label, normalized queue state, current attempt/fence, resource summary, worker registration, cancellation/reconciliation, transfer, and output-admission state. Raw provider commands/output, credentials, private paths, license details, environment values, and unbounded identifiers MUST be omitted, escaped, or reduced to safe digests.

#### Scenario: Operator inspects queued hardware work

- GIVEN a current attempt is queued or running through an external dispatcher
- WHEN an operator requests human or JSON status
- THEN Mantle MUST distinguish dispatch, provider allocation, worker registration, input transfer, execution, output transfer, and admission phases
- AND it MUST state that queue/allocation evidence does not prove execution success, output trust, or release eligibility.

#### Scenario: Provider emits control-looking diagnostics

- GIVEN adapter stderr or provider status includes protocol frames, terminal escapes, scheduler directives, secrets, or output-admission labels
- WHEN Mantle records or renders diagnostics
- THEN it MUST treat the bytes as bounded untrusted text subject to escaping and redaction
- AND they MUST NOT mutate lifecycle, priority, lease, fence, transfer, or output-trust state.

### Requirement: Hardware workload composition preserves generic semantics [r[external_batch_dispatchers.hardware_workload_composition]]

The external-dispatch validation rail SHOULD execute the bounded hardware-simulation generation, compile, link, and smoke workload when that reference capability is available. Dispatch MUST preserve the workload’s exact action/tool/source identities, resource/token and locality decisions, shared-result behavior, CAS-only transfer, result artifacts, and non-claims without introducing HDL-specific provider logic.

#### Scenario: Hardware actions run through the fixture dispatcher

- GIVEN the admitted hardware reference graph, compatible resource inventory, and fixture batch adapter
- WHEN Mantle dispatches its ready actions remotely
- THEN each allocation MUST use the same generic worker, transfer, admission, and action-result seams as other builds
- AND evidence MUST identify executed and reused stages without claiming production cluster throughput or commercial-license behavior.

#### Scenario: Hardware action is not eligible

- GIVEN a hardware action lacks required resources, named token capacity, tool capability, upload permission, output trust, or source readiness
- WHEN dispatch planning runs
- THEN it MUST remain ineligible before provider submission
- AND queue availability, locality, age, or an adapter response MUST NOT bypass the blocker.

### Requirement: External dispatcher validation covers failures and recovery [r[external_batch_dispatchers.final_validation]]

The external batch-dispatcher change MUST include positive and negative pure-protocol, resource, adapter confinement, coordinator restart, fence, worker handoff, transfer, Slurm translation, diagnostics, and workload-composition evidence. Fixture success MUST NOT be reported as production provider support, and unavailable real-provider evidence MUST remain an explicit non-claim.

#### Scenario: Dispatcher change is ready for lifecycle review

- GIVEN protocol, profile, process, coordinator, worker, provider adapter, or report behavior changes
- WHEN validation evidence is assembled
- THEN it MUST include focused positive and negative checks, exact commands and outputs, Cairn validation, proposal/design/tasks gates, and relevant first-party quality rails
- AND it MUST distinguish provider-free/fake-Slurm composition from any separately proven real-cluster compatibility.
