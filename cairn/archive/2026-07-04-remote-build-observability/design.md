## Context

Distributed builds add more failure phases than local builds: route planning, ticket/access authorization, worker matching, input sync, remote execution, output transfer, and output trust. Operators need bounded reports that explain those phases without treating untrusted logs or secrets as control data.

## Decisions

### 1. Status is a redacted snapshot

**Choice:** Remote status reports include endpoint id, worker counts, capabilities, queue phases, active jobs, recent failures, log cursors, ticket metadata, and configured limits, while redacting bearer tokens, private key paths, raw environment values, full argv, and uploaded content.

**Rationale:** Status should be useful to operators and safe to paste into evidence.

### 2. Build reports carry route and transfer evidence

**Choice:** Build JSON includes selected route, rejected-route reason codes, remote endpoint, upload summary, transfer mode, transferred/reused bytes, fallback reason, signer/trust basis, artifact attestation references, and non-claims.

**Rationale:** Automation should not scrape human logs to know whether a remote result was accepted and why.

### 3. Logs are bounded and replayable

**Choice:** Remote log replay uses cursors, byte caps, chunk caps, and phase labels. Slow subscribers are truncated or failed according to policy rather than causing unbounded buffering.

**Rationale:** Long remote builds must remain observable without becoming a memory or secret-retention hazard.

## Risks / Trade-offs

- Excessive redaction can make debugging hard; reports need stable opaque ids and short reason codes.
- Existing tests may assume older JSON shape and need schema-compatible extension assertions.
