# Specification: Transient-handle admission

## ADDED Requirements

### Requirement: Transient messages cannot introduce handles

r[remote_builds.transient_handle_introduction] A transient Mantle protocol
message MUST NOT introduce an unknown handle, session, lease, or binding. A
lifetime-bearing declaration MUST establish the handle first.

Every protocol boundary that consumes a handle MUST name the declaration that
introduces it. The rule MUST be documented with the boundary table and MUST NOT
change the wire format.

#### Scenario: Established handle is accepted

- GIVEN a boundary whose declaration has established a handle
- WHEN a later message uses that handle
- THEN the message MUST be admitted under the existing rules

#### Scenario: Unknown handle fails closed

- GIVEN a message that uses a handle with no established declaration
- WHEN the boundary processes the message
- THEN it MUST fail closed before payload or authority work

### Requirement: Checkpoints and acknowledgements are never authority

r[remote_builds.transient_handle_rejection] A checkpoint, cursor, or
acknowledgement MUST NOT be accepted as content, admission, or authority.
Output admission MUST continue to require signed PathInfo, content, requested
output, prefix, and attestation facts.

#### Scenario: Checkpoint is not content

- GIVEN a durable checkpoint for a transfer session
- WHEN a consumer treats it as proof of content
- THEN the check MUST fail

#### Scenario: Attempt requires a lease

- GIVEN an attempt message for a remote job
- WHEN no lease has established the attempt
- THEN the receiver MUST reject the attempt
