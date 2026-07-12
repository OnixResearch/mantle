## Implementation

- [ ] [serial] r[remote_builds.durable_attempt_fencing] Inventory current job, lease, retry, authorization, resume, log, transfer, and output-admission mutations; document which mutations currently lack attempt/fence identity.
- [ ] [serial] r[remote_builds.durable_attempt_fencing] Add typed job, attempt, fence-generation, and event identities plus a bounded attempt-state model that preserves the existing normalized realization key.
- [ ] [depends:attempt-identities] r[remote_builds.pure_attempt_decisions] Extract pure authorization, retry, transition, idempotency, and fence-validation functions over immutable input facts.
- [ ] [depends:pure-attempt-decisions] r[remote_builds.durable_attempt_fencing] Make the coordinator shell persist a new attempt and advanced fence before exposing assignment, and migrate legacy durable state fail-closed.
- [ ] [depends:durable-attempt-state] r[remote_builds.idempotent_attempt_reporting] Require job, attempt, fence, event id, and canonical payload digest on worker state changes, log appends, transfer checkpoints, result reports, and completion.
- [ ] [depends:idempotent-attempt-reporting] r[remote_builds.durable_attempt_fencing] Reject stale/superseded reports before state mutation or output admission and expose bounded stable reason codes in status and build reports.
- [ ] [depends:durable-attempt-state] r[remote_builds.pure_attempt_decisions] Thread named retry budgets and explicit supplied time facts through typed Nickel remote-farm policy without introducing provider-specific scheduler fields.

## Verification

- [ ] [depends:pure-attempt-decisions] r[remote_builds.pure_attempt_decisions] Add table, property, and Kani tests proving equivalent facts yield equivalent decisions, fence generations never decrease, and terminal states cannot transition back to live states.
- [ ] [depends:idempotent-attempt-reporting] r[remote_builds.idempotent_attempt_reporting] Positive: repeat a byte-identical current-attempt report and prove the second application is an idempotent no-op with the same durable state.
- [ ] [depends:idempotent-attempt-reporting] r[remote_builds.idempotent_attempt_reporting] Negative: reuse an event id with a different digest and prove the coordinator rejects it without changing job, log, transfer, or output state.
- [ ] [depends:durable-attempt-state] r[remote_builds.durable_attempt_fencing] Positive: expire or reassign an attempt, restart the coordinator, complete the new attempt, and prove only the advanced fence can publish current state.
- [ ] [depends:durable-attempt-state] r[remote_builds.durable_attempt_fencing] Negative: deliver late start, log, transfer, failure, and completion reports from the superseded fence and prove every mutation is rejected before output admission.
- [ ] [depends:attempt-verification] r[remote_builds.durable_attempt_fencing] Run focused remote-build/coordinator tests, remote configuration checks, Cairn validate, and proposal/design/tasks gates; capture the pre-existing policy blocker separately if it remains.
