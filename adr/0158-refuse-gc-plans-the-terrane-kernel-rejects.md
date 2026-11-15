# 0158. Refuse GC plans the Terrane kernel rejects

Status: Proposed

## Context

ADR 0157 imported Terrane's verified `reclaim_plan` kernel and showed, on every
store of up to three entries, that `plan_gc` agrees with it and that the
kernel's `check_plan` accepts Mantle's output. Production stores are larger
than the enumerated space, so a `plan_gc` defect outside it could still delete
a live path.

The store deletes on two routes, both derived from a core plan. On the overlay
route it removes every overlay path outside the plan's retained set. On the
Casita route it removes the plan's candidates. Both core plans come from
`plan_gc` in `crunch-store` (`build_plan` and the Casita castore planner), and
each execution replans before deleting (ADR 0064).

ADRs 0060 and 0157 kept `vstd` out of the production dependency graph.

## Decision

This ADR reverses that driver for one crate. `crunch-gc-check` compiles the
imported kernel in production, with `vstd` as a normal dependency, because
`vstd` supplies the `verus!` macro that erases the kernel's ghost code under
plain rustc. No other crate depends on `vstd` directly, but every binary that
links `crunch-store` now links `vstd` and `verus_builtin`. `check_plan` has no
`requires` clause, so erasure removes no precondition its callers must meet.

`GcCheck::prepare` projects a `GcPlanRequest` into kernel indices before
`plan_gc` consumes it. Entries are sorted by path identifier, references and
roots are deduplicated, overlay entries are reclaimable, and a path that names
no entry maps out of range. Duplicate entries are refused. `GcCheck::check`
maps the plan's retained and candidate paths back onto those indices, refuses
any unknown or repeated path, requires the deletion intents to name exactly the
candidates, and runs the kernel's `check_plan`.

`crunch-store` obtains every core plan through `checked_plan_gc`. A request
that `plan_gc` refuses keeps its existing error. A plan the check refuses fails
with `gc-plan-check-failed` before any removal or Casita fence write, in both
dry-run and execution.

The kernel also proves that a planned result depends only on the sets of
roots and references (`planned_result_depends_only_on_sets`), which covers the
projection's sorting and deduplication, and that it is the least valid plan
and therefore unique (`planned_results_are_unique`). Every valid plan's
candidates are a subset of the planned result's, and adding roots, references,
or protection never adds a candidate (`stronger_input_never_adds_candidates`).

The ADR 0157 parity rail moves to `crunch-gc-check` and now exercises the
production projection and check, so the shipped projection is the one compared
with the kernel on the enumerated space.

## Consequences

- A plan that does not keep every root, is not closed under references, deletes
  a retained or base entry, or omits an unretained overlay entry is refused at
  runtime, whatever its size.
- With `plan_gc`'s reachability walk mutated to skip a reference, the store's
  GC tests fail with `gc-plan-check-failed` during planning, before the
  removal code is reached.
- At one million entries and 1.8 million references, preparing and checking
  took about 1.2 s against 2.1 s for `plan_gc` (release build, one local run).
  The projection also holds a second copy of every path identifier while it is
  alive.
- `vstd` and the Verus macro crates enter the production build, so their
  versions and licenses become part of Mantle's supply chain review.
- A refused plan blocks GC until the defect is fixed; there is no override.
- The kernel's bytes now ship in production, but, as for the Trellis import,
  no flake check runs the importer's `--check`, and the Tiger Style scope now
  covers the vendored module, whose bytes cannot be changed to satisfy it.

## Non-Claims

The projection is not verified beyond the set-invariance theorem: the mapping
from path identifiers to indices, and the treatment of unknown paths, are
unproved. The kernel is compiled with Mantle's `vstd`
(2026-01-25), not the Terrane cohort it was verified under (2026-10-18), and is
not re-verified there; erasure keeps its executable code unchanged. The check
does not establish that roots or references are complete, that the snapshot
contains every overlay path the store would remove, plan identity, byte totals,
or that removal executes the plan. Sidecar files, blob and chunk collection, and
action-result collection are derived from the checked retained set but are not
themselves checked.
