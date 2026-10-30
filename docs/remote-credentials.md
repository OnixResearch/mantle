# Remote credential operations

Mantle keeps remote bearer values out of durable state. The remote service needs two external secrets:

- `TICKET_VERIFIER_KEY`: `<key-id>:<canonical-base64url-32-bytes>`
- `RESULT_SIGNING_KEY`: a Nix-format Ed25519 signing keypair

The checked [`secretspec.toml`](../secretspec.toml) declares metadata only. It has no defaults, generators, secret files, or cache configuration.

Ticket issuance uses 32 bytes of operating-system randomness. The reviewed policy limits ticket life to 2,592,000 seconds and use count to 100,000.

## Source identity

Mantle pins SecretSpec `0.17.0` in Cargo and Nix metadata.

- Reviewed source commit: `a8794e46ec9664a0e1a3869cc3105d0853937e48`
- Crates.io checksum: `68498f9695bb3662c157b8fd4b4665a594f1157de022ff5b0f891af4c7ec75d2`
- Scope: `mantle-remote`

## Provider policy

Use these fixed combinations:

| Operation | Profile | Allowed provider |
|---|---|---|
| Production service and ticket issue | `production` | `systemd-credential://` |
| Initial bootstrap | `bootstrap` | An explicit `sops://...` URI |
| Key rotation | `rotation` | `systemd-credential://` or an explicit `sops://...` URI |

Mantle rejects other profiles and providers. It also ignores the ambient SecretSpec scope.

The provider runs in a worker process. Mantle applies a deadline, process-group cleanup, retained-output limits, a CPU limit, and a memory limit.

Provider failure stops the operation. Mantle does not try another provider, generate a key, or write ticket state.

## Credential admission

Wire and legacy-state records remain structural until the pure credential core admits them.

The core uses distinct types for ticket identity, issued and presented bearers, verifier identity, validity, use state, and resource limits.

Secret types have no ordinary display or Serde path. Build-time and upload limits cannot be exchanged without an explicit checked conversion.

Malformed records fail before verifier comparison or policy evaluation. Compatibility parsing still accepts bounded public identifiers such as `ticket-1`.

## Create a ticket

Open a caller-owned descriptor before the command. Do not use standard output or standard error for the bearer.

```sh
exec 9>ticket.secret
mantle --json remote ticket create \
  --ticket-fd 9 \
  --display-name builder-a \
  --secret-manifest ./secretspec.toml \
  --secret-profile production \
  --secret-provider systemd-credential://
exec 9>&-
chmod 600 ticket.secret
```

The JSON report contains the ticket identifier and public policy. It does not contain the bearer or verifier bytes.

Mantle does not keep a recoverable bearer. If `ticket.secret` is lost, revoke the ticket and create a replacement.

For attended issuance, replace `--ticket-fd 9` with `--interactive-operator-terminal-reveal`. This explicit action writes the new bearer to the controlling terminal once.

## Migrate plaintext-era state

First, inspect the invalidation plan:

```sh
mantle --json remote ticket migrate-legacy \
  --invalidate-legacy-tickets \
  --dry-run
```

Then, execute the migration:

```sh
mantle --json remote ticket migrate-legacy \
  --invalidate-legacy-tickets
```

The command invalidates all old ticket identifiers. It does not preserve old bearer validity.

The replacement state uses schema version 2. The state directory uses mode `0700`, and `tickets.json` uses mode `0600` on Unix.

## Rotate keys

Provision the new keys in the `rotation` profile. Then run:

```sh
mantle --json remote ticket rotate-keys \
  --secret-manifest ./secretspec.toml \
  --secret-profile rotation \
  --secret-provider systemd-credential://
```

Mantle invalidates tickets that name an older verifier key identifier. Create replacement tickets after rotation.

## Backup and incident response

Treat every schema-version-1 backup as a bearer disclosure. Do not restore it into a running service. Migrate it offline, then issue replacement tickets.

A schema-version-2 backup contains verifiers and public policy. Protect it because it exposes ticket identifiers, limits, and activity state.

If a bearer might be exposed, revoke that ticket. If a verifier or signing key might be exposed, rotate the keys and issue replacements.

Rollback can restore an older binary only during offline recovery. It must not restore a plaintext state file or reactivate invalidated tickets.

## Start a remote service

Pass the same explicit boundary to the service:

```sh
mantle remote serve \
  --binding stdio-once \
  --executor local-build \
  --secret-manifest ./secretspec.toml \
  --secret-profile production \
  --secret-provider systemd-credential://
```

`mantle build --builder ...` accepts matching `--remote-secret-*` options for its generated local service process. It reads the bearer from `--ticket-fd`; it does not accept a bearer in argv.

The current stdio and SSH-stdio bindings do not authenticate a peer identity. They reject endpoint-bound tickets. A future authenticated transport must supply that identity from its handshake.

Wall-clock freshness is host-owned. Mantle rejects authorization when the service clock is before ticket issuance, but operators must monitor clock synchronization and rollback.

The service reads bounded admission frames before it takes the state lock. Under the lock, it reloads state and obtains the service time. It then checks the peer and request, decrements the ticket, and atomically saves and syncs the state. It releases the lock before `AuthOk`, uploads, execution, or output. It does not save the admission copy again.

Protocol input and completed authentication owners wipe their buffers. Frame encoding and verifier-state encoding use zeroizing output owners before serialization starts. A failed partial `serde_json` decode can allocate a secret string before constructing its zeroizing owner. Mantle does not claim complete allocator-memory erasure for that failure path.

