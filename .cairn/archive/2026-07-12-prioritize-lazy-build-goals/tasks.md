## Implementation

- [x] [serial] r[build_scheduling.deterministic_priority_kernel] Inventory every ready-queue insertion/pop path, dynamic-goal activation path, and route/resource fact source; record the current FIFO/discovery-order behavior.
- [x] [serial] r[build_scheduling.deterministic_priority_kernel] Define bounded provider-neutral scheduling facts, named priority tuple fields, total ordering, stable reason codes, and typed Nickel policy without opaque weighted scores.
- [x] [depends:scheduling-facts] r[build_scheduling.lazy_known_critical_path] Implement pure known-graph dependency/waiter pressure propagation with explicit structural fallback and bounded affected-goal recomputation.
- [x] [depends:scheduling-facts] r[build_scheduling.resource_locality_preference] Normalize eligible worker resource fit, receiver content presence, and transfer-cost classes outside the priority core.
- [x] [depends:scheduling-facts] r[build_scheduling.deterministic_starvation_bound] Implement pure scheduling-epoch and age-class transitions that eventually promote continuously ready eligible goals.
- [x] [depends:scheduling-core] r[build_scheduling.deterministic_priority_kernel] Replace `VecDeque` dispatch with a deterministic priority-ready abstraction while preserving goal dedupe, waiter notification, `max_jobs`, streaming evaluation, and dynamic plans.
- [x] [depends:priority-ready-set] r[build_scheduling.priority_decision_evidence] Add bounded human/JSON priority-basis evidence, including known-graph/history basis, resource/locality classes, age class, and stable tie-break reason.
- [x] [depends:priority-ready-set] r[build_scheduling.lazy_known_critical_path] Update ADR 0001 and scheduler docs to state that Mantle remains lazy and reports only a known-graph critical-path estimate.

## Verification

- [x] [depends:scheduling-core] r[build_scheduling.deterministic_priority_kernel] Add table/property/Kani tests proving comparator totality, antisymmetry, transitivity, permutation invariance, stable ties, bounded arithmetic, and identical replay for equivalent facts.
- [x] [depends:priority-ready-set] r[build_scheduling.lazy_known_critical_path] Positive: use chain, diamond, shared-dependency, streaming-root, and dynamic-plan fixtures to prove known critical-path pressure prioritizes work that unlocks more blocked roots without requiring full evaluation.
- [x] [depends:priority-ready-set] r[build_scheduling.resource_locality_preference] Positive and negative: prefer eligible resource/locality fits, but prove missing capability, output trust, upload policy, or hard resource fit remains ineligible regardless of priority.
- [x] [depends:priority-ready-set] r[build_scheduling.deterministic_starvation_bound] Positive: keep a goal continuously ready while higher critical-path work arrives and prove policy-bound age promotion dispatches it; negative: blocked or resource-ineligible goals do not age into eligibility.
- [x] [depends:priority-evidence] r[build_scheduling.priority_decision_evidence] Negative: shuffle discovery/map order, vary response timing, omit or stale the history snapshot, and prove deterministic fallback plus redacted diagnostics.
- [x] [depends:scheduler-verification] r[build_scheduling.deterministic_priority_kernel] Run focused crunch-build tests, scheduler benchmarks against FIFO fixtures, Cairn validate, and proposal/design/tasks gates; report performance evidence as comparative rather than optimality proof.

## Evidence summary

- Focused scheduler, Worker, Nickel-policy, build-report, producer-parity, and generated-contract suites passed; the combined rerun is pueue task `249`.
- Three Kani harnesses verified comparator antisymmetry/transitivity and bounded epoch advance with zero failures in pueue task `247`.
- `scheduler-benchmark.json` records six comparative fixtures and explicit non-claims; it does not claim global optimality or production throughput.
- Machine-contract generation/self-test/check and all three Cairn gates passed. See `evidence/validation.md` for commands, counts, receipts, and bounded broad-rail blockers.
