# Delta Transfer Specification

## Purpose

Defines crunch-native delta transfer for remote substitution using castore-backed
objects, negotiated chunk profiles, and trust-preserving acceptance rules.
## Requirements
### Requirement: Castore-native Delta Transfer

Mantle MUST describe delta-capable store transfer in terms of castore-backed
objects: logical output identities, directory nodes, whole blobs, and
content-defined blob chunks. Whole blobs and chunks MUST be identified by the
BLAKE3 digest of their finalized content bytes. Delta transfer MUST NOT require
NAR archives to be the primary transfer unit between mantle peers.

#### Scenario: Sender streams castore objects instead of NAR payloads

- GIVEN a sender and receiver that both support mantle delta transfer
- WHEN the sender plans a transfer for a requested output
- THEN the plan describes reusable directory or blob identities plus missing
  blob or chunk payloads
- AND the receiver can ingest the result into castore without first rebuilding
  a NAR archive as the canonical data model

### Requirement: Protocol Negotiation Declares Version and Chunk Profile

Delta-capable peers MUST negotiate a protocol version and a chunk profile before
chunk-level reuse planning begins.

Protocol version 1 MUST use FastCDC with 128 KiB minimum, 256 KiB average, and
512 KiB maximum chunk size.

#### Scenario: Peers agree on the initial chunk profile

- GIVEN two peers that both implement delta protocol version 1
- WHEN they negotiate a transfer session
- THEN both peers use FastCDC with 128 KiB minimum, 256 KiB average, and 512
  KiB maximum chunk size for chunk-level planning
- AND chunk identities are interpreted as BLAKE3 digests of finalized content
  bytes

### Requirement: Receiver Compatibility Manifest

A delta-capable receiver MUST expose a bounded compatibility manifest that is
scoped to the requested output or requested closure transfer. It MUST state the
logical store prefix and which reusable outputs, directory nodes, blobs, and
chunks are actually present with backing castore content. Manifest construction
MUST NOT require downloading content bytes merely to decide presence.

#### Scenario: Prefix mismatch rejects delta reuse

- GIVEN a sender using logical store prefix `/crunch/store`
- AND a receiver reporting logical store prefix `/nix/store`
- WHEN delta negotiation begins
- THEN the peers reject delta reuse for that session
- AND mantle reports the prefix mismatch explicitly

#### Scenario: Metadata without backing content is treated as absent

- GIVEN the receiver has `PathInfo` for an output but its referenced castore
  node is missing locally
- WHEN the receiver builds its compatibility manifest
- THEN that output is not advertised as reusable content

#### Scenario: Large store does not require whole-store inventory export

- GIVEN a receiver whose store is much larger than the requested transfer
- WHEN mantle builds the compatibility manifest
- THEN the manifest is scoped to the requested output or closure transfer
- AND mantle does not have to enumerate the receiver's entire store to start
  delta planning

### Requirement: Reuse Is Attempted Coarsest-first

The delta planner MUST attempt reuse in descending granularity: whole output,
unchanged directory subtree, whole blob, then content-defined chunk.

#### Scenario: Matching output skips transfer

- GIVEN the receiver already has the requested logical output with valid
  backing content
- WHEN the sender plans the transfer
- THEN mantle reuses the whole output
- AND no finer-grained directory, blob, or chunk plan is emitted for that
  output

#### Scenario: Unchanged subtree is reused under a changed output root

- GIVEN the requested output root differs between sender and receiver
- BUT a child directory subtree within that output has the same directory digest
- WHEN the sender plans the transfer
- THEN that subtree is reused without expanding it into per-file transfer work

#### Scenario: Partial file change sends only missing chunks

- GIVEN sender and receiver share the same blob except for a changed region
- WHEN the sender reaches chunk-level planning for that blob
- THEN the transfer contains only the missing content-defined chunks
- AND unchanged chunks are reused from receiver-local storage

#### Scenario: Shared blob is reused across different outputs in one closure

