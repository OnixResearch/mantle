# Evidence: bounded NAR output-transfer slice

Date: 2026-07-01

## Question

Can the stdio remote-build protocol carry output bytes beyond serialized PathInfo metadata and materialize an imported output from transferred bytes in the client store?

## Inspected evidence

- `src/remote_build.rs` now records optional NAR payload digest/size metadata on `RemoteProducedOutput` when an executor returns an output payload.
- Builder response planning now emits a second bounded `OutputTransferArtifact` with kind `nar` for outputs that carry a NAR payload, while preserving the existing `path-info-json` artifact.
- `RemoteBuildFinished.output_digest_blake3` now commits to the NAR payload digest/size metadata as part of canonical output metadata.
- Admission validation now checks all expected transfer artifacts before import, including request id, output identity, kind, BLAKE3 digest, size, duplicates, and aggregate byte limits.
- Client import planning carries the NAR payload into `RemoteOutputImportAction` and ingests it into local castore with `snix_store::nar::ingest_nar_and_hash` before calling `StoreHandle::persist_and_export_signed_output`.
- The new positive test `remote_build::tests::full_nar_transfer_artifact_materializes_remote_output_bytes` proves a framed NAR artifact is imported and exported as a real symlink output from transferred bytes.
- Negative coverage remains for tampered/missing transfer artifacts and signed PathInfo binding failures.

## Decision

Progress slice accepted, not full drain. The remote output path no longer relies on serialized PathInfo metadata alone for bounded local-build outputs. The current frame still carries inline JSON payload bytes, so large/chunked CAS transfer, source/CAS input upload, delta/full production negotiation, top-level `crunch-build-report-v1` integration, and authenticated P2P binding remain open.

## Owner

Mantle remote-build owner.

## Next action

1. Add source/CAS input upload artifacts that materialize missing builder inputs before sandbox start.
2. Replace bounded inline NAR output artifacts with chunked output/CAS transfer and delta/full fallback reporting.
3. Promote remote-client JSON output into the stable top-level `crunch-build-report-v1` shape.

## Validation

```text
cargo test -p mantle --bin mantle full_nar_transfer_artifact_materializes_remote_output_bytes -- --nocapture

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1027 filtered out; finished in 0.01s
```

```text
cargo test -p mantle --bin mantle remote_build::

test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 966 filtered out; finished in 0.01s
```

```text
cargo test -p mantle --test remote_stdio_cli

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

```text
rustfmt --edition 2024 --check src/remote_build.rs
git diff --check
cargo build -p mantle

Result: passed. cargo build finished dev profile after the focused remote-build test rail.
```

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal p2p-remote-builders --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design p2p-remote-builders --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks p2p-remote-builders --root .

final gate output: stage=tasks valid=true verdict=PASS
```
