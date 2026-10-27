# Design: Nix-compatible remote service gateway

## Context

Nix clients expect a daemon-store protocol over transports such as `ssh-ng`. Hosted build integrations also need an asynchronous request and observation API. Mantle should provide these edges while retaining its own durable execution semantics.

The gateway is an adapter. It is not a new scheduler, store of truth, cache authority, or evaluator.

## Architecture

```text
Nix client or CI orchestrator
  -> transport and protocol shell
  -> typed authorization context
  -> pure request translation and policy checks
  -> existing Mantle attempt, store, CAS, and scheduler services
  -> bounded observations and signed completion event
```

### Protocol shell

The shell terminates SSH or another configured transport, negotiates a supported Nix daemon protocol version, enforces frame and message limits, and parses operations with the vendored Nix compatibility layer.

Only an explicit operation table is enabled. Concrete store-object transfer, PathInfo queries, substitution checks, and build requests can be mapped to Mantle services. Unsupported operations fail with stable protocol errors. The gateway never evaluates flakes, resolves registries, fetches arbitrary sources on behalf of a caller, or exposes a command shell.

### Functional translation core

The core accepts a parsed operation, authenticated authority facts, service policy, and explicit time or sequence inputs. It returns a typed Mantle command or a rejection. It performs no transport, filesystem, clock, network, store, or scheduler I/O.

The core preserves the concrete-request boundary. A build command identifies admitted derivations and store inputs. It cannot carry evaluator expressions or arbitrary executable commands.

### Existing Mantle services

The imperative adapter calls existing attempt creation, fencing, transfer, locality, lease, scheduler, CAS, PathInfo, cache, and signed-result interfaces. It does not mutate their invariants or combine their authority domains.

A client disconnect does not erase a durable attempt. Reconnection uses an opaque public attempt identifier plus valid read authority. Duplicate submissions use an explicit idempotency key and return the same admitted attempt only when caller and request identities match.

## Authorization

The gateway converts either a hardened compatibility ticket or a verified UCAN into one internal `RemoteAuthority` value. The value carries subject identity, project or account scope, allowed operations, audience, expiry, and evidence references.

Policy checks distinguish at least:

- submit build;
- upload admitted input;
- read store object;
- read status and logs;
- cancel owned attempt;
- publish cache result;
- administer service.

No broad bearer token implies all operations. Compatibility tickets are an explicit migration surface and cannot obtain administration authority. UCAN verification occurs before request translation and uses the existing UCAN verification boundary.

Gateway verification uses UCAN's strict zero-clock-skew limits at both initial
admission and non-consuming revalidation. Lifetimes are half-open: the exact
effective expiration second is already denied. Initial lifetime rejection occurs
before the holder nonce is spent; a live session does not spend it again during
revalidation. Operator-snapshot currentness, generation, revocation, exact
operation, target, request digest and selected caveat checks remain required.

## Build API

The versioned API provides bounded operations for:

- submit a concrete request with an idempotency key;
- inspect one authorized attempt;
- read a bounded log range by cursor;
- read a bounded event page by cursor;
- request cancellation;
- discover signed results;
- read bounded project or account usage summaries.

Every list or byte response has named configurable bounds. Cursors are opaque, versioned, and authenticated against query scope. The API returns typed reason codes, not raw internal errors.

## Completion events

Mantle appends an event when an admitted attempt changes observable state. Completion events include public attempt and request identities, terminal class, result-evidence identity when present, policy version, sequence, and producer signature.

Delivery is at least once. Consumers deduplicate by event identity and sequence. Events contain references to bounded logs and evidence, not log bodies or credentials.

## Store and cache boundaries

Nix store compatibility does not make the transport authoritative for bytes or PathInfo. Mantle verifies imported content, checks PathInfo and CAS availability, and applies store-write authority before publication.

Result discovery remains separate from execution authority. A signed action result is usable only after current policy, platform identity, output identity, and CAS availability checks pass.

## Failure and abuse controls

The gateway limits connections, negotiations, frames, request sizes, concurrent operations, log windows, event pages, and idle time with named policy values. It rejects malformed lengths before allocation.

