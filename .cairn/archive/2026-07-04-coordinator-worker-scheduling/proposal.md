## Why

Single-builder dispatch is useful, but distributed capacity needs worker-dialed registration, capability queues, duplicate request suppression, resumable result delivery, and durable queue state. The coordinator should schedule resources without becoming an output-trust root.

## What Changes

- Add a coordinator runtime that records worker registrations, capabilities, concurrency, output signing-key identities, queue state, and resumable job summaries.
- Match concrete build requests by normalized build key, system, sandbox/network mode, feature labels, resource limits, logical store prefix, upload feasibility, and output-trust preflight.
- Deduplicate identical in-flight requests and reject conflicting live output claims.
- Preserve phase-classified job/log/result state across coordinator or worker restarts when retained state matches the normalized request.

## Impact

- **Files**: coordinator state core, worker registration shell, queue matching, durable state, status reporting, tests, docs, and Cairn remote-builds spec delta.
- **Testing**: positive worker registration/dispatch, duplicate attach, resume redelivery; negative capability mismatch, output-trust blocker, conflicting live output claim, exhausted resource limit, and stale resume state.

## Out of Scope

- Making the coordinator an output-signing authority.
- P2P discovery beyond worker/coordinator sessions.
- Replacing signed output admission with queue admission.
