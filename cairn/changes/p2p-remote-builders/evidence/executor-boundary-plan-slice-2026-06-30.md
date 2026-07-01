# Evidence: remote executor-boundary plan slice

Date: 2026-06-30

## Question

Can the remote-build protocol move beyond opaque payload strings by validating typed action/derivation payloads and producing an internal executable plan before the stdio fixture emits output metadata?

## Inspected evidence

- `src/remote_build.rs::plan_remote_executable_request(...)` rejects raw frontend evaluation, validates expected output identity, parses `mantle-remote-action-v1` action specs, and converts derivation payloads with `crunch_glue::convert` under the request store prefix.
- `RemoteExecutablePlan` now captures the executor boundary: source identity, command argv, command environment, system, expected outputs, and a BLAKE3 plan digest.
- Action payloads must bind the outer `action_id`, schema, executor builder, and declared output names to `ConcreteBuildRequest.expected_outputs`.
- Derivation payloads must produce the declared `.drv` path and, for known input-addressed outputs, match the requested logical output path.
- The loopback/stdio fixture output digest now hashes the typed executable plan digest instead of raw `spec_json` / `drv_json` strings.
- `tests/remote_stdio_cli.rs` sends a typed `mantle-remote-action-v1` payload through the public child-process `mantle remote serve --binding stdio-once` fixture.

## Decision

Progress slice accepted, not full drain. This proves the payload-to-plan admission seam and keeps frontend evaluation outside the builder boundary, but it still does not run the real Mantle sandbox executor or persist signed PathInfo/artifact attestations.

## Owner

Mantle remote-build owner.

## Next action

1. Execute `RemoteExecutablePlan` through the real Mantle build executor in `stdio-once`.
2. Replace deterministic fixture output metadata with signed PathInfo/artifact metadata.
3. Wire client `mantle build --builder` / `--ticket` dispatch and JSON build-report integration.

## Validation

```text
cargo test -p mantle --bin mantle remote_build

test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```

```text
cargo test -p mantle --test remote_stdio_cli

running 2 tests
test remote_serve_stdio_once_rejects_unknown_ticket_without_stdout_frames ... ok
test remote_serve_stdio_once_exchanges_frames_and_redeems_ticket ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

```text
rustfmt --edition 2024 --check src/remote_build.rs tests/remote_stdio_cli.rs
git diff --check -- src/remote_build.rs tests/remote_stdio_cli.rs

completed successfully
```

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal p2p-remote-builders --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate design p2p-remote-builders --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks p2p-remote-builders --root /home/brittonr/git/mantle

final gate output: stage=tasks valid=true verdict=PASS
```
