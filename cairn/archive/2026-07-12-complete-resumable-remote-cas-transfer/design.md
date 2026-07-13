## Context

Mantle already owns blob/directory castore services, signed PathInfo, NAR rendering/import, source-bundle state, delta negotiation, output admission, transfer capability labels, quotas, and remote session leases. Production remote transfer still serializes complete artifacts into frame payload vectors. The archived farm evidence accurately records that gap even though its task file was checked complete.

## Decisions

### 1. Reuse castore and delta identities

**Choice:** The transfer manifest references existing castore blob/directory identities, PathInfo and attestation identities, source-bundle identities, NAR descriptors, and delta artifacts. No second object store or Rio-specific object model is introduced.

**Rationale:** Mantle already has the authoritative content and trust boundaries. A parallel CAS would create duplicate identity, GC, and admission logic.

### 2. Make the receiver drive demand

**Choice:** A canonical sorted manifest describes the candidate closure. The receiver validates bounds and policy, probes local content, and returns a deterministic missing-object demand set. The sender may transmit only demanded objects and declared metadata.

**Rationale:** Receiver-driven demand enables dedupe, content-presence cutoff, privacy review, and predictable quotas without trusting sender locality claims.

### 3. Separate control frames from data chunks

**Choice:** Bounded control frames carry negotiation, demand, acknowledgements, checkpoints, and completion. Artifact bytes flow as bounded chunks through a streaming adapter with receiver-issued byte/chunk credits.

**Rationale:** Control-frame limits should remain small and auditable; large objects must not require whole-memory serialization.

### 4. Resume by verified state, not cursor trust

**Choice:** A resume checkpoint binds transfer-session id, manifest BLAKE3, job/attempt/fence, policy digest, acknowledged-object summary, sequence position, and byte counters. On reconnect the receiver revalidates the checkpoint and local object presence, then recomputes remaining demand. A cursor is an optimization, never proof that bytes are safely present.

**Rationale:** Recomputing from verified receiver state prevents stale or forged cursors from skipping required data.

### 5. Scope remote resume to the current fence

**Choice:** Remote transfer checkpoints are accepted only for the current durable attempt and fence. Reassignment invalidates the old transfer session; verified objects already committed to castore may still be reused by a new session through ordinary missing-object negotiation.

**Rationale:** Content can survive retries, but stale workers must not retain authority to mutate current transfer or result state.

### 6. Apply backpressure before allocation

**Choice:** Typed policy supplies named limits for chunk bytes, in-flight credits, buffered chunks, total bytes, object count, idle progress, and checkpoint size. The receiver grants credit only after policy and storage capacity checks. Limit failures occur before allocating or buffering the rejected payload.

**Rationale:** Quotas without allocation ordering do not protect memory or disk pressure.

### 7. Define content-addressed early cutoff narrowly

**Choice:** Transfer ends with `already-present` or `demand-satisfied` only when the receiver verifies that the requested content identity and required closure metadata are already complete, or every demanded object has been acknowledged. An expected CA output path, sender claim, partial prefix, or unadmitted PathInfo is insufficient.

**Rationale:** This captures Rio's useful early-cutoff optimization without pretending a not-yet-built CA output is known or trusted.

### 8. Keep transfer and output admission separate

**Choice:** Transfer completion proves only that declared bytes/objects arrived and matched transport identities. Signed PathInfo, store prefix, requested output, artifact attestation, producer policy, and claim-strength checks still run through ordinary output admission.

**Rationale:** Efficient transport is not output correctness or trust.

### 9. Repair evidence with a superseding record

**Choice:** Preserve the archived farm package as historical evidence. Add a current tracked reconciliation note that cites the checked task, contradictory session evidence, inline implementation paths, and the new passing or blocked validation state.

**Rationale:** Rewriting the archive would hide the proof-before-claim failure rather than correct it.

## Functional Core / Imperative Shell

- **Core**: canonical manifest validation, missing-set derivation, chunk/checkpoint validation, credit decisions, resume planning, quota arithmetic, completion classification, early-cutoff decision, and stable reason codes.
- **Shell**: castore probes and writes, NAR/delta streaming, socket/stdio transport, durable checkpoint persistence, lease/GC roots, clocks/timeouts, output admission, and report rendering.

## Risks / Trade-offs

- Directory closure reconstruction must use existing castore semantics; incomplete directory metadata cannot be papered over by a cursor.
- Checkpoint durability adds write amplification, so checkpoint cadence must be policy-driven and bounded.
- Old peers need explicit capability downgrade to full bounded transfer or rejection; they must not be mislabeled streaming.
- Reusable verified objects may outlive a failed session and need ordinary castore GC accounting.
