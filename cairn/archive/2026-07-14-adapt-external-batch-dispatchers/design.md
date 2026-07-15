## Context

Mantle’s coordinator already owns durable jobs and attempts, fence generations, remote worker registration, transfer, logs, and output admission. Its scheduler can choose an eligible remote route from explicit capability and trust facts. The active scarce-resource change adds provider-neutral resource requirements and fenced reservations, including named token pools suitable for representing scarce licensed-tool capacity without carrying credentials.

Existing HPC installations often require work to enter through a batch scheduler. The build engine should therefore request an allocation and arrange for a Mantle worker to join the current attempt, but it must not treat queue state, provider job ids, shared filesystems, or adapter output as build truth.

## Decisions

### 1. Define a narrow external adapter protocol

**Choice:** Add `mantle-batch-dispatch-adapter-v1` with bounded canonical request/response records for `submit`, `observe`, `cancel`, and `reconcile`. Requests bind operation id, dispatcher profile ref, realization/job/attempt/fence refs, worker-bootstrap ref, semantic capability class, admitted scheduling resource vector, deadline/retention classes, and redacted diagnostic context. Responses bind operation id, provider job locator, normalized state, bounded reason fields, and adapter evidence ref.

**Rationale:** A process protocol lets Slurm, LSF, or site-specific dispatchers remain replaceable shells while Mantle preserves one semantic core.

### 2. Identify and confine the adapter executable

**Choice:** Typed Nickel configuration names an absolute immutable adapter executable ref, BLAKE3 identity, supported protocol/schema, allowed operations, provider class, timeout/output limits, and environment allowlist. Mantle launches it directly with typed argv and canonical JSON input, clears ambient environment except explicit non-secret handles, captures bounded stdout/stderr, and never evaluates a configured shell command string.

**Rationale:** An adapter is privileged scheduling code. Ambient executable discovery and shell interpolation would create hidden authority and injection paths.

### 3. Keep resource allocation separate from action identity and output trust

**Choice:** Only resource requirements already validated by Mantle’s provider-neutral core enter the request. CPU, memory, scratch, accelerator, and named token quantities are scheduling facts unless separately declared semantic action inputs. Token labels may map to provider constraints, but license tokens, credentials, server addresses, and secret environment values remain worker/provider-side handles and are never serialized into public records.

**Rationale:** Scarce resource authorization determines where work may run; it does not prove what the action means or whether its outputs are trustworthy.

### 4. Bind external jobs to current fenced attempts

**Choice:** A provider job locator is accepted only as metadata attached to the exact current job/attempt/fence and adapter operation. The scheduled allocation must register a Mantle worker using current coordinator authority before receiving action material. Late registration, stale observations, stale cancellation, or results from an earlier fence are rejected or retained only as diagnostic facts.

**Rationale:** External schedulers may retry, delay, duplicate, or report out of order. Mantle fencing remains the authority that prevents stale results from winning.

### 5. Use Mantle transfer instead of shared filesystem authority

**Choice:** The allocation receives a minimal worker-bootstrap descriptor and then obtains declared source/input objects through the existing receiver-driven CAS transfer path. Outputs return through the same transfer, PathInfo, signature, attestation, and action-result admission path. A provider shared directory may hold adapter-local bootstrap material only when policy allows, but its paths and bytes do not become action inputs or accepted outputs without ordinary object admission.

**Rationale:** This preserves portability and correctness across clusters without NFS and prevents ambient cluster state from silently entering the build.

### 6. Reconcile lifecycle conservatively

**Choice:** Coordinator state stores canonical dispatch request/ref, adapter operation ids, provider job locator, normalized observed state, current attempt/fence, resource lease ref, worker registration status, cancellation intent/outcome, and bounded diagnostics. Restart reconciliation re-observes active jobs through idempotent operations. Unknown, unreachable, contradictory, or timed-out provider states cannot release current leases or admit outputs optimistically.

**Rationale:** Queue submission success and physical execution are not exactly-once, and process/network failures can occur between every persistence step.

### 7. Ship one optional Slurm shell adapter

**Choice:** Implement `slurm-cli-v1` outside scheduling/core modules. It translates canonical requests into typed `sbatch`, `squeue`/`sacct`, and `scancel` argv, generates any batch script from a fixed template plus escaped data fields, parses bounded machine-oriented output, and returns canonical responses. Tests use exact fake executables and no real Slurm service. LSF is an external protocol implementation, not part of this change.

**Rationale:** One concrete adapter proves the protocol is implementable while preserving a provider-neutral core and deterministic ordinary tests.

### 8. Preserve diagnostic and claim boundaries

**Choice:** Human/JSON reports identify dispatcher profile/class, adapter executable digest, provider job locator digest or redacted label, normalized queue state, current attempt/fence, resource summary, worker registration, cancellation/reconcile outcomes, transfer mode, output-admission state, and non-claims. Raw provider output, commands, environment, credentials, license details, and unbounded ids are omitted or escaped.

**Rationale:** Operators need queue visibility, but provider diagnostics are untrusted data and scheduler success is not build success.

## Functional Core / Imperative Shell

- **Core**: protocol validation/canonicalization, operation and state transition planning, attempt/fence binding, resource projection, response classification, idempotency/conflict handling, reconcile/cancel plans, report facts, bounds, and stable reason codes.
- **Shell**: Nickel export loading, executable identity checks, process launch, stdin/stdout/stderr I/O, timeouts, provider CLI calls, coordinator persistence, clocks, worker bootstrap/registration, network transport, cancellation effects, and rendering.

## Risks / Trade-offs

- Provider queue and cancellation semantics differ; the protocol intentionally normalizes only the state needed by Mantle and retains unknown provider detail as bounded diagnostics.
- A provider may report cancellation while the process still runs. Fencing prevents stale output admission, but exactly-once physical termination is not claimed.
- License resources can be represented as named capacities, but real server freshness and checkout behavior remain external operational facts.
- Starting a worker through a batch allocation adds queue latency; shared action-result reuse should be attempted before dispatch.
- Fake Slurm fixtures prove translation and lifecycle logic, not compatibility with every Slurm release or site policy.

## Non-Goals

- No general workflow engine, CI pipeline language, cluster autoscaler, scheduler replacement, or interactive remote shell.
- No arbitrary provider command templates in trusted configuration.
- No transfer of secret license material through CAS, action specs, adapter records, logs, or public evidence.
- No assumption that a provider-visible filesystem is complete, immutable, or suitable for action identity.
- No REv2 semantic-core migration and no claim that adapter compatibility proves remote output correctness or release eligibility.
