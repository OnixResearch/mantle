# Evidence: local derivation-backed remote executor slice

Date: 2026-07-01

## Question

Can `mantle remote serve --binding stdio-once` keep the deterministic fixture executor while exposing an explicit local-build executor mode that turns derivation-backed remote requests into normal Mantle build-engine execution?

## Inspected evidence

- `src/main.rs` adds `mantle remote serve --executor fixture|local-build`, defaulting to `fixture` so existing protocol tests and fixture workflows remain deterministic.
- `src/remote_build.rs::RemoteBuildExecutor` now receives the original `ConcreteBuildRequest` alongside the admitted `RemoteExecutablePlan`, so executor implementations can validate concrete payload identity before producing output metadata.
- `RemoteLocalBuildExecutor` carries the remote shell's state dir, physical output store, logical store prefix, signing keypair, trust settings, and verbosity into the executor shell.
- `execute_remote_local_build(...)` is fail-closed for unsupported input materialization and store-prefix mismatches, then dispatches derivation payloads to a Linux local-build adapter.
- `execute_remote_local_build_linux(...)` opens the normal `StoreHandle`, builds a `DerivationRegistry` from the concrete derivation payload, wires `FetchBuildService` plus `BubblewrapBuildService` through `DispatchBuildService`, runs `crunch_build::Builder`, and maps the resulting signed `PathInfo`s back into `RemoteExecutionOutcome` output metadata.
- `local_derivation_registry(...)` recomputes the derivation path from the concrete JSON payload and rejects stale/mismatched declared `.drv` identity before any build starts.
- Tests cover positive registry reconstruction, action-payload rejection, unsupported missing-input materialization, signed PathInfo-to-remote-output mapping, and wrong-output-path rejection.

## Decision

Progress slice accepted, not full drain. The remote shell now has an explicit local-build executor path for derivation-backed plans, but the complete product still needs client `mantle build --builder/--ticket` dispatch, real missing-input CAS/source upload into the remote store, full output/CAS byte transfer beyond PathInfo metadata frames, queue/status enforcement, JSON build-report integration, and production P2P transport.

## Owner

Mantle remote-build owner.

## Next action

1. Add client-side `mantle build --builder` / `--ticket` dispatch that sends concrete derivation payloads to stdio/local-build endpoints.
2. Materialize missing input refs into the remote store instead of rejecting non-empty input manifests in the local executor.
3. Extend output transfer from signed PathInfo artifacts to full output/CAS bytes with delta/full fallback reporting.

## Validation

```text
cargo test -p mantle --bin mantle remote_build::

test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 965 filtered out; finished in 0.01s
```

```text
cargo test -p mantle --test remote_stdio_cli

running 2 tests
test remote_serve_stdio_once_rejects_unknown_ticket_without_stdout_frames ... ok
test remote_serve_stdio_once_exchanges_frames_and_redeems_ticket ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

```text
git diff --check
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal p2p-remote-builders --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design p2p-remote-builders --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks p2p-remote-builders --root .

final gate output: stage=tasks valid=true verdict=PASS
```
