# Store Transports Specification

## Purpose

Define resumable receiver-driven transfer over Mantle's existing content-addressed store identities.

## Requirements

### Requirement: Castore transfer sessions are resumable from verified receiver state [r[store_transports.resumable_castore_sessions]]

Mantle MUST support versioned transfer sessions whose canonical manifests bind the requested content identities, artifact classes, logical store prefix, required metadata, named limits, and BLAKE3 manifest identity. Resume planning MUST validate the checkpoint and recompute remaining demand from verified receiver state rather than trusting a cursor alone.

#### Scenario: Interrupted transfer resumes missing content only

- GIVEN a transfer session committed and acknowledged a subset of demanded castore objects before interruption
- WHEN the receiver validates a matching checkpoint and reprobes local object completeness
- THEN Mantle MUST derive the remaining demand deterministically and request only missing or incomplete content
- AND already verified complete objects MUST NOT be retransmitted merely because transport reconnected.

#### Scenario: Stale or tampered checkpoint fails closed

- GIVEN a checkpoint has the wrong manifest digest, policy digest, transfer identity, regressed acknowledgement state, impossible byte counters, malformed bounds, or missing receiver objects
- WHEN Mantle plans resume
- THEN it MUST reject or safely recompute from verified receiver state with a stable reason code
- AND it MUST NOT skip required bytes or declare transfer complete from the checkpoint alone.

### Requirement: Streaming transfer is receiver-driven and backpressured [r[store_transports.receiver_driven_backpressure]]

Mantle MUST separate bounded control messages from bounded artifact chunks and MUST let the receiver grant explicit byte/chunk credit before payload transmission. Named policy MUST bound chunk size, in-flight credit, buffered chunks, object count, total bytes, checkpoint size, and progress limits, and rejection MUST occur before allocating or buffering disallowed payload.

#### Scenario: Credit bounds in-flight data

- GIVEN a sender has more demanded content than the receiver's current credit
- WHEN the sender streams chunks
- THEN it MUST stop transmitting payload when credit is exhausted until acknowledgement grants more credit
- AND receiver memory or disk staging MUST remain within configured named bounds.

#### Scenario: Quota overflow is rejected before persistence

- GIVEN a chunk, object, manifest, checkpoint, or cumulative transfer would exceed configured policy
- WHEN the receiver validates the next operation
- THEN Mantle MUST reject it before disallowed allocation or persistence
- AND it MUST preserve a deterministic transfer-phase diagnostic without importing partial output as successful.

### Requirement: Verified content presence permits early transfer cutoff [r[store_transports.content_presence_early_cutoff]]

Mantle SHOULD stop transfer work when verified receiver state already contains the complete requested content identity and required closure metadata, or when every demanded object has been acknowledged. An expected content-addressed path, sender assertion, partial digest prefix, incomplete directory tree, or unadmitted PathInfo MUST NOT trigger successful cutoff.

#### Scenario: Complete receiver content avoids payload transfer

- GIVEN the receiver verifies the requested root identity, complete object closure, and required metadata before granting payload credit
- WHEN transfer demand is planned
- THEN Mantle SHOULD return an `already-present` completion disposition without requesting artifact payload bytes
- AND later output reuse or import MUST still pass its ordinary trust and admission policy.

#### Scenario: Partial content cannot fabricate completion

- GIVEN the receiver has only a root node, partial directory closure, stale PathInfo, mismatched attestation, or unverified sender claim
- WHEN early-cutoff eligibility is evaluated
- THEN Mantle MUST continue missing-content negotiation or reject the transfer
- AND it MUST NOT report `already-present` or successful output admission.