Disconnect, timeout, cancellation, duplicate messages, stale fences, invalid cursors, unknown protocol versions, unauthorized operations, and partial uploads have explicit outcomes. Durable attempt ownership and transfer resumption remain intact.

## Evidence and non-claims

The gateway records bounded protocol version, operation class, authority decision identity, attempt identity, state transition, byte counts, and error category. It never records bearer material or raw OIDC tokens.

Compatibility evidence proves only that selected operations interoperate under the tested client and policy. It does not prove arbitrary Nix compatibility, hermetic builds, sandboxing, output correctness, or release eligibility.

## Rollout

The service starts on a private test endpoint. Interoperability fixtures then cover supported Nix clients and explicit negative cases. Public exposure requires archived credential-hardening evidence and OnixOS deployment policy. The internal Mantle protocol remains available throughout rollout and rollback.

## Configured production host

On Linux, `src/nix_gateway/api_host.rs` implements `Services` over the existing
durable coordinator and `prepare_remote_dispatch` / `execute_remote_dispatch`.
The API journal records delivery and opaque public IDs; it is not another
scheduler. Resource reservations are idempotent against the exact coordinator
job, attempt and fence before dispatch. A host-lifetime exclusive lock prevents
a second gateway from misclassifying a live host as interrupted during recovery.
Duplicate submission reaches the idempotent host under fresh current authority.
A queued intent can recover its existing fenced identity; an uncertain execution
is not automatically rerun. Connection loss does not stop an admitted worker.
A new public request cannot attach to an existing scheduler job merely because
its normalized concrete action matches. Such attachment is not sharing authority
and would mix request/accounting identities. Only the separate verified completed
result-reuse gate can create that consumer link; otherwise admission fails closed.
Successful signed output import retains immutable producer evidence immediately,
without requiring an unrelated result-read call to make later authorized reuse
possible. Every actual result read and reuse still checks current signed store
metadata and CAS availability.


The explicit commands are:

```bash
mantle remote gateway provision-quota --directory "$quota_dir"
mantle remote gateway binding --request "$request_json" --project "$project"
mantle --state-dir "$coordinator_state" remote gateway serve \
  --config "$gateway_config" --socket "$private_socket" --ticket-fd 3
mantle remote gateway call --socket "$private_socket" --envelope "$invocation_json"
mantle --state-dir "$coordinator_state" remote gateway nix-once \
  --config "$gateway_config" --socket "$nix_socket" --ticket-fd 3 \
  --envelope "$concrete_build_invocation_json"
```

FD 3 must already contain an explicitly issued remote-runner ticket; the commands
do not mint tickets, UCAN grants, holder signatures or keys. Quota provisioning is
a separate, explicit empty-ledger operation in an existing private directory.
The JSON configuration is the strict `GatewayConfig` schema in `api_host.rs`:
version, endpoint, audience, public policy digest, authority configuration,
journal/quota/evidence directories, cursor-secret and completion-key files,
resource policy, per-system reviewed features, builder program and argv, trusted
builder keys, observability configuration, build deadline, and explicit
SecretSpec manifest/profile/provider. All classes must target that configured
endpoint; the existing runner inventory and leases still determine admission.
The API adapter does not supply a sandbox or invent platform feature facts.

`required_resources` is an explicit map from each exact reviewed system to its
canonical `RemoteResourceRequirements` (quantities plus semantic accelerator
classes). Its key set must equal `features`, be nonempty and remain within the
existing class bound. Missing systems, zero/oversized quantities, mismatched
semantic accelerators and noncanonical vectors are rejected; no catalog capacity
or inferred default supplies a request declaration. The gateway attaches those
operator-declared minima before selection and ordinary lease admission. Retries
may increase selected quantities but cannot lower this declaration. The public
submit target remains the raw `.drv` identity; the host internally projects its
all-output `.drv!*` native target after ordinary signed-derivation verification.

The envelope contains API version, the operation-specific authority request,
compact UCAN, holder nonce/signature, and a typed request. `binding` prints only
the public operation/target/request-digest tuple for the holder to sign. A new
holder invocation is required for a retry of a consumed envelope; its submit
idempotency key and concrete derivation remain unchanged. Status, logs, events,
result, usage and cancellation each require their own matching current operation
grant. Cursor bounds and cancellation fence are included in that signed tuple.
The owner-private Unix transport uses a four-byte big-endian length prefix,
128-KiB credential-envelope and 1-MiB response limits, a 10-second read/write
deadline, and same-owner peer credentials. There is no TCP/public listener.

