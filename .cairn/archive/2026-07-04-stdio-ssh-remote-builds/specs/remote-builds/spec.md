## ADDED Requirements

### Requirement: Stdio and SSH-stdio are hardened remote-build bindings

r[remote_builds.stdio_ssh_hardened_bindings] Mantle MUST support stdio and SSH-stdio as operator-facing remote-build bindings that carry the same versioned remote-build frame protocol and enforce the same authentication, endpoint identity, message limits, concrete-request validation, input-sync checks, and output-trust admission as other remote transports. Stdout MUST be reserved for protocol frames, while logs and diagnostics MUST be bounded and separated from control data.

#### Scenario: SSH-stdio preserves protocol semantics

GIVEN a remote builder is launched over SSH-stdio with a configured endpoint identity and ticket
WHEN the client performs hello, authorization, input sync, build execution, and output transfer
THEN Mantle MUST drive the same state transitions as the stdio binding
AND accepted outputs MUST pass the same signed output-admission checks before import.

#### Scenario: human stdout corrupts the protocol

GIVEN a stdio or SSH-stdio builder writes unframed human text, logs, tracing, or errors to stdout
WHEN the client decodes the remote-build stream
THEN Mantle MUST fail the session as terminal protocol corruption
AND it MUST NOT enqueue a build or import outputs from that corrupted stream.

#### Scenario: handshake is cheap and bounded

GIVEN a remote builder has expensive store scans, input walks, or sandbox setup available after connection
WHEN a stdio or SSH-stdio session starts
THEN Mantle MUST complete hello, authorization, capability reporting, and request shape validation before expensive work starts
AND timeout, oversized-frame, child-exit, and invalid-sequence failures MUST include phase-classified diagnostics.
