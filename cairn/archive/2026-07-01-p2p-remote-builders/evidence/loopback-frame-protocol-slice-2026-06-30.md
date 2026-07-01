# Evidence: loopback frame/protocol slice

Date: 2026-06-30

## Question

Can the remote-build change make concrete progress on the transport/frame decision and a first end-to-end session path without overclaiming production P2P remote builders?

## Inspected evidence

- `cairn/changes/p2p-remote-builders/design.md` now records the first frame contract as `u32be-length-prefixed-json` and the first implementation binding as in-process loopback. Stdio/SSH-stdio are constrained to the same frame contract; production P2P remains a later binding.
- `src/remote_build.rs` now has pure frame encode/decode, explicit phase transitions, input manifest/upload validation, output trust preflight, transfer-mode planning with delta-to-full fallback, failure classification, session lease planning, and a deterministic loopback session report.
- `src/remote_build.rs::cmd_remote` now exposes protocol metadata with frame encoding and supported bindings while still reporting `metadata-only` instead of claiming a production listener.

## Decision

Progress slice accepted, not full drain. The slice proves protocol control-flow and first-binding mechanics for loopback only. It does not prove production P2P transport, a live stdio child server, real Mantle sandbox execution on a remote host, signed PathInfo persistence from a remote builder, or build-report integration for `mantle build --builder` / `--ticket`.

## Owner

Mantle remote-build owner.

## Next action

1. Add a real stdio child binding that reserves stdout for length-prefixed frames and sends all diagnostics to stderr.
2. Wire loopback/stdio sessions to a real concrete Mantle build executor instead of deterministic core output digests.
3. Add client-side `mantle build --builder` / `--ticket` dispatch and JSON build-report integration once output import persists signed PathInfo and artifact attestations.

## Validation

Baseline before edits (tool output in session):

```text
cargo test -p mantle --bin mantle remote_build

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```

After implementation and formatting:

```text
cargo test -p mantle --bin mantle remote_build

test remote_build::tests::output_trust_requires_trusted_key_and_prefix ... ok
test remote_build::tests::protocol_mismatch_fails_closed ... ok
test remote_build::tests::oversized_frame_length_is_rejected_before_payload_read ... ok
test remote_build::tests::missing_inputs_are_derived_from_declared_refs ... ok
test remote_build::tests::protocol_state_machine_accepts_loopback_order ... ok
test remote_build::tests::protocol_state_machine_rejects_out_of_order_build_request ... ok
test remote_build::tests::status_view_redacts_ticket_secret ... ok
test remote_build::tests::upload_set_mismatch_fails_before_queue_admission ... ok
test remote_build::tests::loopback_session_uploads_missing_input_and_imports_trusted_output ... ok
test remote_build::tests::valid_request_redeems_ticket_once ... ok
test remote_build::tests::untrusted_loopback_output_key_fails_before_ticket_redemption ... ok
test remote_build::tests::length_prefixed_frame_roundtrip_and_stdio_pollution_rejected ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 962 filtered out; finished in 0.00s
```

Formatting:

```text
rustfmt src/remote_build.rs
Task 1922: completed successfully
```
