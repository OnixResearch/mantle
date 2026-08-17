# Design: Portable build receipt bundles

## Architecture

Receipt bundles are evidence transports. They must not import output payloads by default and must not decide trust from bearer tickets or archive presence. The pure core validates bundle structure, IDs, matching rules, import plans, graph-edge admissibility, and non-claim classification. The shell reads/writes bundle files, loads local output/source/store facts, persists verified receipt/graph sidecars, and renders CLI output.

## Bundle shape

Use a Mantle-owned format name such as `mantle-build-receipt-bundle-v1`. A bundle should include:

- format version, policy hash, producer identity, and created-by metadata;
- root selectors and output identities the evidence is intended to explain;
- action refs and build-correctness receipt refs when present;
- source-bundle or source-state refs used by the build when known;
- PathInfo logical paths, output names, object refs, NAR hashes/sizes, signatures, and store prefix;
- artifact and closure attestation refs or embedded canonical sidecars;
- sandbox/network policy reports and reference-scan refs when strong build-correctness evidence is claimed;
- semantic graph node and edge records for source, recipe/action, provider, sandbox, output, proof, and release/witness nodes;
- trust-basis summaries that name public verifier identities, signer key fingerprints, policy refs, revocation-list refs, expiration windows, and trust-root snapshot digests without private material;
- evidence-chain completeness markers that state whether source/input refs, action refs, sandbox/network policy, reference scans, output refs, attestations, and signatures are all present for the requested claim strength;
- BLAKE3 bundle digest and per-record digests.

Records must be deterministic and bounded. Bundle verification should reject duplicate conflicting edges, stale output identities, unsupported mandatory record kinds, unbounded metadata, and path traversal in embedded sidecar names.

## Export flow

`mantle receipt bundle export` selects local outputs or build reports and gathers their available receipt/graph/attestation sidecars. It should report incomplete evidence rather than fabricating graph edges. Export may produce a partial diagnostic bundle only under an explicit mode that refuses strong correctness claims.

## Verify flow

`mantle receipt bundle verify` checks bundle structure and content refs without requiring the original source checkout. If paired with a store archive or local output facts, it verifies output identity, store prefix, object refs, signatures, and claimed action refs match the candidate output. It must evaluate trust using either the current configured trust policy or an explicitly replayed trust-root snapshot, including revocation and expiration records. It must report missing source bundle refs, missing sandbox reports, stale action refs, revoked signer, expired trust window, or unsupported producer policies before any import claim.

## Import flow

`mantle receipt bundle import` persists only verified receipt/graph/attestation material. It must be idempotent when equivalent records already exist and fail closed on conflicts. It must not overwrite a different local graph edge or attestation with the same key unless the bytes match or an explicit migration mode is selected.

## Semantic graph portability

Imported receipt bundles should let `mantle why` and related semantic graph queries explain remote/offline outputs using portable node identities. Missing graph data should produce the existing incomplete-graph diagnostic instead of inventing edges.

## Validation strategy

- Pure positive tests for valid bundle canonicalization, output-match verification, trust-root snapshot verification, complete evidence-chain classification, import action planning, and graph-edge reconstruction.
- Pure negative tests for unsupported version, duplicate conflicting records, path traversal, missing required proof for a strong claim, stale output/action refs, wrong store prefix, untrusted signer, revoked signer, expired trust window, incomplete evidence chain, and graph conflict.
- CLI tests for export/list/verify/import human and JSON behavior, including redaction.
- Integration tests pairing a small store archive or fake remote-build output with a receipt bundle, proving accepted evidence enables `why` while missing or stale evidence fails closed.
