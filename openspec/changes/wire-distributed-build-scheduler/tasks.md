## Phase 1: Implementation

- [x] [serial] Trace the current `Worker`/orchestration insertion points for realization key and resolver dispatch. ✅ 1m (started: 2026-05-12T03:32:08Z → completed: 2026-05-12T03:32:40Z)
  Evidence: added `scheduler-wiring-plan.md` trace showing `Builder::build_all -> Worker::want/run -> dispatch_ready -> Builder::prepare_build -> finish_build`; selected the first safe resolver boundary after `derivation_to_build_request(...)` and before sandbox dispatch.
- [x] [serial] Design the default-local wiring shape and diagnostic events without provider lock-in. ✅ 1m (started: 2026-05-12T03:33:03Z → completed: 2026-05-12T03:33:44Z)
  Evidence: `scheduler-wiring-plan.md` now defines `DistributedSchedulerConfig` defaulting to `LocalOnly`, keeps adapters on `Builder`, preserves `Worker` drv-path dedup/max-jobs behavior, and maps opt-in resolver outcomes to existing `DistributedDiagnostic` receipts.
- [ ] [serial] Add implementation tasks for a later drain and validate the OpenSpec change.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate wire-distributed-build-scheduler --strict` and record evidence before archive.
