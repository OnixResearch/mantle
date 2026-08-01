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

## Non-claims

Verifier state does not prove ticket-user identity. Client time and client endpoint claims are not authority facts. SecretSpec resolution does not prove provider security, key freshness, or correct operator policy.
