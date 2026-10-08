# Resumable remote transfer

Mantle uses one bounded resumable-transfer core for existing castore blobs/directories, NARs, source bundles, PathInfo records, attestations, and delta blobs/chunks. The production local stdio client/server path is wired through `run_stdio_remote_child` and `cmd_remote_serve`; it does not introduce another CAS or replace ordinary output admission.

## Policy

`lib/remote-builders.ncl` exposes a typed `profile.transfer_policy` record. Its defaults bound:

- chunk bytes and total bytes;
- artifact and chunk counts;
- in-flight bytes/chunks and buffered chunks;
- control and checkpoint bytes;
- idle progress and replay rounds.

Rust deserializes this as `crunch_build::distributed::RemoteTransferPolicy` and validates hard limits before manifest acceptance, source reads, payload allocation, receiver persistence, sandbox start, or output import.

## Transient-handle admission

A transient Mantle protocol message MUST NOT introduce an unknown handle,
session, lease, or binding. A lifetime-bearing declaration MUST establish it
first. The receiving building-plane boundary rejects an unknown reference
locally, without adding a rejection frame to the wire. A checkpoint or
acknowledgement records progress, not receiver content or output authority.
See [ADR 0083](../adr/0083-require-declarations-before-transient-remote-handles.md).

| Boundary | Establishing declaration | Established/unknown fixture |
| --- | --- | --- |
| Transfer chunk/session/artifact | Canonical manifest, receiver demand, and chunk credit | `crunch-build` `credit_and_chunk_validation_reject_before_payload_allocation` reserves the exact chunk; an unknown session or artifact fails before payload allocation. |
| Acknowledgement | Manifest-scoped in-flight chunk reservation | `crunch-build` `acknowledgement_is_monotonic_and_digest_checked` accepts matching scope and verified digest; unknown session or digest fails. |
| Resume checkpoint | Canonical manifest and fenced transfer lease; receiver-owned bytes establish content separately | `crunch-build` `receiver_missing_set_is_permutation_invariant_and_resume_requests_only_missing_chunks` resumes verified content; `checkpoint_tamper_scope_regression_missing_object_and_forged_cursor_fail_closed` rejects unknown sessions and acknowledged-but-missing content. |
| Resource-scoped attempt | Coordinator assignment plus matching resource lease | `mantle` `resource_attempt_report_requires_preexisting_matching_lease` accepts the assigned attempt and rejects its report when the lease is missing. |
| Production session binding | Coordinator job assignment naming the current worker, attempt, and fence | `mantle` `production_session_binding_requires_assigned_job_and_worker` resolves the assigned binding and rejects unknown jobs or unassigned workers. |
| Loopback session | Concrete request deriving the client session identity | `mantle` `loopback_session_uploads_missing_input_and_imports_trusted_output` accepts the bound session; `loopback_unknown_session_rejected_before_ticket_redemption` rejects an unrelated session without redeeming the ticket. |
| Output admission | Signed PathInfo and verified content, requested output, prefix, and attestation facts | `mantle` `durable_remote_output_import_persists_signed_pathinfo_attestation_and_report` admits signed content; `builder_rejects_executor_pathinfo_without_signature` rejects an unsigned result, and `durable_remote_output_import_rejects_missing_pathinfo_bundle` rejects missing PathInfo despite transfer progress. |

These checks constrain protocol shape; none substitutes for digest verification,
proves delivery or crash consistency, or promotes an acknowledgement to trust.

## Resume and fencing

A canonical manifest binds the session to the job, attempt, fence generation, policy digest, store prefix, requested content identity, and artifact set. Checkpoints and leases are atomically written under:

```text
<state-dir>/remote-transfers/<session-id>.json
```

A per-session exclusive lock rejects concurrent writers before progress. On reconnect, Mantle probes receiver-owned bytes and recomputes demand. A checkpoint cursor is never proof of content. Wrong-session, wrong-manifest, stale-attempt/fence, expired, regressed, forged, content-missing, or tampered checkpoints fail closed or are invalidated before demand is recomputed. Reassignment invalidates session authority while already verified content remains reusable through ordinary digest probing and GC ownership.

