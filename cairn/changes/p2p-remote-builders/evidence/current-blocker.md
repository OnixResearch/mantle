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
- a durable client-side remote-output import seam that accepts only admitted outputs carrying signed PathInfo bundles, validates store-prefix/path/signing-key/artifact-attestation digest binding, persists PathInfo and artifact sidecars through `StoreHandle`, exports root outputs, and records substitution reports for build-report consumers;
- signed PathInfo wire propagation from executor-produced output metadata into `BuildFinished` frames, with builder-side rejection of mismatched or unsigned executor PathInfo before emitting finished output frames;
- bounded `OutputTransferArtifact` frames for serialized PathInfo payloads, including BLAKE3 digest/size checks and fail-closed rejection when a framed PathInfo output is missing or carries a tampered transfer artifact;
- an explicit `mantle remote serve --executor fixture|local-build` selector that preserves deterministic fixture execution by default while allowing derivation-backed plans to flow through a local Mantle `StoreHandle` + `crunch_build::Builder` executor on Linux;
- fail-closed local-build executor checks for action payloads, stale derivation identity, store-prefix mismatch, input-upload set mismatch, and unexpected input-addressed output paths before output metadata is framed;
- a first client-side `mantle build --builder <endpoint> --ticket <id:secret>` stdio dispatch path that locally evaluates/lower derivations, frames concrete derivation requests, validates signed builder output frames, avoids local store lock contention by importing after the child exits, and imports admitted signed outputs for no-input derivations, content-addressed derivations with final paths returned after execution, and nested derivation input refs materialized from the serialized payload.

## Blocker

Still not drainable as a complete remote-builder product surface. The active tasks still require CAS/source input byte upload beyond nested derivation payload refs, full output/CAS byte transfer beyond serialized PathInfo artifact frames, delta/full fallback integration with top-level JSON build reports, redacted queue/status snapshots beyond ticket views, lifecycle negative tests, and a concrete authenticated production P2P binding.

The frame schema, first deterministic binding, executor seam, local derivation-backed executor adapter, client stdio dispatch path, signed PathInfo frame propagation, serialized PathInfo transfer-artifact frames, durable client-side import seam, content-addressed output path admission, and nested derivation input-ref dispatch now exist for fixture/protocol proofs and default self-spawn local-builder smokes. Completing the change honestly still requires product and compatibility decisions for the production P2P transport plus implementation work that materializes arbitrary required input/output bytes instead of relying on serialized derivation payloads and PathInfo metadata.

## Evidence

- Baseline before the signed PathInfo frame-propagation slice: `cargo test -p mantle --bin mantle remote_build::` passed with 42 tests, and `cargo test -p mantle --test remote_stdio_cli` passed with 2 tests.
- `cargo test -p mantle --bin mantle remote_build::` passed after the signed PathInfo frame-propagation slice landed: 45 passed.
- Baseline before the serialized transfer-artifact frame slice: `cargo test -p mantle --bin mantle remote_build::` passed with 45 tests.
- `cargo test -p mantle --bin mantle remote_build::` passed after the serialized transfer-artifact frame slice landed: 47 passed.
- `cargo test -p mantle --test remote_stdio_cli` passed after the serialized transfer-artifact frame slice landed: 2 passed.
- `cargo test -p mantle --bin mantle remote_build::` passed after the local-build executor slice landed: 52 passed.
- `cargo test -p mantle --test remote_stdio_cli` passed after the local-build executor slice landed: 2 passed.
- `cargo test -p crunch-glue` passed after the client-dispatch slice landed: 78 passed plus 0 doctests.
- `cargo test -p mantle --bin mantle remote_build::` passed after the client-dispatch slice landed: 56 passed.
- `cargo test -p mantle --bin mantle build_cli_accepts_remote_builder_ticket_dispatch_flags` passed after the client-dispatch slice landed: 1 passed.
- `cargo test -p mantle --test remote_stdio_cli` passed after the client-dispatch slice landed: 2 passed.
- Manual temp-store smoke for `mantle build --builder local-builder --ticket ticket-1:secret-1` passed for a no-input, input-addressed derivation after adding bwrap to PATH and produced a `mantle-remote-client-build-v1` JSON report with one imported signed output.
- `cargo test -p crunch-store artifact_attestation` passed after exporting the artifact-attestation digest helper: 4 passed.
- `cargo test -p mantle --bin mantle remote_build::` passed after the input-ref/CA-output slice landed: 61 passed.
- `cargo test -p mantle --bin mantle build_cli_accepts_remote_builder_ticket_dispatch_flags` passed after the input-ref/CA-output slice landed: 1 passed.
- `cargo test -p mantle --test remote_stdio_cli` passed after the input-ref/CA-output slice landed: 2 passed.
- `cargo test -p crunch-glue` passed after the input-ref/CA-output slice landed: 78 passed plus 0 doctests.
- `rustfmt --edition 2024 --check src/remote_build.rs src/main.rs`, `git diff --check`, and `cargo build -p mantle` passed after the input-ref/CA-output slice landed.
- Manual temp-store smokes passed for a content-addressed remote build whose output path was returned after execution and for a nested derivation-input remote build with non-empty input refs framed through `InputUpload`.
- `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` plus proposal/design/tasks gates passed after the input-ref/CA-output slice; final gate output was `stage=tasks valid=true verdict=PASS`.
- `rustfmt --edition 2024 --check src/remote_build.rs crates/crunch-store/src/attestation.rs crates/crunch-store/src/lib.rs crates/crunch-store/src/handle.rs` passed for the import seam.
- `rustfmt --edition 2024 --check src/remote_build.rs` passed after the serialized transfer-artifact frame slice.
- `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` passed with `valid: true`, 9 changes, and 19 specs validated after the serialized transfer-artifact frame slice.
- `nix run path:/home/brittonr/git/cairn#cairn -- gate proposal p2p-remote-builders --root .` passed.
- `nix run path:/home/brittonr/git/cairn#cairn -- gate design p2p-remote-builders --root .` passed.
- `nix run path:/home/brittonr/git/cairn#cairn -- gate tasks p2p-remote-builders --root .` passed.

## Next best step

Implement true source/CAS input-byte materialization and output-byte transfer: `mantle build --builder/--ticket` should upload missing CAS/source inputs with digest-checked artifact frames before sandbox start, transfer full output/CAS bytes with delta/full fallback beyond serialized PathInfo metadata, and surface top-level JSON build-report evidence. Keep the existing fixture executor as the deterministic protocol test seam while that product path lands.
