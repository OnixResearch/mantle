## Implementation

- [x] [serial] r[remote_builds.durable_attempt_fencing] Inventory current job, lease, retry, authorization, resume, log, transfer, and output-admission mutations; document which mutations currently lack attempt/fence identity.
- [x] [serial] r[remote_builds.durable_attempt_fencing] Add typed job, attempt, fence-generation, and event identities plus a bounded attempt-state model that preserves the existing normalized realization key.
- [x] [depends:attempt-identities] r[remote_builds.pure_attempt_decisions] Extract pure authorization, retry, transition, idempotency, and fence-validation functions over immutable input facts.
- [x] [depends:pure-attempt-decisions] r[remote_builds.durable_attempt_fencing] Make the coordinator shell persist a new attempt and advanced fence before exposing assignment, and migrate legacy durable state fail-closed.
- [x] [depends:durable-attempt-state] r[remote_builds.idempotent_attempt_reporting] Require job, attempt, fence, event id, and canonical payload digest on worker state changes, log appends, transfer checkpoints, result reports, and completion.
- [x] [depends:idempotent-attempt-reporting] r[remote_builds.durable_attempt_fencing] Reject stale/superseded reports before state mutation or output admission and expose bounded stable reason codes in status and build reports.
- [x] [depends:durable-attempt-state] r[remote_builds.pure_attempt_decisions] Thread named retry budgets and explicit supplied time facts through typed Nickel remote-farm policy without introducing provider-specific scheduler fields.

## Verification

- [x] [depends:pure-attempt-decisions] r[remote_builds.pure_attempt_decisions] Add table, property, and Kani tests proving equivalent facts yield equivalent decisions, fence generations never decrease, and terminal states cannot transition back to live states.
- [x] [depends:idempotent-attempt-reporting] r[remote_builds.idempotent_attempt_reporting] Positive: repeat a byte-identical current-attempt report and prove the second application is an idempotent no-op with the same durable state.
- [x] [depends:idempotent-attempt-reporting] r[remote_builds.idempotent_attempt_reporting] Negative: reuse an event id with a different digest and prove the coordinator rejects it without changing job, log, transfer, or output state.
- [x] [depends:durable-attempt-state] r[remote_builds.durable_attempt_fencing] Positive: expire or reassign an attempt, restart the coordinator, complete the new attempt, and prove only the advanced fence can publish current state.
- [x] [depends:durable-attempt-state] r[remote_builds.durable_attempt_fencing] Negative: deliver late start, log, transfer, failure, and completion reports from the superseded fence and prove every mutation is rejected before output admission.
- [x] [depends:attempt-verification] r[remote_builds.durable_attempt_fencing] Run focused remote-build/coordinator tests, remote configuration checks, Cairn validate, and proposal/design/tasks gates; capture the pre-existing policy blocker separately if it remains.

## Evidence summary

- Mutation inventory: `evidence/mutation-inventory.md`.
- Baseline at `f2e43a9fa396683d624f3d9e6df6586062a522d3`: coordinator 9 passed, remote config 9 passed, distributed core 41 passed (`pueue` task 1352).
- Final focused tests: pure attempt core 13 passed (task 1356), coordinator 20 passed (task 1357), typed remote config 12 passed (task 1359), existing distributed core 41 passed (task 1360), and `cargo check -p mantle --bin mantle` passed (task 1363).
- Three `#[kani::proof]` harnesses cover deterministic equivalent decisions, monotonic fences, and terminal-state closure. `cargo-kani` is unavailable on this host (task 973), so Kani execution is not claimed.
- Final Cairn validation reported `valid: true` with no policy/spec/change issues. Proposal, design, and post-completion tasks gates each reported `PASS` with no issues; exact receipts are recorded in `evidence/validation-2026-07-12.md`.
- The separate repository-wide Tracey profile remains blocked by pre-existing release-provenance debt: `valid: false`, 18 of 73 referenced, 55 missing, zero dangling; the next missing group is `mantle.release_provenance.cairn_evidence_handoff.*` (task 1422).
