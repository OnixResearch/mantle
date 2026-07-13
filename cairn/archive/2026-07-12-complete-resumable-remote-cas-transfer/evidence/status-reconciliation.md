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

## Task 1017 production checkpoint (historical)

### Question

Do the implementation commits through `a6a8d615` and pueue task 1017 supersede the original production-streaming blocker?

### Inspected evidence

- Task 1017 passed 109 `remote_build::tests::`, 17 `remote_transfer::tests::` plus two subprocess legs, and 3 `remote_transfer_production` tests.
- The production tests interrupt and resume multi-chunk input and output transfers across fresh client/server processes, inspect durable checkpoints, assert output reused bytes, compare imported output bytes, and reject an upload quota before checkpoint/admission.
- The focused fence/security tests reject stale post-write/final-chunk state before acknowledgement or completion, reject stale input before source disclosure, reject excess credit with zero disclosure, and fail closed on hostile authority-state paths.

### Decision

Yes for the bounded production streaming replacement and interrupted production resume tasks: their previous blocker is superseded, and those task boxes may be checked. At task 1017 time, no broad completion claim followed because that packet did not yet prove production delta fallback, current store/delta package reruns, a production-scale 8 MiB rail, or Kani execution. The later fallback/scale checkpoint below supersedes those three production blockers while retaining the Kani non-claim.

### Owner

The active change remains the owner until the exact unchecked fallback and final validation tasks are proven.

### Next action

Exercise delta-unavailable and delta-failure fallback through the production interactive path, rerun the current store/delta and production-scale large-output rails, then reassess the final completion task. Do not sync or archive before those requirements are proven.

## Fallback and scale checkpoint

### Question

Do the public capability, fallback, package, and 8 MiB results close the remaining production-transfer blockers?

### Inspected evidence

- Public `mantle build --remote-delta` maps to typed delta/full/streaming client capabilities; the production server advertises the same bounded set.
- Pueue task 1140 passed 199 `crunch-store` library tests, 36 `crunch-delta` library tests, and 5 production integration tests.
- The production fallback test observes `mode = "full"` plus `fallback_reason = "delta-unavailable"`, verifies multiple bounded chunks, and reaches ordinary admission without claiming delta reuse.
- The production-scale test transfers and admits a byte-identical 8 MiB output through more than 100 acknowledged chunks.
- Pueue task 1166 independently passed current store, delta, 109 remote-build, and five production integration tests.

### Decision

Yes. The exact production fallback and scale blockers are proven. Pueue task 1196 then reported valid Cairn validation and PASS proposal, design, and tasks receipts, so the fallback and final validation tasks may both be checked without broadening the Kani non-claim.

### Owner

The active change owns the final lifecycle gate packet. No sync/archive action is authorized in this session.

### Next action

Commit the lifecycle evidence and checked task markers. Do not sync or archive the active change in this session.
