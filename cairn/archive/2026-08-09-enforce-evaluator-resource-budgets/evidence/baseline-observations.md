# Baseline observations

Question: What does the current evaluator measure and enforce before this change?

Inspected evidence:

- `crates/crunch-eval/src/session.rs` keeps Nickel evaluation in the operator process.
- `examples/benchmark_support.rs` records wall time and explicit API request counts.
- `/tmp/mantle-eval-budget-baseline.json` records one smoke sample.
- `evidence/baseline-tests.md` records current evaluator and benchmark tests.

Observed baseline:

- The smoke sample recorded `evaluation_wall_ns=125970865` and `total_wall_ns=125977448`.
- The bundle has no peak-RSS, CPU-time, policy, mechanism-support, cancellation, or teardown fields.
- `explicit_top_level_root_force_count` means a Mantle API request. It is not a Nickel thunk count.
- The current evaluator path has no owned process that a timeout can kill and reap.
- A runaway allocation remains in the Mantle process. Sampling that process cannot enforce a worker memory boundary.

Decision: Keep the existing in-process path as observe-only rollback. Add strict authority only through an owned worker process.

Owner: This Cairn change.

Next action: Add a pure admission/protocol core, a same-binary worker shell, and cohort-bound benchmark resource facts.