- GIVEN two outputs in the same requested closure both reference the same blob
- AND the receiver already has that blob from one prior output
- WHEN the sender plans the closure transfer
- THEN the shared blob is transferred at most once
- AND both outputs may reuse it from the receiver-local store

### Requirement: Delta Planning Uses Finalized Content

Delta transfer MUST plan reuse from finalized store content only, after any
content-addressed self-reference rewriting or output-path fixups have already
been applied.

#### Scenario: Content-addressed output is compared after final rewrite

- GIVEN a content-addressed output whose finalized bytes differ from its
  provisional builder output only by self-reference rewriting
- WHEN mantle computes delta reuse for that output
- THEN chunk and blob identities are computed from the finalized persisted
  bytes
- AND provisional pre-rewrite bytes are not used for transfer planning

### Requirement: Acceptance Semantics Match Ordinary Substitution

A delta-transferred result MUST be accepted only after mantle verifies the
final signed `PathInfo` for the logical store path and records or synthesizes
the same artifact-attestation outcome required for ordinary substitution.

#### Scenario: Trusted delta transfer becomes a cache hit

- GIVEN a delta-capable substituter returns the missing content plus signed
  `PathInfo` for the target output
- AND at least one signature matches the receiver's trusted keys
- WHEN mantle verifies the completed transfer
- THEN the output is accepted as a cache hit
- AND mantle records or synthesizes the artifact attestation exactly as it
  would for a non-delta substitution

#### Scenario: Untrusted delta transfer is rejected

- GIVEN the reconstructed output bytes match the requested transfer plan
- BUT the final `PathInfo` signatures do not match any trusted key
- WHEN mantle verifies the completed transfer
- THEN the result is rejected as a cache hit
- AND mantle falls back according to substitution policy

#### Scenario: Partial transfer does not emit an attestation

- GIVEN a delta transfer has started but not all required content has arrived
- WHEN mantle has not yet verified the final signed `PathInfo`
- THEN mantle does not emit an artifact attestation for that in-progress result

### Requirement: Delta Negotiation Is Optional

Delta transfer MUST be optional and MUST fall back to full-artifact
substitution when peer capability, manifest exchange, or reuse planning does
not justify delta mode.

#### Scenario: Peer without delta support uses existing substitution path

- GIVEN a trusted substituter that only supports ordinary full-artifact fetch
- WHEN mantle asks for a cache hit
- THEN mantle uses the existing substitution path
- AND the absence of delta support is not treated as an error

### Requirement: Delta Transfer Is Streamed with Bounded Memory

The sender and receiver MUST process delta transfers in a streamed manner with
memory usage bounded by active object and chunk windows rather than total
closure size.

#### Scenario: Large closure transfers without whole-closure buffering

- GIVEN a closure large enough that full in-memory buffering would be
  impractical
- WHEN mantle performs delta transfer for that closure
- THEN mantle streams objects and missing chunks incrementally
- AND peak memory does not scale with total closure bytes alone

### Requirement: VectorCDC Chunk Profile Evaluation [r[delta-transfer.vectorcdc-evaluation]]

Mantle MUST treat VectorCDC/hashless CDC chunkers as opt-in evaluation candidates below the castore raw-content identity boundary. The FastCDC profile used by delta protocol version 1 MUST remain the default until a later evidence-backed promotion change explicitly updates negotiation semantics.

#### Scenario: Default transfer keeps FastCDC [r[delta-transfer.vectorcdc-evaluation.default-fastcdc]]

- GIVEN no experimental chunk-profile configuration is enabled
- WHEN mantle performs delta-capable planning or blob chunking
- THEN it uses the existing FastCDC profile
- AND protocol version 1 peers continue to interpret chunks as FastCDC with 128 KiB minimum, 256 KiB average, and 512 KiB maximum sizes

#### Scenario: Blob identity ignores physical chunker [r[delta-transfer.vectorcdc-evaluation.identity-boundary]]

