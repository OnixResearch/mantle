## Phase 1: Implementation

- [x] [serial] Trace the current `Worker`/orchestration insertion points for realization key and resolver dispatch. ✅ 1m (started: 2026-05-12T03:32:08Z → completed: 2026-05-12T03:32:40Z)
  Evidence: added `scheduler-wiring-plan.md` trace showing `Builder::build_all -> Worker::want/run -> dispatch_ready -> Builder::prepare_build -> finish_build`; selected the first safe resolver boundary after `derivation_to_build_request(...)` and before sandbox dispatch.
- [x] [serial] Design the default-local wiring shape and diagnostic events without provider lock-in. ✅ 1m (started: 2026-05-12T03:33:03Z → completed: 2026-05-12T03:33:44Z)
  Evidence: `scheduler-wiring-plan.md` now defines `DistributedSchedulerConfig` defaulting to `LocalOnly`, keeps adapters on `Builder`, preserves `Worker` drv-path dedup/max-jobs behavior, and maps opt-in resolver outcomes to existing `DistributedDiagnostic` receipts.
- [x] [serial] Add implementation tasks for a later drain and validate the OpenSpec change. ✅ 1m (started: 2026-05-12T03:34:18Z → completed: 2026-05-12T03:34:50Z)
  Evidence: added a follow-up implementation task cut covering config defaults, `Builder` fields, key derivation helper, opt-in resolver dispatch, artifact adaptation, diagnostics, unchanged `Worker` semantics, and focused tests; `openspec validate wire-distributed-build-scheduler --strict` passed.

## Phase 2: Completion

- [x] [depends:implementation] Run `openspec validate wire-distributed-build-scheduler --strict` and record evidence before archive. ✅ 1m (started: 2026-05-12T03:35:09Z → completed: 2026-05-12T03:35:28Z)
  Evidence: `openspec validate wire-distributed-build-scheduler --strict` passed before archive.