`nix-once` accepts one explicitly authorized concrete derivation on one Nix
connection. Its BuildPaths bridge calls the actual asynchronous host and waits
for durable completion, then requires current signed derivation/output/CAS
verification. It does not turn a cached callback into build success. The `read`
subcommand retains the existing verified read-only Nix socket. Evaluation,
source fetching, arbitrary commands and administrative Nix operations remain
unsupported.

Public logs contain bounded host lifecycle/reconciliation events, not executor
stdout/stderr or credentials. Worker byte counts are measurements, not log
content. Result reads reopen current verified store authority and check signed
outputs/CAS; reuse is a consumer link to a verified producer, not another
execution or quota charge. Signed PathInfo evidence references fingerprint the
actual retained signed metadata, not an attestation digest relabeled a signature.

### Cancellation and recovery boundary

For the same-binary local worker, cancellation is a durable fenced intent;
the transport terminates/reaps the local process tree before recording its
teardown acknowledgement. A cancelled attempt without complete authenticated
measurements retains its quota reservation in pending reconciliation. No zero
charge or complete metrics are inferred from cancellation.

The existing remote/SSH worker protocol has no independently authorized remote
cancel-and-teardown acknowledgement. Remote cancellation therefore remains a
durable intent and its reservation remains pending until authenticated complete
worker facts are received. An SSH disconnect is never a teardown acknowledgement.
Immediate remote cancellation requires that additional operation-specific worker
protocol; it is not claimed by this host. Likewise, reopening after an uncertain
execution cannot safely redispatch or release its reservation merely because the
client disappeared. Real configured-fixture, farm interoperability and deployment
proofs remain separate observed evidence, not consequences of these code paths.

### Focused verification handoff

The captured baseline at
`.cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/v1-gates-2026-10-22/head-crunch-bin-16-isolated.log`
records an expired session accepted at explicit time 1901 (signed deadline 1900)
and three resource-host fixtures denied `StoreUnavailable`. The session repair
uses strict bounds, including exact expiry; resource fixtures now create the
required private quota directory rather than relaxing production access checks.
These changes have not been executed in this handoff. No runtime/gate acceptance
is inferred from the source repair.

After integration, the focused commands are:

```bash
cargo test -p mantle --bin mantle nix_gateway::
cargo test -p crunch-nix-gateway-core
cargo test -p crunch-nix-gateway
cargo test -p crunch-nix-gateway --test nix_path_info installed_nix_ -- --ignored
```

The ignored interoperability cases require the installed Nix 2.36 client named
by their fixture. For an actual configured host, retain producer/client output
and endpoint configuration identities. In one terminal, run the documented
`serve` command with the private configuration and existing ticket FD; in another,
use freshly signed envelopes for each operation:

```bash
mantle remote gateway call --socket "$private_socket" --envelope "$submit_envelope"
mantle remote gateway call --socket "$private_socket" --envelope "$status_envelope"
mantle remote gateway call --socket "$private_socket" --envelope "$events_envelope"
mantle remote gateway call --socket "$private_socket" --envelope "$usage_envelope"
mantle remote gateway call --socket "$private_socket" --envelope "$cancel_envelope"
```

For the separate Nix `nix-once` process documented above, submit the exact admitted
derivation from its signed build envelope with an installed compatible client:

```bash
nix-store --realise --store "unix://$nix_socket" "$drv_path"
```

The read-only service is a separate process (`mantle --state-dir
"$coordinator_state" remote gateway read`); its actual Nix read command is:

```bash
nix path-info --store "unix://$coordinator_state/nix-gateway/nix-worker.sock" "$signed_store_path"
```

These commands do not provision credentials or derive holder signatures. Real
ticket/UCAN issuance, signed derivation/input/output admission, trusted full keys,
registered worker inventory, quotas, and private owner-controlled paths must
already exist. Public deployment, OnixOS farm integration and external Valence
consumer evidence remain independent unexecuted requirements.


