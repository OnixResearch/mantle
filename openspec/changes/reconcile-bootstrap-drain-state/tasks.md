## Phase 1: Implementation

- [ ] [serial] Audit `.drain-state.md` against current OpenSpec queue and recent commits.
- [ ] [serial] Delete, archive, or replace the stale state with a current handoff pointer.
- [ ] [serial] Run `openspec validate --all --strict` and inspect `git status`.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate reconcile-bootstrap-drain-state --strict` and record evidence before archive.
