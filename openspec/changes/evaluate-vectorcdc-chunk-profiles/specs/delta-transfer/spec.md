## ADDED Requirements

### Requirement: VectorCDC Chunk Profile Evaluation [r[delta-transfer.vectorcdc-evaluation]]

Crunch MUST treat VectorCDC/hashless CDC chunkers as opt-in evaluation candidates below the castore raw-content identity boundary. The FastCDC profile used by delta protocol version 1 MUST remain the default until a later evidence-backed promotion change explicitly updates negotiation semantics.

#### Scenario: Default transfer keeps FastCDC [r[delta-transfer.vectorcdc-evaluation.default-fastcdc]]

- GIVEN no experimental chunk-profile configuration is enabled
- WHEN crunch performs delta-capable planning or blob chunking
- THEN it uses the existing FastCDC profile
- AND protocol version 1 peers continue to interpret chunks as FastCDC with 128 KiB minimum, 256 KiB average, and 512 KiB maximum sizes

#### Scenario: Blob identity ignores physical chunker [r[delta-transfer.vectorcdc-evaluation.identity-boundary]]

- GIVEN two physical chunkers produce different chunk boundaries for the same finalized blob bytes
- WHEN crunch records or verifies the blob identity
- THEN the authoritative blob digest is still BLAKE3 over the raw finalized blob bytes
- AND chunking parameters do not change the logical blob identity or final PathInfo trust decision

### Requirement: Deterministic Chunker Boundary [r[delta-transfer.vectorcdc-evaluation.boundary]]

Candidate chunkers MUST expose a deterministic boundary-only interface that returns ordered, contiguous, non-overlapping chunks covering the full finalized blob byte range. Candidate output MUST be rejected before persistence or planning if the boundaries are invalid.

#### Scenario: Candidate returns valid full coverage [r[delta-transfer.vectorcdc-evaluation.boundary.valid]]

- GIVEN finalized blob bytes and a selected candidate chunker
- WHEN crunch computes chunk boundaries
- THEN every byte is covered exactly once
- AND chunks are ordered, contiguous, and non-overlapping
- AND every non-final chunk satisfies the configured profile's size bounds

#### Scenario: Invalid candidate metadata is rejected [r[delta-transfer.vectorcdc-evaluation.boundary.invalid]]

- GIVEN a candidate chunker returns a gap, overlap, out-of-order range, zero-length chunk, or out-of-bounds non-final chunk
- WHEN crunch validates the chunk list
- THEN the candidate result is rejected before upload, manifest persistence, or delta reuse planning
- AND crunch falls back according to the configured evaluation policy

### Requirement: CDC Benchmark Evidence [r[delta-transfer.vectorcdc-evaluation.evidence]]

Crunch MUST require reproducible benchmark evidence before adopting any non-default chunk profile. Evidence MUST compare the candidate against the current FastCDC baseline on representative Crunch/Nix-store-like corpora and report both algorithm-only and end-to-end storage effects where practical.

#### Scenario: Baseline and candidate metrics are comparable [r[delta-transfer.vectorcdc-evaluation.evidence.metrics]]

- GIVEN a candidate chunk profile is evaluated
- WHEN benchmark evidence is recorded
- THEN the report includes corpus provenance, CDC throughput, total ingest wall time, chunk-size distribution, chunk count, dedup/reuse ratio, object-count impact, and separated BLAKE3, compression, and object-store costs where practical
- AND the report includes the same metrics for the current FastCDC baseline

#### Scenario: Promotion requires end-to-end benefit [r[delta-transfer.vectorcdc-evaluation.promotion]]

- GIVEN candidate CDC throughput improves over FastCDC in isolation
- WHEN end-to-end ingest, substitution, or storage metrics regress enough to erase the benefit
- THEN crunch does not promote the candidate as a default or negotiated profile
- AND the adoption decision records whether to keep FastCDC, tune FastCDC, continue research, or open a narrower promotion OpenSpec

### Requirement: Optional Experimental Dependency Boundary [r[delta-transfer.vectorcdc-evaluation.optional-deps]]

Experimental VectorCDC/SIMD dependencies MUST remain outside default builds and default protocol negotiation until promoted. Unsupported platforms MUST either use a verified scalar fallback for the same candidate semantics or fail closed before advertising the candidate profile.

#### Scenario: Default build excludes experimental SIMD dependency [r[delta-transfer.vectorcdc-evaluation.optional-deps.default]]

- GIVEN a default Crunch build with no experimental feature enabled
- WHEN dependencies and exposed chunk profiles are inspected
- THEN VectorCDC/SIMD candidate dependencies are absent from the default dependency graph
- AND only the existing supported chunk profiles are advertised

#### Scenario: Unsupported platform does not advertise candidate [r[delta-transfer.vectorcdc-evaluation.optional-deps.unsupported]]

- GIVEN a candidate requires SIMD support that is unavailable on the current platform
- AND no verified scalar fallback is enabled
- WHEN crunch constructs its chunk-profile capability manifest
- THEN the candidate profile is not advertised
- AND attempts to select it fail closed before transfer planning or blob metadata persistence
