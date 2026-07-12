## Reconciliation

- [x] [serial] r[verification_evidence.production_transfer_completion_claim] Add a tracked superseding status record that cites the archived checked I3/V5/V6 tasks, the contradictory session evidence, and the current inline NAR/input payload implementation without rewriting the archive. Evidence: `evidence/status-reconciliation.md`.
- [x] [depends:transfer-status-reconciliation] r[verification_evidence.production_transfer_completion_claim] Remove or narrow any current operator wording that presents `RemoteTransferMode::Streaming` or the archived farm change as implementation-complete before new evidence exists. Evidence: `evidence/status-reconciliation.md` and the narrowed enum documentation in `src/remote_build.rs`.

## Implementation

- [ ] [depends:fence-durable-remote-attempts] r[remote_builds.attempt_scoped_transfer_resume] Add typed transfer-session, attempt/fence, manifest, demand, chunk, acknowledgement, checkpoint, credit, and completion DTOs with named bounds.
- [ ] [depends:transfer-dtos] r[store_transports.resumable_castore_sessions] Implement a pure canonical-manifest validator and deterministic receiver-missing-set/resume planner over existing castore, PathInfo, source-bundle, attestation, NAR, and delta identities.
- [ ] [depends:transfer-planner] r[store_transports.receiver_driven_backpressure] Implement pure quota arithmetic and receiver-credit decisions that reject before disallowed allocation or buffering.
- [ ] [depends:transfer-planner] r[store_transports.content_presence_early_cutoff] Implement pure `already-present`, `demand-satisfied`, `continue`, and `reject` cutoff decisions from verified receiver facts.
- [ ] [depends:transfer-core] r[store_transports.resumable_castore_sessions] Add shell streaming adapters for castore objects, NAR bytes, source bundles, PathInfo/attestations, and existing delta frames without adding another CAS.
- [ ] [depends:transfer-shell] r[remote_builds.attempt_scoped_transfer_resume] Persist bounded checkpoints and session leases, revalidate receiver state after reconnect, and invalidate stale attempt/fence checkpoints while retaining reusable verified castore objects.
- [ ] [depends:transfer-shell] r[store_transports.receiver_driven_backpressure] Thread typed Nickel transfer/backpressure/quota policy into runtime Rust and enforce credits on both upload and download paths.
- [ ] [depends:transfer-shell] r[store_transports.content_presence_early_cutoff] Stop sender work after verified receiver cutoff and report zero/partial transferred bytes without treating expected CA identity or unadmitted metadata as completion.
- [ ] [depends:transfer-shell] r[remote_builds.attempt_scoped_transfer_resume] Replace production whole-payload frames with the streaming path; retain inline payloads only behind an explicitly bounded fixture/bootstrap capability and report that narrower mode honestly.
- [ ] [depends:transfer-shell] r[remote_builds.attempt_scoped_transfer_resume] Preserve full-NAR and delta fallback through ordinary digest and output-admission checks with stable fallback reasons.

## Verification

- [ ] [depends:transfer-core] r[store_transports.resumable_castore_sessions] Add table/property/Kani tests for canonical ordering, permutation-invariant missing sets, monotonic acknowledgement, replay-safe checkpoints, and overflow-safe quota accounting.
- [ ] [depends:transfer-shell] r[remote_builds.attempt_scoped_transfer_resume] Positive: interrupt a multi-chunk input and output transfer, restart the transport/coordinator, resume the current fenced session, and verify only missing content is resent before ordinary output admission.
- [ ] [depends:transfer-shell] r[store_transports.content_presence_early_cutoff] Positive: preseed complete receiver content and prove transfer cuts off with verified `already-present`, zero unrequested payload bytes, and no output-trust overclaim.
- [ ] [depends:transfer-shell] r[store_transports.receiver_driven_backpressure] Negative: exceed chunk, credit, object, total-byte, checkpoint, and idle-progress policy and prove rejection occurs before unbounded allocation, persistence, sandbox start, or output import.
- [ ] [depends:transfer-shell] r[remote_builds.attempt_scoped_transfer_resume] Negative: submit stale-fence, wrong-manifest, tampered-digest, regressed-acknowledgement, missing-object, and forged-cursor checkpoints and prove deterministic fail-closed diagnostics.
- [ ] [depends:transfer-verification] r[verification_evidence.production_transfer_completion_claim] Run focused store/delta/remote-build tests, a bounded local multi-process large-output rail, Cairn validate, and proposal/design/tasks gates; record exact command evidence before checking completion tasks.
