## Context

crunch already moved in a Tiger Style direction in several places, but the
2026-04-12 audit shows that the most sensitive paths still carry mixed
responsibilities:

- top-level shell dispatch is still oversized,
- CA output finalization still combines pure and effectful work,
- castore rewrite/export still use recursion,
- large functions still lack dense assertions.

These are not cosmetic issues. They sit on paths that decide build identity,
path persistence, self-build staging, and store reconstruction.

## Goals / Non-Goals

**Goals**
- Make the highest-risk shell entrypoints small and phase-oriented.
- Isolate pure CA planning from store mutation and logging.
- Replace recursion in critical castore traversals with explicit bounded
  iteration.
- Raise assertion density and fixed-limit coverage where regressions would be
  expensive.

**Non-Goals**
- Rewrite the whole repo to `#![no_std]` core/shell crates in one pass.
- Change the user-visible CLI semantics for build, self-build, or store export.
- Eliminate every long function in the workspace in this change.
- Replace every use of dynamic allocation.

## Decisions

### 1. Scope the change to audited critical paths first

**Choice:** Refactor only the largest, highest-risk control paths found by the
Tiger Style audit: CLI dispatch, self-build orchestration, CA finalization,
and recursive castore walkers.

**Rationale:** These paths decide correctness properties for the whole system.
A smaller, sharper change is more likely to land than a repo-wide style sweep.

**Alternative:** Apply a broad Tiger Style cleanup across the whole workspace.

**Why not:** Too much surface area. It would blur core correctness work with
mechanical cleanup.

### 2. Use pure planning helpers plus imperative executors

**Choice:** For CA finalization and similar flows, add pure helpers that return
planned rewrites, path names, attestation graph inputs, or validation results.
Shell helpers then execute blob/directory/store writes, registry updates, and
logging.

**Rationale:** This keeps business logic testable with plain values and makes
store mutations easier to audit.

**Alternative:** Keep the current all-in-one async functions and add comments.

**Why not:** Comments do not create test seams or reduce mixed control flow.

### 3. Replace recursive walkers with explicit bounded worklists

**Choice:** Rewrite castore traversal helpers to use explicit stacks or queues
with fixed limits instead of recursive async calls.

**Rationale:** Tiger Style prefers bounded, explicit control flow. Worklists
also make per-node assertions and limit checks easier to place.

**Alternative:** Keep recursion and rely on depth counters alone.

**Why not:** The explicit recursion is still harder to audit and keeps stack
shape coupled to input shape.

### 4. Treat assertions as part of the contract

**Choice:** Add targeted runtime assertions and compile-time constant checks in
critical helpers, especially around marker lengths, output counts, queue
limits, and required invariants.

**Rationale:** These paths already rely on strong assumptions. Tiger Style says
make them executable and local.

**Alternative:** Rely on existing tests alone.

**Why not:** Tests catch sampled behavior. Assertions document and enforce the
internal contracts continuously.

## Risks / Trade-offs

**Refactor churn in delicate paths** -> Mitigation: keep behavior-preserving
changes small, add focused regression tests, and split pure planning from I/O
before changing algorithms.

**Spec becoming too prescriptive** -> Mitigation: require structural
boundaries and bounded traversal, not one exact helper layout.

**Audit drift after landing** -> Mitigation: keep one repeatable Tiger Style
scan in the verification plan so the same hotspots can be checked again.
