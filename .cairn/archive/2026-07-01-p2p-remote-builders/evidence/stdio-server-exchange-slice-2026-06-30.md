# Evidence: stdio server exchange slice

Date: 2026-06-30

## Question

Can the remote-build change prove a deterministic stdio server exchange over framed stdin/stdout without claiming a production daemon or real remote execution?

## Inspected evidence

- `src/remote_build.rs::remote_client_request_frames(...)` now renders a concrete client request sequence as `RemoteFrame`s.
- `src/remote_build.rs::plan_remote_builder_frames(...)` validates the client sequence, performs auth/request/input/upload checks, derives missing inputs, redeems tickets only after valid upload, and emits framed builder responses.
- `src/remote_build.rs::serve_stdio_remote_once(...)` is a generic `Read`/`Write` shell seam: read bounded stdin bytes, decode frames, delegate to pure planner, and write length-prefixed response frames to stdout.
- Tests prove the happy path writes decodable server response frames and the incomplete client sequence fails without partial stdout or ticket redemption.

## Decision

Progress slice accepted, not full drain. This proves the stdio server-once exchange and ticket redemption boundary for framed protocol messages. It does not prove a long-running remote builder daemon, SSH-stdio process management, production P2P networking, real Mantle sandbox execution, signed PathInfo persistence, or client-side `mantle build --builder` / `--ticket` dispatch.

## Owner

Mantle remote-build owner.

## Next action

1. Connect `serve_stdio_remote_once` to an operator-visible fixture command or hidden test server mode.
2. Replace deterministic loopback output digests with a real concrete Mantle build executor call.
3. Add client-side output import through signed PathInfo/artifact attestation verification.

## Validation

Baseline before this slice:

```text
cargo test -p mantle --bin mantle remote_build

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```

After implementation:

```text
cargo test -p mantle --bin mantle remote_build

test remote_build::tests::protocol_state_machine_rejects_out_of_order_build_request ... ok
test remote_build::tests::status_view_redacts_ticket_secret ... ok
test remote_build::tests::stdio_child_exit_failure_is_terminal_transport_setup ... ok
test remote_build::tests::stdio_child_human_stdout_is_terminal_protocol_corruption ... ok
test remote_build::tests::remote_frame_read_write_helpers_roundtrip_one_frame ... ok
test remote_build::tests::stdio_child_output_decodes_frame_stream_and_keeps_stderr_diagnostic ... ok
test remote_build::tests::upload_set_mismatch_fails_before_queue_admission ... ok
test remote_build::tests::untrusted_loopback_output_key_fails_before_ticket_redemption ... ok
test remote_build::tests::valid_request_redeems_ticket_once ... ok
test remote_build::tests::stdio_server_rejects_incomplete_client_sequence_without_redeeming_ticket ... ok
test remote_build::tests::stdio_stderr_summary_is_bounded ... ok
test remote_build::tests::stdio_server_once_exchanges_framed_request_and_response ... ok

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```
