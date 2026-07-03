## Context

Recent topology changes use serial focused tests as reliable evidence. Parallel runs can expose races around compiler-policy fixtures, environment poisoning, shared temp paths, or subprocess outputs. The goal is not to relax validation; it is to make existing validation deterministic under repeated execution.

## Decisions

### 1. Fixture state is per-test unless explicitly locked

**Choice:** Tests that create rustc wrappers, cargo shims, OUT_DIRs, receipt paths, or source roots must use per-test temp directories or named locks when state cannot be isolated.

**Rationale:** Parallel libtest execution should not cause unrelated topology tests to observe each other's state.

### 2. Ambient environment mutation remains subprocessed

**Choice:** Tests that prove planner resistance to ambient env values should continue to spawn child libtest processes rather than mutate process-global env in-process.

**Rationale:** In-process env mutation is inherently racy under parallel tests.

### 3. Stable command recipes become documented evidence rails

**Choice:** After stabilization, document the smallest serial and parallel commands that are expected to pass.

**Rationale:** Operators need to know which rails are first-party proof and which remain expensive or excluded.

### 4. Flake triage records root causes

**Choice:** Any remaining serial-only tests must have comments or docs naming the shared resource and why isolation is not yet practical.

**Rationale:** Serial-only requirements should be explicit technical debt.

## Risks / Trade-offs

- Some race fixes may require more fixture setup code.
- Parallel topology tests can become slower when each test owns isolated roots.
- Stabilization may reveal real planner bugs that were hidden by serial runs.
