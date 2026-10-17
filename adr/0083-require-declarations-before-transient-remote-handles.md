# ADR 0083: Require declarations before transient remote handles

## Status

Proposed

## Context

Remote transfer and build messages carry session, artifact, attempt, and output
identities. A transient message has no lifetime of its own. Allowing one to
introduce a new identity would let a chunk, acknowledgement, checkpoint, or
attempt claim authority that no preceding declaration established. ADR 0080
places this enforcement in the building plane, independent of a daemon.

## Decision Drivers

- Reject an unknown identity before payload or authority work.
- Preserve existing framing, digest checks, fencing, and output admission.
- Make the declaration for each consumed identity inspectable and testable.
- Do not turn checkpoint progress into proof of receiver content.

## Decision

A transient remote message MUST NOT introduce an unknown handle, session,
lease, or binding. A lifetime-bearing declaration MUST establish it first.
The building plane checks the current declaration when a message consumes that
identity; it fails closed locally without a new protocol-level rejection
message. The reviewed protocol offers no reliable distinction between a
rejected message and a dead peer, so feedback is not an authority mechanism.

| Consumed identity | Establishing declaration | Checked boundary |
| --- | --- | --- |
| Chunk artifact and session | Canonical transfer manifest plus receiver demand and credit | Chunk header scope and exact demanded occurrence before payload allocation |
| Transfer acknowledgement | Manifest-scoped in-flight reservation | Acknowledgement scope, sequence, observed digest, and byte count |
| Resume checkpoint | Manifest scope and receiver-owned verified bytes under a bounded lease | Checkpoint scope, digest, counters, and reprobed content |
| Resource-scoped attempt | Coordinator assignment and matching active resource lease | Attempt report against the assigned attempt and lease |
| Production session binding | Coordinator job assignment naming the current worker, attempt, and fence | Resolve the assigned binding; reject unknown jobs or unassigned workers before dispatch |
| Loopback session | Concrete client request deriving the session identity | Match the established request before ticket redemption |
| Output admission | Signed PathInfo, requested output, prefix, content, and attestation | Existing store import and trust policy, never a transfer completion or acknowledgement |

The fixture map and commands are in [`docs/remote-transfer.md`](../docs/remote-transfer.md).
Keep the checks at these existing boundaries rather than introducing a central
handle registry or changing wire frames. A new consumed handle needs its own
explicit declaration and both established/unknown fixtures.

## Consequences

An unknown handle fails before the corresponding payload or authority work.
Acknowledgement and checkpoint progress remain useful for resume but cannot
establish content or output trust. This rule proves protocol shape for the
checked paths only; it does not prove delivery, completeness, crash consistency,
worker honesty, digest correctness, or release eligibility.
