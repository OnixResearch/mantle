# Store authority capabilities

Mantle splits store authority into concrete Rust values. Each value has private fields and named operations.

## Ownership table

| Capability | Owner | Allowed operations | Excluded operations |
|---|---|---|---|
| `BuildStore` | `crunch-build::Builder` | Resolve closures, transform castore nodes, calculate NAR facts, check caches, substitute outputs, and admit signed outputs. | Garbage collection, repair, source admission, arbitrary root mutation, backend replacement, and raw service access. |
| `ActionResultPort` | `crunch-build::Builder` | Discover, probe, admit, and publish action results through fixed backends. | Backend replacement, publisher replacement, and raw service access. |
| `BuildServiceStore` | Pipeline build-service wiring | Construct approved sandbox services and ingest fetched host paths. | Raw service access and store administration. |
| `OutputLookup` | `crunch-pipeline` | Find one exact `PathInfo` value and reject conflicting identities. | Output mutation and raw `PathInfoService` access. |
| `RootRegistry` | `crunch-pipeline` | Register a selected root only when its output exists. List retained roots. | Build execution, arbitrary store mutation, and raw service access. |
| `SourceAdmission` | CLI or source shell | Preflight and ingest verified sources. Adopt an approved local output. | Build execution and store administration. |
| `StoreAdmin` | Operator shell | Run garbage collection with explicit retained castore roots. | Build realization. |

## Builder boundary

`Builder` owns only `BuildStore` and `ActionResultPort`. It does not own `StoreHandle` or any writable service trait object.

The pipeline keeps `OutputLookup` and `RootRegistry`. It uses these values after the builder finishes.

`list_interest_reports` rejects a legacy registry beyond the configured
`max_roots` policy limit before extending its report map; retained-interest
records keep their separate record-count bound. The contextual reference
finish-gate scanner reads castore blobs and symlinks without mutating them,
consumes EOF at the exact scanned-byte limit, and rejects an additional byte.
Neither observation admits output content or grants root mutation authority.

The pipeline creates sandbox services through `BuildServiceStore`. It does not receive blob, directory, or `PathInfo` services.

## Compatibility facade

`StoreHandle` remains a shell-owned compatibility facade. CLI and operator shells can split it into narrow capabilities.

Archive, repair, pin, migration, and transfer shells can use this facade during migration.

Focused unit tests can use test-only service constructors. These constructors use `#[cfg(test)]` and do not change the production API.

`workspace_shell` remains an explicit shell compatibility boundary. It can adapt services required by the existing workspace protocol.

## Migration rules

1. Select the capability that owns the required operation.
2. Add a named, bounded operation to `crunch-store` when no operation exists.
3. Do not return a writable service object or a broad callback.
4. Keep policy decisions in pure cores. Keep capability methods as thin I/O shells.
5. Keep `ActionResultPort` backends fixed after construction.
6. Keep output lookup and selected root registration outside `Builder`.

Source-policy tests reject raw service escape, broad builder ownership, administrative build methods, and action-result backend replacement.

Compile-fail examples also reject garbage collection, repair, source import, root mutation, and raw blob access from `BuildStore`.

## Store backend selection

The capability views do not change with the store backend.

Only the CLI composition root chooses `--store-backend`. `StoreConfig` requires the backend, and no library constructor supplies a default. See [Store backends](store-backends.md).

## Compatibility and claim boundary

This split does not change store formats, report schemas, signatures, output identities, or supported build behavior.

The split proves only local Rust API reachability for the checked callers. It does not prove sandbox isolation or filesystem confinement.

It also does not prove output correctness, cache trust, reproducibility, provenance, availability, or release eligibility.

## Production gateway authority

The gateway's host-owned `GatewaySession` is minted only by Basalt's reviewed
Mantle operation verifier after signed UCAN proof-chain, holder-signature,
local caveat and durable replay admission. Parsing a request, ticket, policy
receipt or capability DTO does not construct a session. Legacy compatibility
tickets remain execution-exchange credentials; they grant no gateway reads,
publication or administration.

Each call checks the exact operation, target and normalized request digest and
revalidates signatures, proofs, expiry, local revocations, endpoint, deployment
audience, policy artifact and authority generation before reaching store or
attempt services. These checks do not replace lease/fence, signed-result, CAS
or PathInfo admission. Revalidation does not spend the initial holder nonce
again. Build/input/read/cancel/publish/admin permissions are operation-distinct.

`GatewayAuthorityConfig` names two already-provisioned owner-private paths:
`authority_file` (0600 regular file) and `replay_directory` (0700 directory).
The bounded JSON authority snapshot contains `version: 1`, monotonic `revision`,
positive `generation`, `issued_unix_s`, `expires_unix_s`, `endpoint`, `audience`,
the reviewed `policy_json` export as a string and its lowercase `policy_blake3`,
explicit `trusted_roots` DIDs, `keys` as arrays of 32 public-key bytes, signed
`proofs` as compact-token strings, and `revoked` token references as arrays of
32 bytes. No key, credential, directory or service is automatically provisioned.
Replace the complete authority file atomically and increase `revision` for
updates; increase `generation` when replacing grant admission policy. Missing,
expired, malformed, rolled-back or same-revision replaced snapshots fail closed.
The snapshot is reloaded on every dispatch. Protect it as operator authority,
not a caller-uploadable document. Its revocation inventory is checked with the
reviewed source-bound currentness core; the replay ledger requires synchronized
durable commits. Only the private operator channel supplies these observations.

Registered filter caveats use domain `mantle`, type `filter-v1`, and ordered keys
`00` through `07`. Payloads are strict JSON objects with `kind`, optional
`pattern`/`template`, and optional `alternatives` containing exact
`{pattern, template}` objects. Duplicate/unknown fields and ordinal gaps deny;
unknown kinds deny all. The pure bounded core checks the original target and
every ancestor prefix, so a child rewrite cannot rescue a parent-denied input.
Gateway effects retain the exact signed target rather than execute a rewrite
to a different target. Absolute logical targets are filter inputs verbatim;
other targets use `/gateway/<action-id>/<hex-target-bytes>`. Tokens and proofs
remain secret; sender-facing authority errors are fixed, credential-free codes.
