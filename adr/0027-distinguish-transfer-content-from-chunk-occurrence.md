# ADR 0027: Distinguish transfer content identity from chunk occurrence identity

## Status

Accepted (2026-07-16)

## Context

Mantle's resumable transfer manifests describe artifacts as ordered bounded chunks. BLAKE3 digests identify chunk content and allow receiver-side deduplication. Equal content can legitimately occur more than once in an artifact, especially for zero-filled, padded, generated, or repetitive outputs.

The original demand lookup selected a chunk by artifact id plus BLAKE3 digest. When two occurrences in one artifact had equal bytes, the later occurrence selected the first descriptor. Full descriptor validation then compared the later offset with the first offset and rejected an otherwise valid production stream as `chunk-digest-mismatch`.

Content identity cannot also serve as artifact-position identity without excluding valid repeated content.

## Decision Drivers

- Preserve BLAKE3 as the Mantle-owned content identity.
- Preserve receiver-side reuse of one verified content chunk at multiple artifact positions.
- Keep artifact layout canonical and deterministic.
- Reject forged indices, offsets, sizes, kinds, or digests before credit or persistence.
- Avoid adding occurrence ids, another CAS key, or a wire-format version solely to repair lookup.

## Decision

Demand lookup selects a chunk occurrence by canonical artifact id plus artifact-local chunk index. Manifest validation already requires indices to be contiguous and unique within each artifact and requires offsets to match the ordered chunk sizes.

After selecting the occurrence, Mantle compares the complete artifact kind and chunk descriptor: index, offset, size, and BLAKE3 digest. Credit reservation and payload handling proceed only after that comparison succeeds.

Receiver chunk storage and verified-presence facts remain keyed by BLAKE3 content identity. Demand planning schedules one absent digest and counts later equal-content occurrences as receiver reuse, so identical bytes are not written repeatedly in one transfer round. One verified content chunk may therefore satisfy multiple equal-content occurrences during demand recomputation and artifact assembly. Artifact occurrence identity controls layout; BLAKE3 controls byte identity.

## Alternatives Considered

### Reject duplicate chunk digests within an artifact

Rejected because repetitive content is valid and common. Rejecting it would make transfer support depend on payload entropy rather than policy or identity.

### Key receiver chunks by artifact id and index

Rejected because it would discard safe cross-occurrence content reuse and turn layout identity into a second content store namespace.

### Select by artifact id, digest, and offset

Rejected as the primary lookup because a forged offset would become `chunk-not-demanded` instead of reaching the existing full-descriptor mismatch check. The canonical index selects the expected occurrence; the complete descriptor remains the validation boundary.

### Add a new random occurrence identifier

Rejected because canonical artifact-local indices already provide deterministic unique occurrence identity without new wire state.

## Consequences

- Repeated equal-content chunks stream and resume correctly.
- Receiver storage can continue deduplicating equal BLAKE3 content.
- Tampered offset, size, kind, or digest remains fail-closed before payload admission.
- Tests must include both repeated-content success and forged-descriptor rejection.
- This decision proves only correct identity separation for the bounded transfer protocol; it does not prove transport reliability, exactly-once delivery, output trust, or release reproducibility.
