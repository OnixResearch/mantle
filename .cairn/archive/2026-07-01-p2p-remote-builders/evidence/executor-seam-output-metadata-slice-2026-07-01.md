# Evidence: remote executor seam and output metadata slice

Date: 2026-07-01

## Question

Can stdio-once route admitted `RemoteExecutablePlan`s through an executor boundary and frame execution-derived output metadata instead of deriving all builder output metadata directly from raw payload strings?

## Inspected evidence

- `src/remote_build.rs::RemoteBuildExecutor` is the new executor seam. It receives a `RemoteExecutablePlan` plus synchronized input refs and returns a `RemoteExecutionOutcome`.
- `RemoteFixtureExecutor` is the deterministic fixture implementation used by the current loopback/stdio path. Tests can inject alternate executors through `plan_remote_builder_frames_with_executor(...)`.
- `build_response_frames(...)` now plans the request, sends queued/started frames, calls the executor seam, validates request id, plan digest, output digest, output sizes, and expected output identity, then frames `BuildFinished` with per-output metadata.
- `RemoteBuildFinished` now carries output metadata: output name, logical path, content BLAKE3, size, PathInfo signing key id, and artifact-attestation BLAKE3. `RemoteOutputAdmissionReport` carries the accepted output metadata too.
- Output admission validates the produced output metadata against the original request and builder key before accepting framed child results.
- `tests/remote_stdio_cli.rs` now verifies the real child-process `mantle remote serve --binding stdio-once` stdout includes signed-output metadata fields in the `build-finished` frame.

## Decision

Progress slice accepted, not full drain. The stdio fixture now crosses a concrete executor seam and reports execution-derived metadata, but the default executor is still deterministic fixture code. Real Mantle sandbox execution and durable signed PathInfo/artifact persistence remain next.

## Owner

Mantle remote-build owner.

## Next action

1. Implement a real Mantle sandbox executor adapter for derivation-backed `RemoteExecutablePlan`s.
2. Persist/import signed PathInfo and artifact attestations from remote outputs.
3. Add client `mantle build --builder` / `--ticket` dispatch and JSON build-report integration.

## Validation

```text
cargo test -p mantle --bin mantle remote_build

test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```

```text
cargo test -p mantle --test remote_stdio_cli

running 2 tests
test remote_serve_stdio_once_rejects_unknown_ticket_without_stdout_frames ... ok
test remote_serve_stdio_once_exchanges_frames_and_redeems_ticket ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

```text
rustfmt --edition 2024 --check src/remote_build.rs tests/remote_stdio_cli.rs
git diff --check

completed successfully
```

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal p2p-remote-builders --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate design p2p-remote-builders --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks p2p-remote-builders --root /home/brittonr/git/mantle

final gate output: stage=tasks valid=true verdict=PASS
```
