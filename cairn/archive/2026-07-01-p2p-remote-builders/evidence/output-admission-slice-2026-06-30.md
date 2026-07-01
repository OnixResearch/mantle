# Evidence: remote output admission slice

Date: 2026-06-30

## Question

Can the remote-build change prove a client-side pre-import validation path for framed remote outputs without claiming durable signed PathInfo/artifact persistence yet?

## Inspected evidence

- `src/remote_build.rs::RemoteOutputAdmissionReport` records the output digest, builder signing key, store prefix, and transfer report that passed admission checks.
- `src/remote_build.rs::validate_remote_output_admission(...)` rejects request-id mismatch, store-prefix mismatch, malformed BLAKE3 digest strings, inconsistent transfer reports, transfer/build-result signing-key mismatch, and untrusted builder keys.
- `src/remote_build.rs::validate_remote_builder_response_output_import(...)` extracts `BuildFinished`, `OutputTransferDone`, and `Done` frames from a framed builder response before delegating to the admission core.
- `src/remote_build.rs::RemoteBuilderFrameResponse` now carries the transfer report alongside response frames so the shell can compare framed data with the planned transfer summary.
- Positive test coverage proves a framed stdio server-once exchange produces an import-admissible response. Negative tests prove untrusted builder keys, transfer-key mismatch, and malformed output digests fail before import.

## Decision

Progress slice accepted, not full drain. This proves pre-import validation for framed remote output metadata and transfer summaries. It does not yet persist signed PathInfo, verify real artifact attestations, execute a real remote Mantle build, or connect the result to `mantle build --builder` / `--ticket` JSON build reports.

## Owner

Mantle remote-build owner.

## Next action

1. Replace deterministic loopback output digest generation with a real Mantle build executor result.
2. Map admitted framed output metadata onto signed PathInfo/artifact attestation persistence.
3. Thread accepted remote substitution/transfer fields into the normal JSON build report.

## Validation

Baseline before this slice:

```text
cargo test -p mantle --bin mantle remote_build

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.01s
```

After implementation:

```text
cargo test -p mantle --bin mantle remote_build

test remote_build::tests::remote_frame_read_write_helpers_roundtrip_one_frame ... ok
test remote_build::tests::remote_output_admission_rejects_malformed_digest ... ok
test remote_build::tests::remote_output_admission_rejects_untrusted_builder_key ... ok
test remote_build::tests::remote_output_admission_rejects_transfer_key_mismatch ... ok
test remote_build::tests::stdio_child_human_stdout_is_terminal_protocol_corruption ... ok
test remote_build::tests::stdio_child_output_decodes_frame_stream_and_keeps_stderr_diagnostic ... ok
test remote_build::tests::untrusted_loopback_output_key_fails_before_ticket_redemption ... ok
test remote_build::tests::upload_set_mismatch_fails_before_queue_admission ... ok
test remote_build::tests::valid_request_redeems_ticket_once ... ok
test remote_build::tests::stdio_stderr_summary_is_bounded ... ok
test remote_build::tests::stdio_server_rejects_incomplete_client_sequence_without_redeeming_ticket ... ok
test remote_build::tests::stdio_server_once_exchanges_framed_request_and_response ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```
