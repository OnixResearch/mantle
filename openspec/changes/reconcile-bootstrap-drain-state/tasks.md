## Phase 1: Implementation

- [x] [serial] Audit `.drain-state.md` against current OpenSpec queue and recent commits. ✅ 1m (started: 2026-05-12T03:28:37Z → completed: 2026-05-12T03:28:48Z)
  Evidence: active OpenSpec queue is `reconcile-bootstrap-drain-state,wire-distributed-build-scheduler`; tracked root `.drain-state.md` is dated `2026-04-27T20:00Z` and lists stale `live-bootstrap-*` active changes, while recent commits include `1b241ee5 openspec: archive gcc40 libgcc semantic member`.
- [x] [serial] Delete, archive, or replace the stale state with a current handoff pointer. ✅ 1m (started: 2026-05-12T03:29:06Z → completed: 2026-05-12T03:29:28Z)
  Evidence: removed tracked root `.drain-state.md`; stale content remains recoverable in git history (`e979cd42`, `e8413b14`) and live drain handoff is now the untracked/generated `openspec/changes/.drain-state.md` loop file.
- [x] [serial] Run `openspec validate --all --strict` and inspect `git status`. ✅ 1m (started: 2026-05-12T03:29:49Z → completed: 2026-05-12T03:30:06Z)
  Evidence: `openspec validate --all --strict` passed (50 items); `git status --short --branch` showed only `openspec/changes/.drain-state.md` and this change's `tasks.md` modified; `test ! -e .drain-state.md` passed.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate reconcile-bootstrap-drain-state --strict` and record evidence before archive.
