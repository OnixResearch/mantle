# Resumable remote transfer

Mantle's resumable data plane transfers existing castore blobs/directories, NARs, source bundles, PathInfo records, attestations, and delta blobs/chunks. It does not introduce another CAS or replace output admission.

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

On reconnect, Mantle probes receiver-owned bytes and recomputes demand. A checkpoint cursor is never proof of content. Wrong-session, wrong-manifest, stale-attempt/fence, expired, regressed, forged, or content-missing checkpoints fail closed. Reassignment invalidates session authority while already verified content remains reusable through ordinary digest probing and GC ownership.

## Data plane and fallback

Bounded control DTOs carry manifests, demand, acknowledgement, checkpoint, and completion state. `mantle-remote-transfer-data-frame-v1` carries one credit-reserved chunk over any `Read`/`Write` transport such as stdio or a socket. The receiver validates the bounded header and reserves credit before allocating the payload buffer.

The production streaming report can only be constructed from a completed runtime transfer report. Negotiating a `streaming` capability label alone is not implementation evidence. Legacy whole-NAR/PathInfo frames remain compatibility fallback only and cannot produce a streaming report.

Delta failure falls back to full NAR with stable reasons (`delta-transfer-failed`, `delta-unavailable`, or `full-nar-unavailable`). Both paths still verify ordinary digests and require ordinary PathInfo/output admission.

## Completion semantics

- `already-present`: required content, closure metadata, and admitted PathInfo were verified before sender payload work; transferred bytes are zero.
- `demand-satisfied`: every demanded object was receiver-verified and required metadata/admission facts are present.
- `awaiting-admission`: bytes are complete, but trust/admission is not.
- `interrupted`: the fenced checkpoint is durable and reconnect may recompute missing content.

Transfer reports always set `output_admission_claimed` to false. Signed PathInfo, store-prefix, requested-output, artifact-attestation, producer-policy, and claim-strength checks remain separate.

## Validation boundary

Focused tests cover multi-chunk upload and download interruption, process restart, no-resend resume, receiver preseed cutoff, socket framing, stale/expired state, tampering, policy limits, castore/NAR/source/PathInfo/attestation/delta adapters, and delta-to-full fallback. Kani source harnesses cover the pure transfer core; Kani execution remains unclaimed when `cargo-kani` is unavailable.
