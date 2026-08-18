## Why

Remote execution is currently selected through special build flags instead of the normal route planner. Operators need to know before mutation whether a root is cached, substitutable, archive-backed, remotely buildable, locally buildable, or blocked by trust/input/capability gaps.

## What Changes

- Add remote-builder candidates to deterministic build realization planning.
- Make `mantle build --plan` evaluate remote-builder eligibility without opening remote sessions or uploading bytes.
- Report capability matches, source/input readiness, upload-budget fit, output-trust basis, network policy, and rejected-route reason codes.
- Keep route ranking deterministic and independent of remote discovery order or first response latency.

## Impact

- **Files**: route planner core, build-plan CLI, remote-builder fact collection, JSON/human route reports, docs, and Cairn realization-routing spec delta.
- **Testing**: positive remote-eligible plan fixture; negative no-output-trust, missing-input, capability-mismatch, offline-mode, and no-remote-eval fixtures; Cairn validate and gates.

## Out of Scope

- Executing remote builds.
- Adding production P2P transport.
- Treating builder resource tickets as output trust.
