## Why

Mantle's lazy goal scheduler is the correct architecture, but its current ready set is a `VecDeque<String>` dispatched with `pop_front()`. FIFO is deterministic only relative to discovery order and ignores which ready goal unlocks the most blocked work, which worker can actually fit it, and which inputs are already local. At farm scale that leaves makespan and transfer efficiency on the table.

Mantle should adapt Rio's critical-path/resource/locality ordering while preserving ADR 0001's lazy graph, streaming evaluation, dynamic goals, waiter dedupe, and provider-neutral build-service boundary. It must not replace the scheduler with Rio's eager global DAG.

## What Changes

- Replace FIFO-ready dispatch with a deterministic priority-ready abstraction.
- Compute a bounded known-graph critical-path/blocked-root pressure estimate that updates as lazy goals and dynamic plans appear.
- Rank explicit resource fit and content locality/transfer-cost facts without using response races or provider-specific fields.
- Add deterministic starvation prevention using scheduling epochs and policy-bound age classes rather than hidden wall-clock reads.
- Extract the comparator/ranking logic as a pure kernel over immutable scheduling facts.
- Emit bounded priority-basis diagnostics so a replay can explain why one goal ran before another.
- Keep `max_jobs`, goal dedupe, waiter notification, route selection, output admission, and CI separation unchanged.

## Impact

- **Surfaces**: `crates/crunch-build/src/worker.rs`, `goal.rs`, `distributed.rs`, scheduler configuration/report DTOs, benchmark fixtures, and ADR 0001 follow-up documentation.
- **Dependencies**: independent of transport implementation; remote resource/locality facts enter only after route eligibility has already passed.
- **Non-claims**: no eager global DAG, optimal makespan proof, predictive/ML scheduler, CI job priority semantics, or implicit latency-based routing.
- **Validation**: comparator property tests, dynamic/diamond graph fixtures, resource/locality and starvation positives/negatives, deterministic replay, benchmark evidence, focused Cargo/Kani checks, and Cairn gates.