## Holder-attenuated grants (optional, advisory, not accepted)

Status: implemented, not accepted. Cairn change `attenuate-build-authority-with-pattern-caveats` keeps its grant-site tasks open.

**Attenuation is advisory, not enforcing.** A ticket request may carry an `attenuated_grant`: a holder-signed UCAN presentation whose preserved Mantle caveats (`mantle`/`filter-v1`, ordinals `00`..`07`) restrict one job. Requests without a presentation use the bearer path above, unchanged. The bearer is the only mandatory credential, so a holder who also has the ticket bearer can omit the presentation and be admitted unrestricted. There is no enforcing mode and no per-ticket "attenuation required" property; adding one needs a ticket-schema change and an owner decision. The restriction binds only requests that carry it.

What the receiver does when a presentation is present. It verifies it under the ticket-state lock, after the bearer and request checks and before the ticket is redeemed, using its own clock. The filter-visible job identity is `/job/<request_id>[/attempt/<production job_id>]/class/<derivation|action>/outputs/<declared output names, sorted>`. Request ids, job ids and output names that the filter vocabulary cannot express (store-name-safe ASCII only), more than nine output names, duplicates, or values over the filter bounds are refused, never truncated. The holder signs a BLAKE3 digest over a domain, the endpoint id and the deterministic `serde_json` encoding of the entire `ConcreteBuildRequest`, so changing any request field after signing is refused (declared order of output names is part of the digest). The ticket id is not bound: the same presentation can be used with any ticket its sender holds. The caveat deadline is the grant's signed expiry, checked against the service clock beside the ticket's own expiry.

A refused presentation writes no protocol frame, redeems no ticket use, starts no work and shows the sender only `remote-attenuated-grant-denied`: never the job, path, caveat or reason. The receiver logs a reason code locally at `warn` (target `mantle::remote::attenuation`, never sent): `no-operator-authority`, `presentation-malformed`, `project-scope-mismatch`, `request-unbindable`, `verification-denied` (the UCAN/Basalt verifier does not distinguish signature, proof chain, preserved parent caveats, validity window, revocation, replay, a full replay ledger, filter denial, or wrong endpoint, audience, generation or holder), `session-binding-mismatch`, `chain-report-denied`. The redacted effective caveat chain is logged at `info` only after the redemption committed. A presentation that verified but whose redemption then failed has still consumed its holder nonce.

A presented grant is refused with the same constant string on the fixture executor, loopback and frame-level admission paths, which cannot enforce it; it is never silently ignored. `--executor` still defaults to the fixture executor; an older receiver that predates this field drops it without enforcing, and the hello does not advertise an attenuation capability.

Operator-owned authority. Place the snapshot at `<state-dir>/attenuation-authority.json` (owner-private mode `0600`, regular file, not a symlink, at most 1 MiB, same schema as the Nix gateway authority file) and the holder replay ledger directory at `<state-dir>/attenuation-replay` (mode `0700`). The snapshot `endpoint` must equal the builder endpoint id and the grant `project` must equal the endpoint binding scope. When the snapshot is absent, a presented grant is refused. When it is present, `remote serve` loads it at start (JSON, version, validity window, policy hash, trusted roots, keys, proofs) and refuses to start on any failure. Revocation freshness equals the snapshot contents and window; the rollback guard is per process and `remote serve` is stdio-once, so restoring an older snapshot is accepted.

Parent proofs and holder keys must be provisioned. The presentation carries no proofs or keys: every parent proof, delegate key and holder key of every grant must already be in the snapshot (at most 256 keys, 256 proofs, 8,192 revoked entries). A holder cannot sub-delegate unilaterally; the operator republishes the snapshot, and the snapshot becomes a registry of grants.

Replay ledger limits. The ledger is shared by all holders, capped at 8,192 records, and has no expiry or garbage collection; every admitted presentation takes one slot permanently, before redemption. When it is full, every attenuated admission is denied (locally logged as `verification-denied`) while legacy bearer admission continues. Rotation procedure: stop the service, move the ledger directory aside, create a new `0700` directory, raise the snapshot `generation` and re-issue grants (a new ledger alone resets nonce history, so an old presentation could replay); restart.

Verification runs under the global ticket-state lock, including a snapshot parse and a ledger rewrite with fsyncs; a slow disk lengthens other admissions' lock wait. This is recorded, not changed.

Ticket issuance reports `effective_caveat_chain` with `delegation: legacy-bearer-not-delegated` and no caveats. A verified UCAN with no caveats is reported as `ucan-unrestricted`, not as attenuated.

The restricted store view and the project-goal site are library and shell code without a production producer: no CLI or grant turns a restricted store view on, an admitted `store info` still shows the path's references, deriver and signatures, and `resolve_project_target_with_authority` and the gateway/project chain report have no production caller. The project site admits nested goals (`goal/packages/check`), which the design text says to refuse.

Claim boundary: the grant verifier checks UCAN proof chains, parent-caveat preservation, snapshot-window revocation and holder replay; it does not prove the authorized build is correct. No sender-supplied clock, endpoint, job, output class or deadline is trusted.

## Non-claims

Verifier state does not prove ticket-user identity. Client time and client endpoint claims are not authority facts. SecretSpec resolution does not prove provider security, key freshness, or correct operator policy.
