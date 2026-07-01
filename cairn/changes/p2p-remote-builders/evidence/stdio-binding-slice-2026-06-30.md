# Evidence: stdio binding validation slice

Date: 2026-06-30

## Question

Can the remote-build change add a real stdio child-output binding seam that preserves stdout-as-wire discipline without overclaiming production remote build execution?

## Inspected evidence

- `src/remote_build.rs` now decodes stdout as a stream of `u32be-length-prefixed-json` `RemoteFrame`s via `decode_remote_frame_stream(...)`.
- `validate_stdio_child_output(...)` accepts only successful child output whose stdout is entirely valid framed protocol bytes, treats stderr as bounded diagnostics, and classifies unframed stdout as terminal request-validation failure before queue admission.
- `run_stdio_remote_child(...)` is a thin imperative shell: spawn child, write framed input to stdin, collect stdout/stderr, then delegate validation to the pure stdio output core.
- `cairn/changes/p2p-remote-builders/design.md` records that stdio uses the same frame decoder and keeps stderr as diagnostics.

## Decision

Progress slice accepted, not full drain. This proves the stdio stdout-as-wire validation seam and a spawnable child shell, but not SSH-stdio, production P2P, a long-running server loop, real Mantle sandbox execution, signed PathInfo import, or `mantle build --builder` / `--ticket` dispatch.

## Owner

Mantle remote-build owner.

## Next action

1. Add a deterministic stdio fixture/server loop that exchanges the existing frames over stdin/stdout instead of validating a captured child output only.
2. Wire the loopback/stdio protocol to a real concrete Mantle build executor.
3. Persist/import signed remote outputs through the existing PathInfo/artifact attestation trust paths.

## Validation

Baseline before this slice:

```text
cargo test -p mantle --bin mantle remote_build

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```

After implementation:

```text
cargo test -p mantle --bin mantle remote_build

test remote_build::tests::protocol_state_machine_accepts_loopback_order ... ok
test remote_build::tests::status_view_redacts_ticket_secret ... ok
test remote_build::tests::stdio_child_exit_failure_is_terminal_transport_setup ... ok
test remote_build::tests::stdio_child_human_stdout_is_terminal_protocol_corruption ... ok
test remote_build::tests::loopback_session_uploads_missing_input_and_imports_trusted_output ... ok
test remote_build::tests::upload_set_mismatch_fails_before_queue_admission ... ok
test remote_build::tests::valid_request_redeems_ticket_once ... ok
test remote_build::tests::length_prefixed_frame_roundtrip_and_stdio_pollution_rejected ... ok
test remote_build::tests::untrusted_loopback_output_key_fails_before_ticket_redemption ... ok
test remote_build::tests::remote_frame_read_write_helpers_roundtrip_one_frame ... ok
test remote_build::tests::stdio_child_output_decodes_frame_stream_and_keeps_stderr_diagnostic ... ok
test remote_build::tests::stdio_stderr_summary_is_bounded ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```
