# Evidence: stdio client exchange validation slice

Date: 2026-06-30

## Question

Can the remote-build change validate a framed stdio child exchange on the client side and classify output-trust failures before any import side effect?

## Inspected evidence

- `src/remote_build.rs::RemoteStdioExchangeReport` combines decoded stdio frames, bounded stderr diagnostics, and the accepted output-admission report.
- `src/remote_build.rs::validate_stdio_child_exchange_output(...)` first enforces framed stdout via `validate_stdio_child_output(...)`, then validates `BuildFinished`/`OutputTransferDone`/`Done` frames through `validate_remote_builder_frames_output_import(...)`.
- Output-trust failure from a syntactically valid child frame stream is classified as `RemoteFailurePhase::OutputImport` with terminal retry class, preserving the distinction between transport/protocol corruption and trust failure.
- Positive tests prove a framed child stdout response becomes an import-admissible stdio exchange report. Negative tests prove untrusted builder keys are classified as output-import failures.

## Decision

Progress slice accepted, not full drain. This proves the client-side validation seam for a stdio child response. It does not yet launch the current Mantle binary as a child fixture, execute real remote builds, persist signed PathInfo/artifact attestations, or expose client `mantle build --builder` / `--ticket` dispatch.

## Owner

Mantle remote-build owner.

## Next action

1. Add an end-to-end child-process fixture that starts `mantle remote serve --binding stdio-once`, sends framed request bytes, and validates framed stdout through this exchange validator.
2. Replace deterministic output digests with real build executor output metadata.
3. Persist accepted remote outputs through signed PathInfo and artifact attestation import.

## Validation

Baseline before this slice:

```text
cargo test -p mantle --bin mantle remote_build

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```

After implementation:

```text
cargo test -p mantle --bin mantle remote_build

test remote_build::tests::remote_frame_read_write_helpers_roundtrip_one_frame ... ok
test remote_build::tests::stdio_child_output_decodes_frame_stream_and_keeps_stderr_diagnostic ... ok
test remote_build::tests::untrusted_loopback_output_key_fails_before_ticket_redemption ... ok
test remote_build::tests::stdio_stderr_summary_is_bounded ... ok
test remote_build::tests::upload_set_mismatch_fails_before_queue_admission ... ok
test remote_build::tests::stdio_child_exchange_output_classifies_untrusted_builder_key_as_output_import ... ok
test remote_build::tests::stdio_child_exchange_output_validates_import_admission ... ok
test remote_build::tests::valid_request_redeems_ticket_once ... ok
test remote_build::tests::stdio_server_rejects_incomplete_client_sequence_without_redeeming_ticket ... ok
test remote_build::tests::stdio_server_state_lookup_redeems_matching_ticket ... ok
test remote_build::tests::stdio_server_state_lookup_rejects_unknown_ticket ... ok
test remote_build::tests::stdio_server_once_exchanges_framed_request_and_response ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```
