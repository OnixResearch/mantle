# Remote Builds Specification

## Purpose

Bind resumable store transfer to the current fenced remote-build attempt and ordinary output admission.

## Requirements

### Requirement: Remote transfer resume is scoped to the current attempt [r[remote_builds.attempt_scoped_transfer_resume]]

Mantle MUST bind each remote input/output transfer session and checkpoint to the durable job, current attempt id, current fence generation, canonical transfer manifest, and transfer policy. Reassignment MUST invalidate the old session's authority while allowing a new current session to reuse independently verified content through ordinary missing-object negotiation.

#### Scenario: Current attempt resumes safely

- GIVEN a current fenced attempt has a valid interrupted transfer checkpoint and verified receiver objects
- WHEN the same attempt reconnects under matching manifest and policy
- THEN Mantle MAY resume by requesting the deterministically remaining content
- AND completion MUST still pass signed PathInfo, requested identity, store-prefix, object, attestation, and claim-strength admission.

#### Scenario: Superseded attempt cannot resume

- GIVEN a transfer checkpoint names an attempt or fence superseded by reassignment
- WHEN the worker reconnects or submits another chunk, acknowledgement, or completion
- THEN Mantle MUST reject that mutation before changing current transfer or result state
- AND verified castore objects from the older session MAY be reused only after the current receiver reprobes them by content identity.

#### Scenario: Fallback remains claim safe

- GIVEN delta or resumable streaming cannot continue safely and policy allows full-NAR fallback
- WHEN Mantle uses the fallback
- THEN it MUST verify the full transfer and ordinary output-admission evidence before success
- AND the report MUST identify the actual transfer mode, bytes, and fallback reason instead of claiming streaming or delta completion.
