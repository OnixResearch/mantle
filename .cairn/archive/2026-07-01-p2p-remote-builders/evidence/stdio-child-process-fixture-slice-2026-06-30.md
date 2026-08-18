# Evidence: stdio child-process fixture slice

Date: 2026-06-30

## Question

Can the remote-build change prove that the operator-visible `mantle remote serve --binding stdio-once` path works as an actual child process over framed stdin/stdout?

## Inspected evidence

- `tests/remote_stdio_cli.rs::remote_serve_stdio_once_exchanges_frames_and_redeems_ticket` writes a persisted ticket state, launches the real `mantle` binary through `assert_cmd`, sends length-prefixed JSON request frames to stdin, decodes framed stdout, and asserts the ticket was redeemed in `remote-builders/tickets.json`.
- `tests/remote_stdio_cli.rs::remote_serve_stdio_once_rejects_unknown_ticket_without_stdout_frames` proves the negative path: unknown tickets fail non-zero and produce no stdout frames.
- The fixture uses the public CLI surface and binary process boundary, not direct module calls, so it exercises the stdio protocol contract that operators and SSH-stdio wrappers will rely on.

## Decision

Progress slice accepted, not full drain. This proves a real child-process stdio exchange around the current fixture server. It still uses deterministic output metadata instead of a real Mantle build executor, and it does not persist signed PathInfo/artifact attestations or implement production P2P/client `mantle build --builder` dispatch.

## Owner

Mantle remote-build owner.

## Next action

1. Replace deterministic output metadata in the server fixture with a real concrete Mantle build executor result.
2. Persist admitted remote output PathInfo/artifact attestations.
3. Thread remote substitution output into `mantle build --builder` / `--ticket` and JSON build reports.

## Validation

```text
cargo test -p mantle --test remote_stdio_cli

running 2 tests
test remote_serve_stdio_once_rejects_unknown_ticket_without_stdout_frames ... ok
test remote_serve_stdio_once_exchanges_frames_and_redeems_ticket ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