The production client and its child bind an active session to the same assigned job, attempt, and fence within those processes. Durable transfer checkpoints do not make executor effects or session ownership process-durable. Reconnection or reassignment is explicit, not an automatic retry by the typed effect loop; the receiver's verified bytes, not an old acknowledgement, determine what can be reused.

Chunk content identity and chunk occurrence identity are distinct. Equal bytes may produce the same BLAKE3 digest at multiple offsets in one artifact. Mantle selects a demanded occurrence by canonical artifact id plus artifact-local chunk index, then verifies the complete kind/index/offset/size/digest descriptor before granting or consuming credit.

## Production data plane and fallback

The checked local stdio path uses the `no_std + alloc` `crunch-remote-core`
for bounded attempt, transfer, output, receipt, and effect decisions, with
Mantle-owned output facts and selected capability ports in
`crunch-remote-app`. The host adapters still send framed bytes, execute the
builder, check credentials against an observed clock, manage workspace and
attempt persistence, publish telemetry, and perform ordinary physical store
admission. A dependent effect cannot succeed without its matching adapter
observation. This is a scoped I4/I5 composition, not a claim that all remote
resource/locality policy or SSH, local, and external-batch provider behavior
has moved into the core and application ports: those I2/I3 migrations remain
open. Snix `PathInfo` and substitution reports, credential material, handles,
and process/network types stay in std adapters, not the portable contracts.

Transfer completion is not store admission: partially published physical
bytes can remain after a failed attempt without becoming an admitted output.

Bounded control DTOs carry manifests, demand, acknowledgement, checkpoint, and completion state. `mantle-remote-transfer-data-frame-v1` carries one credit-reserved chunk over an ordered `Read`/`Write` transport such as stdio or a socket. The receiver validates the bounded header and reserves credit before allocating the payload buffer.

The std transport checks canonical frame, manifest, normalized output-key, and chunk-digest scope against the assigned request; the core's receipt preimage does not verify a signature or confer output trust. Signed PathInfo, verified content, requested output, logical store prefix, and attestation are checked at the ordinary store boundary.

The production local stdio route performs this sequence for both directions:

1. `run_stdio_remote_child` launches the checked production client protocol.
2. `cmd_remote_serve --binding stdio-once --executor local-build` runs the production server path.
3. The client streams only receiver-demanded input artifacts before sandbox execution.
4. The server streams the built output manifest and receiver-demanded chunks.
5. Transfer completion remains non-authoritative until the client admits signed PathInfo, content, requested output, logical store prefix, and artifact-attestation evidence through the ordinary store path.

`RemoteInputUploadArtifact::payload` and `RemoteOutputTransferArtifact::payload` remain compatibility DTOs for bounded fixture/bootstrap and full-artifact seams. Their presence is not streaming evidence. A `streaming` report can be constructed only from a completed runtime transfer report.

The production `--remote-delta` option advertises delta/full/streaming capability. A production delta hit is not yet bound. When delta is unavailable, Mantle records `mode = "full"` with `fallback_reason = "delta-unavailable"`, transfers the full NAR through the same bounded chunk data plane, and still performs ordinary output admission. It does not report a delta hit.

## Completion semantics

- `already-present`: required content, closure metadata, and admitted PathInfo were verified before sender payload work; transferred bytes are zero.
- `demand-satisfied`: every demanded object was receiver-verified and required metadata/admission facts are present.
- `awaiting-admission`: bytes are complete, but trust/admission is not.
- `interrupted`: the fenced checkpoint is durable and reconnect may recompute missing content.

Transfer reports always set `output_admission_claimed` to false. Signed PathInfo, store-prefix, requested-output, artifact-attestation, producer-policy, and claim-strength checks remain separate.

## Operator and validation workflow

The supported local production workflow is under
[`examples/projects/remote-build-loopback/`](../examples/projects/remote-build-loopback/).
Its default selector remains a small one-use-ticket example; `.#resumable-payload`
produces deterministic repeated-content bytes large enough to cross multiple
production chunks.

