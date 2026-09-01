# Build Tiger Style baseline

## Result

The focused strict command exits 1 with 20 findings:

- 9 in `crates/crunch-build/src/build_request.rs`;
- 5 in `crates/crunch-build/src/execution_profile.rs`;
- 3 in `crates/crunch-build/src/ca_plan.rs`;
- 1 in `crates/crunch-build/src/registry.rs`;
- 1 in `crates/crunch-build/src/worker.rs`;
- 1 in `crates/crunch-rustc-wrapper/src/lib.rs`.

Lint families are assertion density (5), too many parameters (3), panic (3),
`expect` or unwrap (2), predicate naming (2), numeric units (1), unchecked
arithmetic (1), recursion (1), compound condition (1), and ambiguous parameters
(1).

The pre-change package command passes 702 tests with zero failures. Its targets
report 677, 1, 20, 2, 2, and 0 tests.

This evidence is a failing strict baseline, not an accepted allowance or finding
budget.