- GIVEN two physical chunkers produce different chunk boundaries for the same finalized blob bytes
- WHEN mantle records or verifies the blob identity
- THEN the authoritative blob digest is still BLAKE3 over the raw finalized blob bytes
- AND chunking parameters do not change the logical blob identity or final PathInfo trust decision

### Requirement: Deterministic Chunker Boundary [r[delta-transfer.vectorcdc-evaluation.boundary]]

Candidate chunkers MUST expose a deterministic boundary-only interface that returns ordered, contiguous, non-overlapping chunks covering the full finalized blob byte range. Candidate output MUST be rejected before persistence or planning if the boundaries are invalid.

#### Scenario: Candidate returns valid full coverage [r[delta-transfer.vectorcdc-evaluation.boundary.valid]]

- GIVEN finalized blob bytes and a selected candidate chunker
- WHEN mantle computes chunk boundaries
- THEN every byte is covered exactly once
- AND chunks are ordered, contiguous, and non-overlapping
- AND every non-final chunk satisfies the configured profile's size bounds

#### Scenario: Invalid candidate metadata is rejected [r[delta-transfer.vectorcdc-evaluation.boundary.invalid]]

- GIVEN a candidate chunker returns a gap, overlap, out-of-order range, zero-length chunk, or out-of-bounds non-final chunk
- WHEN mantle validates the chunk list
- THEN the candidate result is rejected before upload, manifest persistence, or delta reuse planning
- AND mantle falls back according to the configured evaluation policy

### Requirement: CDC Benchmark Evidence [r[delta-transfer.vectorcdc-evaluation.evidence]]

Mantle MUST require reproducible benchmark evidence before adopting any non-default chunk profile. Evidence MUST compare the candidate against the current FastCDC baseline on representative Mantle/Nix-store-like corpora and report both algorithm-only and end-to-end storage effects where practical.

#### Scenario: Baseline and candidate metrics are comparable [r[delta-transfer.vectorcdc-evaluation.evidence.metrics]]

- GIVEN a candidate chunk profile is evaluated
- WHEN benchmark evidence is recorded
- THEN the report includes corpus provenance, CDC throughput, total ingest wall time, chunk-size distribution, chunk count, dedup/reuse ratio, object-count impact, and separated BLAKE3, compression, and object-store costs where practical
- AND the report includes the same metrics for the current FastCDC baseline

#### Scenario: Promotion requires end-to-end benefit [r[delta-transfer.vectorcdc-evaluation.promotion]]

- GIVEN candidate CDC throughput improves over FastCDC in isolation
- WHEN end-to-end ingest, substitution, or storage metrics regress enough to erase the benefit
- THEN mantle does not promote the candidate as a default or negotiated profile
- AND the adoption decision records whether to keep FastCDC, tune FastCDC, continue research, or open a narrower promotion OpenSpec

### Requirement: Optional Experimental Dependency Boundary [r[delta-transfer.vectorcdc-evaluation.optional-deps]]

Experimental VectorCDC/SIMD dependencies MUST remain outside default builds and default protocol negotiation until promoted. Unsupported platforms MUST either use a verified scalar fallback for the same candidate semantics or fail closed before advertising the candidate profile.

#### Scenario: Default build excludes experimental SIMD dependency [r[delta-transfer.vectorcdc-evaluation.optional-deps.default]]

- GIVEN a default Mantle build with no experimental feature enabled
- WHEN dependencies and exposed chunk profiles are inspected
- THEN VectorCDC/SIMD candidate dependencies are absent from the default dependency graph
- AND only the existing supported chunk profiles are advertised

#### Scenario: Unsupported platform does not advertise candidate [r[delta-transfer.vectorcdc-evaluation.optional-deps.unsupported]]

- GIVEN a candidate requires SIMD support that is unavailable on the current platform
- AND no verified scalar fallback is enabled
- WHEN mantle constructs its chunk-profile capability manifest
- THEN the candidate profile is not advertised
- AND attempts to select it fail closed before transfer planning or blob metadata persistence
