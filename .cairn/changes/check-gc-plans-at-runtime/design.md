# Design: Refuse GC plans the Terrane kernel rejects

## Decisions

### Decision: One production crate holds the kernel

**Choice:** `crunch-gc-check` is the only production crate that depends on
`vstd`. `crunch-gc-core` stays `no_std` and free of it.

**Rationale:** The kernel needs `std` and the `verus!` macro. Confining both to
one small crate keeps the reversal of the ADR 0060 driver narrow and visible.

### Decision: Project before planning

**Choice:** `GcCheck::prepare` borrows the request before `plan_gc` consumes
it. If preparation fails but `plan_gc` also refuses the request, the planner's
error is reported, so existing errors are unchanged.

**Rationale:** No copy of the request is needed, and callers that rely on
planner errors see no change.

### Decision: Fail closed on unmappable output

**Choice:** A retained or candidate path the request does not contain, a path
listed twice, or intents that differ from the candidates refuse the plan
before the kernel runs.

**Rationale:** Masks cannot represent these, so silently dropping them would
let the kernel check a different plan from the one executed.

### Decision: Check at the planner boundary

**Choice:** The check runs inside `checked_plan_gc`, which both `build_plan`
and the Casita castore planner call. Execution replans through the same
function before deleting.

**Rationale:** Every core plan that drives deletion passes one choke point,
and the plan that is executed is the plan that was checked.

## Non-Claims

See ADR 0158. The overlay route removes the complement of the checked
retained set within the overlay snapshot; that the snapshot and the request
describe the same paths is not checked.
