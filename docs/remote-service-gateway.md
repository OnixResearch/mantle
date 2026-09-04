# Remote service gateway

The remote service gateway connects Nix clients and CI clients to existing Mantle services. It does not create a second scheduler or store.

## Supported interfaces

The gateway has two interfaces:

- The Nix interface serves one daemon-store session on standard input and standard output.
- The Build API serves one versioned JSON operation on standard input and standard output.

An SSH service can run either command as a forced command. Keep the endpoint private during the first rollout.

### Nix operation table

The Nix interface supports these store operations:

- `is-valid-path`
- `query-path-info`
- `query-path-from-hash-part`
- `query-valid-paths`
- `query-valid-derivers`
- `add-to-store-nar`

The interface rejects all other daemon operations. It disables generic `nix-compat` stubs that would otherwise return synthetic empty results. The interface does not evaluate flakes, resolve registries, fetch sources, or run arbitrary commands.

The supported daemon protocol has major version 1. The supported minor versions are 23 through 37. Older `STDERR_READ` transfer framing is outside this gateway because it cannot enforce the same frame-synchronization guarantee.

### Build API operation table

The `mantle-remote-build-api-v1` interface supports these operations:

- `submit-build`
- `read-status`
- `read-log-range`
- `read-event-page`
- `cancel-attempt`
- `discover-signed-result`
- `usage-summary`

The pure planner also models store reads, input upload, cache publication, and service administration. Their effect adapters remain separate authority domains.

## Authority

A verifier must write the authority facts file before it starts the gateway command. The remote client cannot select this file. The gateway requires a regular file and does not follow a symlink.

The authority facts contain a subject, an account scope, an audience, an expiry, capabilities, and evidence references. They contain no bearer credential.

The API request repeats the authority facts. The gateway rejects the request if these facts differ from the verifier-produced file.

The gateway accepts these authority sources:

- a hardened compatibility-ticket verifier
- a Basalt UCAN verifier

A compatibility ticket never grants service-administration authority. A UCAN grants only the capabilities in its verified authority facts.

Capabilities are separate for these operations:

- build submission
- input upload
- store read
- status read
- log read
- owned-attempt cancellation
- cache publication
- service administration

## Private policy

`lib/remote-builders.ncl` exports `GatewayPolicy`. The top-level `gateway` field uses private defaults.

Run this command to inspect the effective policy:

```sh
mantle remote gateway status gateway-policy.json
```

The status output contains policy identity, visibility, bounds, and non-claims. It contains no credentials.

A public policy must contain this archived credential-hardening tasks-gate receipt:

```text
644a7002dc721e081f3f09e26cc96e313f0e23ae40074ad159b69f56ea654060
```

Any other value blocks public exposure. The receipt comes from `.cairn/archive/2026-08-01-harden-remote-credential-boundary/`.

## Plan an API request

Use the planner before you run an effect adapter:

```sh
mantle remote gateway plan request.json
```

The planner performs no file, store, scheduler, clock, or network operation. It returns a typed command or a stable rejection.

## Run one Build API operation

Prepare a file descriptor that contains exactly 32 random bytes. Do not use a terminal or a standard descriptor.

Then run one operation:

```sh
mantle \
  --state-dir /srv/mantle/state \
  remote gateway api-dispatch-stdio-once \
  --authority /run/credentials/mantle-gateway-authority.json \
  --cursor-key-fd 3 \
  --secret-manifest /etc/mantle/secretspec.toml \
  --secret-profile production \
  --secret-provider systemd
```

Send one `RemoteGatewayDispatchRequest` JSON value on standard input. The request contains the versioned API request and an optional coordinator request.

A submit operation requires a concrete derivation coordinator request. The request identity, derivation path, inputs, and expected outputs must match the API operation.

The shell replaces the client time with the service clock. The shell uses this time for expiry, deadline, and cursor checks.

## Run one Nix session

Run one Nix daemon session with an operator-owned authority file and trusted store keys:

