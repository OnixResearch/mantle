# Evaluator parity and boundary review

## Question

Can strict worker evaluation become authoritative while observe-only remains a bounded rollback?

## Inspected evidence

- `strict_worker_matches_existing_eval_and_reports_supported_resources` compares whole-value strict output with the existing unbudgeted evaluator output.
- `selected_and_all_root_requests_report_honest_force_counts` compares strict and observe-only selected-root and all-root outputs.
- `worker_preserves_declared_imports_and_bounded_evaluator_errors` compares strict and observe-only evaluator error class and diagnostic count.
- `strict_worker_confines_imports_to_admitted_roots` proves the unbudgeted path can read an outside import while strict Landlock confinement rejects it.
- Timeout, cancellation, late-success, forced-kill, panic, signal, CPU exhaustion, memory exhaustion, malformed frame, response flood, stderr flood, and simulated failed-reap fixtures have distinct terminal classes.
- Request metrics keep explicit root requests separate from unavailable Nickel internal evaluation counts.

## Decision

Accept strict worker authority for policy-bound `mantle eval`. Keep unbudgeted `mantle eval` and policy `observe-only` as rollback paths.

The selected-root adapter can render an integral Nickel number as `42.0` through `serde_json::Value`. Strict and observe-only selected-root paths agree on this output. Whole-value strict output remains equal to the existing whole-value output. This known adapter representation does not transfer a claim about arbitrary evaluator equivalence.

## Owner

Mantle evaluator boundary.

## Next action

Keep unexplained output, error-class, diagnostic, root, or request-observation drift as a cutover blocker. Keep performance differences separate from semantic drift.
