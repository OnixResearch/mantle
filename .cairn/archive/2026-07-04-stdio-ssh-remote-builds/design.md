## Context

The remote-build protocol already has bounded frame semantics and stdio fixture coverage. The production gap is not the core frame model; it is the operator binding: child process launch, SSH-stdio command construction, stdout isolation, stderr/log limits, deadlines, and evidence that the same state machine runs over both bindings.

## Decisions

### 1. Stdio is the baseline production transport

**Choice:** Support a local child `stdio` binding and an `ssh-stdio` binding that both carry the same length-prefixed remote-build frames.

**Rationale:** SSH-stdio covers common remote-builder deployments without requiring a daemon listener or peer discovery.

### 2. Stdout is protocol-only

**Choice:** Any unframed stdout byte is terminal protocol corruption. Human diagnostics, logs, and tracing must go to stderr or framed log messages with configured byte limits.

**Rationale:** Mixed stdout cannot be parsed safely or replayed deterministically.

### 3. Handshake precedes expensive work

**Choice:** The server binding completes hello, authorization, capability reporting, and request shape validation before store scans, input walks, or sandbox preparation.

**Rationale:** Invalid clients should be rejected cheaply and without exposing host resource facts unnecessarily.

## Risks / Trade-offs

- SSH process management must avoid shell-quoting footguns by passing explicit argv where possible.
- Some remote hosts may have noisy shell startup files; protocol-only stdout will reject those hosts until configured correctly.