```sh
mantle \
  --nix-compat \
  --state-dir /srv/mantle/state \
  remote gateway nix-stdio-once \
  --authority /run/credentials/mantle-gateway-authority.json \
  --policy gateway-policy.json \
  --trusted-store-key 'cache.example-1:BASE64'
```

Bind this command to an SSH forced command for `ssh-ng`. The gateway does not provide a shell.

## Bounds

The default policy has these maximum values:

| Resource | Maximum |
|---|---:|
| Connections | 1,024 |
| Frame bytes | 1,048,576 |
| Message bytes | 4,194,304 |
| Concurrent operations | 1,024 |
| Idle period | 3,600 seconds |
| Cursor bytes | 512 |
| Log window | 1,048,576 bytes |
| Event page | 1,024 items |
| Partial transfer | 1,073,741,824 bytes |
| Paths per request | 4,096 |
| Usage summary | 4,096 items |

The one-shot Nix adapter admits one connection and one operation stream. It also applies a 67,108,864-byte connection budget.

The Nix parser applies the selected policy before allocation. It limits each byte value and NAR frame, each collection item count, and aggregate request metadata. A NAR body uses the separate partial-transfer limit.

A policy can select a lower limit than the default maximum. Both JSON API planning and the Nix parser enforce that selected limit. The immutable log service applies its existing record, replay, retention, and redaction limits. The API cannot increase these limits.

## Reconnect and idempotency

A build submission derives a public attempt identity from the subject, account, idempotency key, and request identity.

An exact duplicate returns the existing attempt. A changed subject, scope, key, or request identity returns an idempotency conflict. This check occurs before coordinator mutation.

A disconnect does not cancel a durable coordinator attempt. An authorized client can use the public attempt identity to read later state.

Opaque cursors bind the subject, account, public attempt, stream, sequence, and expiry. A keyed BLAKE3 MAC authenticates each cursor.

## Cancellation

Cancellation requires the `cancel-owned-attempt` capability. The subject and account scope must match the stored attempt.

If the attempt is active, the gateway uses the current fenced coordinator binding. If the attempt is terminal, cancellation does not change it.

Both outcomes return a signed terminal event. Repeated reads return the same event identity and signature. On load, the adapter recomputes persisted event identities and validates sequence, attempt, policy, request, and signature-name linkage before reuse.

## Completion events

A completion event contains these public facts:

- the public attempt identity
- the request identity
- the terminal class
- an optional result-evidence identity
- the policy identity
- a sequence
- the producer key name and signature

The event contains no log body or credential. The gateway signs this domain and event identity:

```text
mantle-remote-completion-signature-v1 || event_identity_blake3
```

Consumers can deduplicate repeated delivery by event identity and sequence.

## Valence mapping

Each gateway evidence record contains a typed `valence_observation` projection. It names profile `valence.build-service-evidence.v1`, role `RecordedOnly`, the source evidence identity, policy identity, authority evidence references, operation class, optional attempt, byte count, outcome, and Valence's exact required non-claim.

The mapping is pinned to the accepted profile at Valence revision `e40c76b4d2070a29636e00c85c0dff93f03dba2f`. It is an input for cross-project bundle assembly. It is not a complete Valence bundle and does not claim Valence validation. Valence remains responsible for canonical bundle identities, linkage validation, and field-level roles.

## Store integrity

Nix transport metadata does not make store bytes authoritative. The gateway verifies the complete NAR hash, size, PathInfo signatures, and store path.

The gateway uses the existing Mantle output persistence path after verification. Transport success cannot override a store rejection.

## Rollback

Stop the forced-command endpoint to remove gateway access. This action does not delete coordinator, store, log, or gateway state.

Restore the prior service configuration if the endpoint fails. Existing Mantle clients continue to use the internal remote protocol.

## Non-claims

Gateway evidence applies only to the selected request, policy, protocol, and observed state. It does not prove these properties:

- arbitrary Nix compatibility
- evaluator correctness
- hermeticity or sandbox correctness
- output or PathInfo correctness
- release eligibility
- authority outside the listed capabilities