The canonical deterministic rail uses a debug-build-only interruption seam:

```bash
nix develop -c cargo test -p mantle --test remote_transfer_production \
  'gallery_resumable_remote_transfer_' -- --nocapture --test-threads=1
```

The positive fixture interrupts after one durable output acknowledgement, starts fresh client/server processes, checks the same manifest BLAKE3, verifies reused bytes and missing-chunk progress, and admits one byte-checked output. The negative fixture changes the acknowledged receiver chunk and proves resume fails with `acknowledged-chunk-missing` before client output admission.

`MANTLE_TEST_REMOTE_INTERRUPT_AFTER_OUTPUT_CHUNKS` and its input counterpart are debug-test seams, not release operator controls. Release binaries ignore them. They provide deterministic cutoff evidence; they do not prove arbitrary process-kill timing or crash consistency beyond the recorded checkpoint boundary.

Focused production regressions also cover multi-chunk input resume, ticket quota rejection before checkpoints/admission, delta-unavailable full fallback, and an 8 MiB output crossing more than one hundred acknowledged chunks.

The focused I4/I5 proofs cover core effect transitions, application ports, production transfer and local stdio adapters, accepted/rejected stdio wire bytes, output-key/fence/replay denial, and a real unskipped worker capture. They do not establish complete I2 resource migration, I3 provider-port cutover, or the Cairn V1–V5 gates. The recorded partial results and the separately repaired live-daemon regression are in the [remote hexagon focused evidence](../.cairn/changes/separate-remote-build-hexagon/evidence/focused-validation.md); the full filtered root suite has not been rerun after that repair.

### Scoped V2 original/current parity

The checked-in V2 parity fixtures include a test-only instrumented `da00f` original signed two-Worker run, not a pristine original executable or whole-CLI successor equivalence. The original used the same job at fences 1→2: a retry before backoff left durable state and bytes unchanged, a stale signed first result was rejected before client PathInfo/export, and the successor signed result was imported and completed. An independent current physical two-Worker test exercises the corresponding fence, backoff, stale-before-import, capacity-release, and signed successor path; the current accepted CLI test exercises one real Worker, not a two-Worker CLI retry.

The accepted current CLI test captures only bounded private Worker **response** stdout. Its parser distinguishes nine semantic control kinds from streaming sideband controls and manifest-bound data headers, then checks each raw output chunk's declared size, artifact/chunk membership, scope, and BLAKE3 digest before advancing. The 4,096 imported bytes, signed PathInfo/NAR, 14 ordered durable telemetry events, and original signed output facts match within that scoped comparison. CLI dispatch matches the Worker's advertised signing-key **name**, as the default local route does; this handshake is not a full-key-material authentication claim. A separate post-import `store verify --trusted-public-keys` with the full public verifying key checks the actual signed store output.

The rejected current CLI case spawns a genuinely absent builder program after dispatch, observes the underlying `ENOENT`, preserves its unused one-use ticket, and records a lost job/failed attempt with six ordered immutable telemetry events and no imported output or PathInfo. The bounded original failed-adapter receipt has the same observed diagnostic class, state, and event projection. Neither original CLI typed EffectIds nor quantified original CLI resources were observed; V2 remains open for the full identities, effects, and whole dual-path compatibility requirements. The exact sources, prior failed focused runs, final 7/7 focused receipt, and nonclaims are in the [focused validation evidence](../.cairn/changes/separate-remote-build-hexagon/evidence/focused-validation.md).

## Claim boundary

This evidence proves bounded local production stdio client/server composition, verified receiver-state resume, repeated-content chunk handling, stable manifest identity, quota/backpressure behavior, and ordinary output admission for the checked fixtures. It does not prove exactly-once network delivery, arbitrary kill-point recovery, a production P2P listener, SSH deployment, independent-machine behavior, hostile-worker honesty, compiler correctness, output trust from transfer alone, release reproducibility, or Kani execution.
