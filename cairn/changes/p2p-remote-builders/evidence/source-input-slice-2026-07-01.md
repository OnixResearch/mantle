# Evidence: bounded source-input upload slice

Date: 2026-07-01

## Question

Can the stdio remote-build path carry real source input bytes to the builder and materialize them before local-build executor dispatch instead of relying only on serialized derivation metadata?

## Inspected evidence

- `ConcreteBuildRequest.source_input_refs` distinguishes derivation source inputs from derivation-input refs that are already represented by serialized payloads.
- `RemoteInputUpload` now carries bounded source-input NAR artifacts with request id, input ref, kind, BLAKE3 digest, size, and payload bytes.
- Client dispatch preparation renders source-input paths as NAR payloads from the local store/host path and updates the upload frame byte count before spawning the stdio builder.
- Builder-side upload validation checks request id, uploaded source-ref set, duplicate artifacts, digest, payload size, and aggregate byte limits before ticket redemption reaches execution.
- The local-build executor ingests uploaded source NARs into castore and exports them into the remote store before build dispatch.
- Positive coverage proves a source-input artifact materializes bytes in a separate remote store; negative coverage proves tampered artifact bytes fail digest validation.

## Decision

Progress slice accepted, not full drain. The path now transfers bounded source-input NAR bytes and bounded output NAR bytes over the current JSON frame protocol. Chunked arbitrary CAS/source transfer, chunked large-output/CAS transfer, production delta/full fallback, top-level build-report integration, and authenticated P2P binding remain open.

## Owner

Mantle remote-build owner.

## Next action

1. Replace bounded inline source/output NAR artifacts with chunked CAS/object transfer.
2. Add delta/full fallback reporting for production-sized output transfer.
3. Promote remote-client JSON output into the stable top-level `crunch-build-report-v1` shape.

## Validation

```text
cargo test -p mantle --bin mantle source_input_upload -- --nocapture

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1028 filtered out; finished in 0.03s
```

```text
cargo test -p mantle --bin mantle remote_build::

test result: ok. 64 passed; 0 failed; 0 ignored; 0 measured; 966 filtered out; finished in 0.03s
```

```text
cargo test -p mantle --test remote_stdio_cli -- --quiet

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

```text
rustfmt --edition 2024 --check src/remote_build.rs src/main.rs
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
