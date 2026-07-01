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
- deterministic fixture executor output metadata carrying output name, logical path, content BLAKE3, size, PathInfo signer id, and artifact-attestation BLAKE3;
- a durable client-side remote-output import seam that accepts only admitted outputs carrying signed PathInfo bundles, validates store-prefix/path/signing-key/artifact-attestation digest binding, persists PathInfo and artifact sidecars through `StoreHandle`, exports root outputs, and records substitution reports for build-report consumers.

## Blocker

Still not drainable as a complete remote-builder product surface. The active tasks still require a real Mantle sandbox executor adapter for derivation-backed plans, client-side `mantle build --builder/--ticket` dispatch, CAS/source input upload beyond fixture refs, signed output transfer framing beyond in-memory PathInfo bundles, delta/full fallback integration with top-level JSON build reports, redacted queue/status snapshots beyond ticket views, lifecycle negative tests, and a concrete authenticated production P2P binding.

The frame schema, first deterministic binding, executor seam, and durable client-side import seam now exist for fixture proofs, but completing the change honestly still requires product and compatibility decisions for the production P2P transport plus implementation work that runs real Mantle builds remotely.

## Evidence

- `cargo test -p mantle --bin mantle remote_build::` passed after the import seam landed: 42 passed.
- `cargo test -p mantle --test remote_stdio_cli` passed after the wire-compatible optional PathInfo field landed: 2 passed.
- `cargo test -p crunch-store artifact_attestation` passed after exporting the artifact-attestation digest helper: 4 passed.
- `rustfmt --edition 2024 --check src/remote_build.rs crates/crunch-store/src/attestation.rs crates/crunch-store/src/lib.rs crates/crunch-store/src/handle.rs` passed.
- `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` passed with `valid: true`, 9 changes, and 19 specs validated.
- `nix run path:/home/brittonr/git/cairn#cairn -- gate proposal p2p-remote-builders --root .` passed.
- `nix run path:/home/brittonr/git/cairn#cairn -- gate design p2p-remote-builders --root .` passed.
- `nix run path:/home/brittonr/git/cairn#cairn -- gate tasks p2p-remote-builders --root .` passed.

## Next best step

Implement the real executor slice: derivation-backed `RemoteExecutablePlan` → normal Mantle build execution in the remote shell → framed signed PathInfo transfer from builder to client → `mantle build --builder/--ticket` dispatch and top-level JSON build-report evidence. Keep the existing fixture executor as the deterministic protocol test seam while that real executor lands.
