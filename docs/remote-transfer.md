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

## Resume and fencing

A canonical manifest binds the session to the job, attempt, fence generation, policy digest, store prefix, requested content identity, and artifact set. Checkpoints and leases are atomically written under:

```text
<state-dir>/remote-transfers/<session-id>.json
```

A per-session exclusive lock rejects concurrent writers before progress. On reconnect, Mantle probes receiver-owned bytes and recomputes demand. A checkpoint cursor is never proof of content. Wrong-session, wrong-manifest, stale-attempt/fence, expired, regressed, forged, content-missing, or tampered checkpoints fail closed or are invalidated before demand is recomputed. Reassignment invalidates session authority while already verified content remains reusable through ordinary digest probing and GC ownership.

Chunk content identity and chunk occurrence identity are distinct. Equal bytes may produce the same BLAKE3 digest at multiple offsets in one artifact. Mantle selects a demanded occurrence by canonical artifact id plus artifact-local chunk index, then verifies the complete kind/index/offset/size/digest descriptor before granting or consuming credit.

## Production data plane and fallback

Bounded control DTOs carry manifests, demand, acknowledgement, checkpoint, and completion state. `mantle-remote-transfer-data-frame-v1` carries one credit-reserved chunk over an ordered `Read`/`Write` transport such as stdio or a socket. The receiver validates the bounded header and reserves credit before allocating the payload buffer.

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

## Claim boundary

This evidence proves bounded local production stdio client/server composition, verified receiver-state resume, repeated-content chunk handling, stable manifest identity, quota/backpressure behavior, and ordinary output admission for the checked fixtures. It does not prove exactly-once network delivery, arbitrary kill-point recovery, a production P2P listener, SSH deployment, independent-machine behavior, hostile-worker honesty, compiler correctness, output trust from transfer alone, release reproducibility, or Kani execution.
