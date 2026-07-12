## Context

`RemoteCoordinatorJobSummary` currently binds a durable job id, normalized build key, assigned worker, phase, output claims, log cursors, and result availability. The coordinator can persist and adopt that state, but logical job identity alone cannot distinguish the current assignment from a superseded worker that reconnects late. Resource authorization and output trust are already separate and must remain so.

## Decisions

### 1. Use a three-level identity model

**Choice:** Preserve the existing normalized realization key for dedupe, retain a durable job id for client attachment, and add a unique attempt id plus monotonic fence generation for each assignment.

**Rationale:** A retry is the same logical request but a different owner. Collapsing those identities makes stale reports indistinguishable from current reports.

### 2. Fence every state-changing worker report

**Choice:** Assignment, start, heartbeat, log append, transfer checkpoint, result-ready, failure, and completion messages carry job id, attempt id, and fence generation. The coordinator accepts mutation only when all three match the current durable assignment.

**Rationale:** Fencing only final completion leaves stale workers able to corrupt intermediate logs, transfer state, or liveness decisions.

### 3. Make event application idempotent

**Choice:** Each attempt report carries a stable event id and canonical payload digest. Repeating the same event id and digest returns an `already-applied` decision; reusing an event id with different content fails as a conflict; reports from a lower or unknown fence fail as stale.

**Rationale:** At-least-once transport delivery is normal. Idempotent state application is safer than trying to make transport exactly once.

### 4. Keep authorization, retry, and transition logic pure

**Choice:** Pure functions consume explicit authorization facts, current attempt state, failure class, retry budget, policy, and supplied time facts, then return decisions and next state. They do not read clocks, files, environment, sockets, or process state.

**Rationale:** The safety boundary should be testable with assertions, property tests, and Kani without standing up the coordinator runtime.

### 5. Persist intent before exposing assignment

**Choice:** The coordinator shell durably records the new attempt and fence before returning an assignment. Completion is durably recorded before redelivery is advertised. The storage adapter may use the existing atomic state file initially, but the core transition contract remains independent of the persistence backend.

**Rationale:** A crash between publication and persistence must not create two apparently current owners or fabricate a retained result.

### 6. Accept one current attempt, not exactly one execution

**Choice:** A superseded worker may continue consuming resources until cancellation reaches it, but its reports and outputs cannot mutate current state or pass output admission. Status and evidence distinguish `superseded`, `stale-report-rejected`, and `current-attempt-completed`.

**Rationale:** Exactly-once execution is not realistic under partitions. Fenced admission gives the property Mantle actually needs.

## Functional Core / Imperative Shell

- **Core**: attempt identity validation, state transition, authorization decision, retry decision, idempotency classification, fence comparison, and redacted reason codes.
- **Shell**: clock reads, durable load/save, worker transport, cancellation, log/object writes, cryptographic output admission, and report rendering.

## Risks / Trade-offs

- Existing persisted coordinator state needs an explicit migration that treats legacy live jobs conservatively rather than inventing a current fence.
- A monotonic fence is coordinator-scoped; multi-coordinator consensus remains out of scope.
- Cancellation latency can waste work, but stale output rejection preserves correctness.
- Event-id retention must be bounded without allowing an old conflicting event to become current again.
