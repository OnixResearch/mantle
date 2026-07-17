## Context

The pure transfer core lives in `crates/crunch-build/src/distributed/remote_transfer.rs`; `src/remote_transfer.rs` owns bounded streaming/checkpoint I/O; and `src/remote_build.rs` wires both directions into `run_stdio_remote_child` and `cmd_remote_serve`. Current production fixtures already exercise multi-process interruption, resume, quota rejection, bounded fallback, and an 8 MiB output. The remaining defect is contradictory operator evidence: the remote-transfer document and gallery non-claim still describe the production wiring as absent.

## Success Contract

Completion requires current evidence that the checked-in gallery payload crosses multiple production chunks, interruption occurs only after a durable acknowledgement, a fresh client/server process resumes the same fenced manifest, acknowledged receiver bytes are reused rather than resent, one output is admitted through the normal store path, and tampered acknowledged bytes prevent admission. Documentation, catalog metadata, and lifecycle evidence must describe exactly that boundary.

False completion includes a standalone-shell-only test, a `streaming` capability label without data-plane execution, an inline whole-payload fixture, docs-only wording, a successful second build that discards the checkpoint, or a test that never checks output admission.

## Decisions

### Decision: Identify chunk occurrences by artifact-local index

**Choice:** Demand lookup uses `(artifact_id, chunk.index)` to select one occurrence, then validates the full kind/index/offset/size/digest descriptor before reserving credit.

**Rationale:** BLAKE3 identifies content, so equal bytes legitimately share a digest. A digest alone cannot identify which offset in one artifact is being transmitted. Artifact-local indices are canonical and unique after manifest validation, while full descriptor comparison preserves fail-closed tamper detection. Demand planning schedules a missing digest once and accounts later equal-content occurrences as reuse. ADR 0027 records this durable distinction.

### Decision: Reuse the production stdio path

**Choice:** Extend the existing `remote-build-loopback` project and `tests/remote_transfer_production.rs` instead of adding another transfer harness or protocol.

**Rationale:** The existing public build command launches `remote serve --binding stdio-once --executor local-build`, streams both directions, persists attempt/fence state, and returns outputs through ordinary admission. Testing that path avoids correlated standalone-shell evidence and avoids a second CAS or demo-only transport.

### Decision: Keep deterministic interruption debug-only

**Choice:** The runbook may name `MANTLE_TEST_REMOTE_INTERRUPT_AFTER_OUTPUT_CHUNKS` only as a checked debug-build validation seam. Release binaries continue to ignore it, and ordinary interruptions remain transport/process failures rather than operator-triggered protocol commands.

**Rationale:** A deterministic cutoff is needed for reproducible validation, but promoting a test hook into public control behavior would create unsupported semantics and weaken the production interface.

### Decision: Exercise a named multi-chunk gallery selector

**Choice:** Preserve the existing small default payload and add a `resumable-payload` selector with a deterministic named byte count large enough to cross multiple bounded chunks.

**Rationale:** Existing beginner behavior remains cheap while the resumable rail has a stable checked-in artifact. Named constants keep size intent reviewable.

### Decision: Falsify resume with receiver-byte tampering

**Choice:** After the first durable output chunk, the negative production fixture changes the exact acknowledged receiver chunk and retries the same request. The retry must report `acknowledged-chunk-missing`, leave the client store without an admitted output, and never emit output-admission evidence.

**Rationale:** This tests the critical invariant that checkpoints are hints over verified receiver state, not authority to skip bytes.

## Approach Registry

| Family | Mechanism | State | Evidence / blocker |
|---|---|---|---|
| protocol-reimplementation | Build new streaming DTOs or a second CAS | falsified | Current `src/remote_build.rs` already performs receiver-driven production streaming; duplicate machinery would weaken identity and admission boundaries. |
| digest-only occurrence lookup | Treat equal content identity as an artifact-position identity | falsified | The repeated-content gallery payload reproduced `chunk-digest-mismatch` because the second equal digest selected the first offset. |
| docs-only reconciliation | Correct stale prose without executable evidence | blocked | Necessary but insufficient because it cannot prove the checked-in gallery crosses the production data plane. |
| standalone-shell promotion | Treat `src/remote_transfer.rs` tests as production proof | falsified | The shell does not by itself prove `run_stdio_remote_child` / `cmd_remote_serve` composition. |
| production-gallery composition | Disambiguate occurrences by canonical index, then run the checked-in project through production stdio with positive and adversarial fixtures | active | The focused core and gallery tests now pass; docs and broad validation remain. |

These serial lenses are correlated because no isolated worker environment is available; deterministic repository tests and lifecycle validators remain authoritative.

## Risks / Trade-offs

- The deterministic interruption seam exists only in debug builds; docs must not imply it is a release operator feature.
- A local stdio worker proves production protocol composition but not P2P listener, SSH deployment, independent-machine behavior, hostile-kernel resistance, or exactly-once delivery.
- Transfer completion still does not admit output trust; signed PathInfo, content, attestation, requested identity, and store-prefix checks remain separate and mandatory.
