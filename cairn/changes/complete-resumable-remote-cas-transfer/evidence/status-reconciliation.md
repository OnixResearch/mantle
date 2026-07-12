# Superseding remote-transfer status reconciliation

Date: 2026-07-12
Change: `complete-resumable-remote-cas-transfer`
Requirement: `r[verification_evidence.production_transfer_completion_claim]`

## Question

Did the archived `production-remote-build-farm` package prove that production resumable remote CAS transfer was complete?

## Inspected evidence

- `cairn/archive/2026-07-06-production-remote-build-farm/tasks.md` checks I3, V5, and V6 as complete.
- `cairn/archive/2026-07-06-production-remote-build-farm/evidence/2026-07-05-session-4.md` states `I3 (chunked CAS streaming) | Not started` and says the frame transfer was not chunked/resumable CAS streaming.
- `src/remote_build.rs::RemoteInputUploadArtifact::payload` and `RemoteOutputTransferArtifact::payload` are whole `Vec<u8>` fields at this reconciliation point.
- `src/remote_build.rs::render_remote_output_nar_payload` rejects NARs larger than `MAX_REMOTE_INLINE_NAR_PAYLOAD_BYTES` and renders accepted NARs into `BoundedAsyncVecWriter`.
- `src/remote_build.rs::run_stdio_remote_child` collects the child output before frame validation, and `cmd_remote_serve` uses the aggregate framed response path.
- Baseline pueue task 259 ran the focused remote-build, `crunch-store`, and `crunch-delta` test chain successfully; those passing tests describe the pre-change baseline and do not prove resumable production transfer.
- Baseline pueue task 257 ran Cairn validation and the proposal, design, and tasks gates successfully.

## Decision

The archived checked boxes are contradicted by the archived session evidence and by the inspected production implementation. At this point, production resumable remote transfer is **incomplete and unproven**. `RemoteTransferMode::Streaming` is only a capability/report label; it is not implementation evidence.

This note supersedes current status claims only. It does not edit or reinterpret the archived files, and it does not claim that the active change has completed the missing implementation.

## Owner

The active Cairn change `complete-resumable-remote-cas-transfer` owns the implementation and current evidence.

## Next action

Replace whole production payload frames with the bounded receiver-driven transfer path, preserve ordinary output admission and fallback, then append current interruption/resume, stale-fence, tamper, quota, backpressure, large-output, and lifecycle evidence before making a completion claim.
