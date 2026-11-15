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
`plan_gc` consumes it. Mantle sorts the entries by path identifier and looks up
an index for every reference and root, but the lookup is not trusted: the
kernel's verified `project_keys` refuses the projection unless the path bytes
are strictly sorted and every computed index names the entry whose bytes equal
the referenced path, and it builds the edges itself. A root or reference that
names no entry is refused. `GcCheck::check` maps the plan's retained and
candidate path lists onto indices the same way, through the kernel's verified
`key_mask`, requires the deletion intents to name exactly the candidates, and
runs the kernel's `check_plan`.

With both validations passed, the kernel's path-level theorems apply to the
plan Mantle executes: no listed candidate path is reachable from a root path
through the request's references (`no_listed_candidate_path_is_live`), and
every path reachable from a root path is listed as retained
(`every_live_path_is_listed_retained`).

`crunch-store` obtains every core plan through `checked_plan_gc`. A request
that `plan_gc` refuses keeps its existing error. A plan the check refuses fails
with `gc-plan-check-failed` before any removal or Casita fence write, in both
dry-run and execution.

The kernel also proves that a planned result depends only on the sets of
roots and references (`planned_result_depends_only_on_sets`), and that it is
the least valid plan
and therefore unique (`planned_results_are_unique`). Every valid plan's
candidates are a subset of the planned result's, and adding roots, references,
or protection never adds a candidate (`stronger_input_never_adds_candidates`).

The ADR 0157 parity rail moves to `crunch-gc-check`. It keeps an independent
test projection for the kernel comparison, requires the production projection
to be refused exactly when a root or reference names no entry and otherwise to
yield the same graph, and runs the production check on every planned result.

## Consequences

- A plan that does not keep every root, is not closed under references, deletes
  a retained or base entry, or omits an unretained overlay entry is refused at
  runtime, whatever its size.
- With `plan_gc`'s reachability walk mutated to skip a reference, the store's
  GC tests fail with `gc-plan-check-failed` during planning, before the
  removal code is reached.
- At one million entries and 1.8 million references, preparing and checking
  took about 1.2 s against 2.1 s for `plan_gc` (release build, one local run).
  That run predates the verified projection, which also copies every reference
  path's bytes; it has not been re-measured.
- `vstd` and the Verus macro crates enter the production build, so their
  versions and licenses become part of Mantle's supply chain review.
- A refused plan blocks GC until the defect is fixed; there is no override.
- The kernel's bytes now ship in production, but, as for the Trellis import,
  no flake check runs the importer's `--check`, and the Tiger Style scope now
  covers the vendored module, whose bytes cannot be changed to satisfy it.

## Non-Claims

The projection's index lookup is unverified but validated on every run, so a
wrong index is refused rather than trusted. Still unproved: that each entry's
path, references, and ownership flag are passed to the kernel together (one
loop in `GcCheck::prepare`). The kernel is compiled with Mantle's `vstd`
(2026-01-25), not the Terrane cohort it was verified under (2026-10-18), and is
not re-verified there; erasure keeps its executable code unchanged. The check
does not establish that roots or references are complete, that the snapshot
contains every overlay path the store would remove, plan identity, byte totals,
or that removal executes the plan. Sidecar files, blob and chunk collection, and
action-result collection are derived from the checked retained set but are not
themselves checked.
