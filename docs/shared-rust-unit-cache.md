# Shared Rust unit cache

Mantle can share signed Rust unit results between workstations and CI clients.
This feature is optional and disabled by default.

Shared cache use does not change derivation `ActionResultRecord` or PathInfo
semantics. Rust unit results use a separate schema and trust policy.

## Lookup order

Mantle uses this fixed order:

1. Reuse the existing execution output, if it is valid.
2. Restore an admitted local castore result.
3. Discover and admit a shared result.
4. Run the compiler after an eligible miss or rejection.

Response time does not change this order. Configured source order sets the
priority for equivalent candidates.

## Read from a directory source

Use a shared directory for offline exchange or a mounted cache volume:

```bash
mantle rust-plan \
  --execute-topology \
  --local-rust-cache read \
  --shared-rust-cache read \
  --shared-rust-cache-source /srv/mantle-rust-results \
  --shared-rust-cache-trusted-key 'builder-1:PUBLIC_KEY_BASE64' \
  --shared-rust-cache-producer-policy builder-policy-v1
```

A directory source is local transport. `--shared-rust-cache-offline` can still
read it. Mantle opens directory metadata with no-follow checks and bounded
reads.

## Read from an HTTP source

Use an HTTP or HTTPS base URL as the source:

```bash
mantle rust-plan \
  --execute-topology \
  --local-rust-cache read \
  --shared-rust-cache read \
  --shared-rust-cache-source https://cache.example.invalid/mantle \
  --shared-rust-cache-trusted-key 'builder-1:PUBLIC_KEY_BASE64' \
  --shared-rust-cache-producer-policy builder-policy-v1
```

Mantle disables ambient proxies and redirects for this client. It uses a
bounded request timeout. The current policy permits zero redirects and zero
retries.

HTTP sources use these domain-specific paths:

- `rust-unit-v1/actions/<action-digest>.json`
- `rust-unit-v1/envelopes/<envelope-digest>.json`
- `rust-unit-v1/objects/<object-digest>.nar`
- `rust-unit-v1/actions/<action-digest>/<envelope-digest>.json` for publication

The lookup response contains bounded candidate claims. A candidate claim is
not reuse authority.

## Publish shared results

Select both local and shared write modes:

```bash
chmod 600 /run/secrets/mantle-rust-signing-key

mantle rust-plan \
  --execute-topology \
  --local-rust-cache read-write \
  --shared-rust-cache read-write \
  --shared-rust-cache-source /srv/mantle-rust-results \
  --shared-rust-cache-publish-target /srv/mantle-rust-results \
  --shared-rust-cache-trusted-key 'builder-1:PUBLIC_KEY_BASE64' \
  --shared-rust-cache-producer-policy builder-policy-v1 \
  --shared-rust-cache-signing-key /run/secrets/mantle-rust-signing-key
```

The signing key uses the Nix `name:base64` keypair format. The decoded value is
32 secret-key bytes followed by 32 public-key bytes.

Mantle requires a bounded regular key file. It rejects symlinks and group or
other permissions. Key buffers use zeroizing owners.

Publication uses this order:

1. Publish the complete immutable NAR object.
2. Publish the signed immutable Rust result envelope.
3. Publish the no-clobber action candidate marker.

The candidate marker is the visibility edge. Readers do not discover a result
before its object and signed envelope exist.

Exact concurrent publications deduplicate. A writer cannot replace different
immutable bytes. Different result references for one action remain visible.

## Trust and admission

The signed envelope binds these facts:

- the Rust action reference;
- the canonical Rust result reference;
- the castore root and artifact manifest;
- the BLAKE3 object reference and byte count;
- the producer and producer-policy identity;
- the claim class;
- the signer name and verifier-key digest.

Trust matches the complete Ed25519 public key and producer-policy identity.
The signer name is descriptive. A matching name with different key bytes does
not authorize reuse.

Mantle verifies the signature before object transfer. It then verifies the
object BLAKE3 identity, NAR root, complete castore tree, and artifact manifest.
Materialization uses private staging and a no-clobber commit.

If admissible records have different artifact sets, Mantle reports a strong
reuse conflict. It does not choose by source order, response time, insertion
order, or last writer.

## Offline and fallback behavior

`--shared-rust-cache-offline` keeps every HTTP result and object source
unopened. Local directory sources remain available.

A malformed, oversized, redirected, timed-out, untrusted, corrupt, truncated,
or incomplete candidate cannot produce a cache hit. Mantle can try the next
candidate. It can run the compiler when the selected policy permits fallback.

A shared publication failure does not invalidate successful local compiler
output.

## Receipt fields

Each Rust unit receipt can contain `shared_cache`. The report includes:

- the local or remote route;
- sanitized source identities;
- action, envelope, and result identities;
- authority dispositions and accepted verifier identity;
- bounded rejection reasons;
- metadata, transferred, and reused byte counts;
- compiler execution status;
- ordered publication observations.

A source identity is a BLAKE3-based public identifier. Receipts do not contain
source URLs, directory paths, URL queries, bearer tokens, private keys, or raw
configuration.

## Store backend

Local and shared Rust unit caches work with `snix` and `casita`. Casita keeps
retained castore-only unit payloads under durable `mantle/castore/` roots; a
fresh process re-ingests each envelope's `content` and verifies its
`node.postcard` against the recovered node and root name before using it.
An absent or changed retained root fails closed with `casita-root-missing` or
`casita-envelope-invalid`; it is not treated as a cache miss that may trigger
recompilation. `store usage` and `store gc` recover a pending Casita GC fence
under the mutation guard before loading the Rust retention set and verifying
payload roots. PathInfo-backed action-result outputs remain independent: they
rehydrate from `mantle/outputs/` without opening the Rust unit cache or
publishing a `mantle/castore/` root. See [Store backends](store-backends.md#capability-profiles).

## Non-claims

A shared hit proves only admitted reuse for the recorded action, policy,
authority, content, platform, and materialization facts.

It does not prove:

- compiler correctness;
- full Cargo compatibility;
- universal reproducibility;
- remote executor correctness;
- release eligibility.

Object presence, transport success, an index entry, or a signer name never
proves output authority by itself.
