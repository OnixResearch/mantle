# Current blocker: P2P remote builders

Date: 2026-07-01

## Current state

The package now captures multiple pure/protocol slices in `src/remote_build.rs` and CLI fixture coverage:

- protocol metadata for `mantle-remote-build/1`;
- ticket creation/list/inspect/reveal/revoke state and redaction;
- version/ALPN/endpoint/capability validation;
- delayed ticket redemption for valid queued requests;
- raw frontend/Nickel request rejection in pure request-shape validation;
- missing-input set derivation, bounded upload checks, and output-trust separation helpers;
- deterministic loopback and `stdio-once` frame bindings using length-prefixed JSON frames with stdout-as-wire discipline;
- state-backed `mantle remote serve --binding stdio-once` fixture coverage;
- a `RemoteBuildExecutor` seam that turns admitted `RemoteExecutablePlan`s into execution-derived output metadata;
- deterministic fixture executor output metadata carrying output name, logical path, content BLAKE3, size, PathInfo signer id, and artifact-attestation BLAKE3.

## Blocker

Still not drainable as a complete remote-builder product surface. The active tasks still require a real Mantle sandbox executor adapter for derivation-backed plans, durable signed PathInfo/artifact persistence, client-side `mantle build --builder/--ticket` dispatch, CAS/source input upload beyond fixture refs, signed output transfer/import, delta/full fallback integration with JSON build reports, redacted queue/status snapshots beyond ticket views, lifecycle negative tests, and a concrete authenticated production P2P binding.

The frame schema and first deterministic binding now exist for fixture proofs, but completing the change honestly still requires product and compatibility decisions for the production P2P transport plus implementation work that runs real Mantle builds remotely.

## Next best step

Implement or split the real executor/import slice: derivation-backed `RemoteExecutablePlan` → normal Mantle build execution in the remote shell → signed PathInfo and artifact sidecars → client admission/import → JSON build-report evidence. Keep the existing fixture executor as the deterministic protocol test seam while that real executor lands.
