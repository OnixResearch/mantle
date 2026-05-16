## Why

Unison remote execution relocates computations by sending content identities first, letting the receiver ask for missing hashes and cache them. Mantle's distributed-build specs already require provider-neutral realization interfaces and Mantle-native realization keys, but the data-plane handshake is not yet specified. A hash-negotiated protocol is the smallest substrate for remote realization without committing to SaaS, REAPI, S3, SSH, or cluster orchestration.

## What Changes

- **Remote realization handshake**: Define request/response phases for action identity, root input identities, missing-hash negotiation, execution, and receipt return.
- **Content-addressed transfer**: Workers request only missing recipes/blobs/proof inputs by digest.
- **Provider-neutral boundary**: Concrete transports remain adapter-owned.

## Capabilities

### New Capabilities
- `hash-negotiated-remote-realization`: content-addressed remote build data plane.

### Modified Capabilities
- `distributed-builds`: remote realization interfaces gain a required hash negotiation contract.

## Impact

- **Files**: distributed build traits, adapter contracts, protocol structs, tests, docs.
- **APIs**: additive provider-neutral handshake types.
- **Testing**: fake worker tests for all-present, missing-input, digest-mismatch, and undeclared-capability cases.
