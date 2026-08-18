# Evidence: stdio serve-once CLI slice

Date: 2026-06-30

## Question

Can the remote-build change expose the framed stdio server-once seam through an explicit command path without turning default `remote serve` into an overclaimed production listener?

## Inspected evidence

- `src/main.rs::RemoteAction::Serve` now has a typed `--binding metadata|stdio-once` option. The default remains `metadata`.
- `src/remote_build.rs::cmd_remote(...)` receives the logical store prefix and routes `remote serve --binding stdio-once` to the framed stdio planner.
- `src/remote_build.rs::plan_stdio_remote_once_from_state(...)` reads bounded framed stdin, decodes client frames, looks up the ticket id from the auth frame in persisted state, plans the exchange, and mutates the ticket state only for valid queued requests.
- `src/remote_build.rs::cmd_remote_serve(...)` saves ticket state before writing framed response frames to stdout, preserving stdout as the protocol stream in stdio mode.
- Positive tests prove state-backed stdio planning redeems the matching ticket. Negative tests prove an unknown ticket fails without mutating state.

## Decision

Progress slice accepted, not full drain. This creates an operator-visible `stdio-once` fixture path and keeps default `remote serve` metadata-only. It does not claim a daemon loop, SSH-stdio supervision, production P2P transport, real Mantle build execution, signed PathInfo persistence, or `mantle build --builder` / `--ticket` client dispatch.

## Owner

Mantle remote-build owner.

## Next action

1. Add a client fixture that launches `mantle remote serve --binding stdio-once` as a child and exchanges framed request/response bytes end to end.
2. Replace deterministic loopback output generation with a real concrete Mantle build executor result.
3. Persist admitted remote outputs through signed PathInfo and artifact-attestation storage.

## Validation

Baseline before this slice:

```text
cargo test -p mantle --bin mantle remote_build

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```

After implementation:

```text
cargo test -p mantle --bin mantle remote_build

test remote_build::tests::stdio_child_human_stdout_is_terminal_protocol_corruption ... ok
test remote_build::tests::length_prefixed_frame_roundtrip_and_stdio_pollution_rejected ... ok
test remote_build::tests::remote_frame_read_write_helpers_roundtrip_one_frame ... ok
test remote_build::tests::stdio_child_output_decodes_frame_stream_and_keeps_stderr_diagnostic ... ok
test remote_build::tests::untrusted_loopback_output_key_fails_before_ticket_redemption ... ok
test remote_build::tests::valid_request_redeems_ticket_once ... ok
test remote_build::tests::upload_set_mismatch_fails_before_queue_admission ... ok
test remote_build::tests::stdio_stderr_summary_is_bounded ... ok
test remote_build::tests::stdio_server_rejects_incomplete_client_sequence_without_redeeming_ticket ... ok
test remote_build::tests::stdio_server_state_lookup_rejects_unknown_ticket ... ok
test remote_build::tests::stdio_server_state_lookup_redeems_matching_ticket ... ok
test remote_build::tests::stdio_server_once_exchanges_framed_request_and_response ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```
