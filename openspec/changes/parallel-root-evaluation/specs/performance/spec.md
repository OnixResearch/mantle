## ADDED Requirements

### Requirement: Parallel root evaluation has checked-in throughput evidence

The benchmark workflow MUST provide checked-in evidence for serial-versus-
parallel all-root evaluation throughput on `tests/fixtures/wide_package_set.ncl`
or another fixed multi-root fixture with the same `/nix/store`-compatible
constraints.

That fixture MUST expose at least 16 top-level roots so parallel throughput
measurements are meaningful.

That evidence MUST report machine-readable metrics for:
- a serial all-root evaluation path,
- a bounded parallel all-root evaluation path, and
- the concurrency value used for the parallel run.

The workflow MUST keep selected-root lazy metrics visible as guardrails rather
than silently replacing them.

The workflow MUST record evidence only from an isolated benchmark run: no
concurrent Cargo build or test may share the same target directory during the
measurement window.

#### Scenario: Maintainer can inspect serial and parallel all-root metrics

- GIVEN a maintainer runs the checked-in parallel-root benchmark workflow
- WHEN the machine-readable result bundle or comparison artifact is inspected
- THEN it includes metrics for both serial and bounded parallel all-root
  evaluation on the same fixed fixture
- AND it reports the concurrency value used for the parallel run
- AND the evidence comes from an isolated benchmark run rather than a shared
  target directory with unrelated Cargo work

#### Scenario: Selected-root guardrails remain visible

- GIVEN a maintainer reviews evidence for parallel root evaluation
- WHEN they inspect the benchmark artifacts
- THEN selected-root lazy metrics remain present as guardrails
- AND the workflow does not treat wide all-root throughput as permission to hide
  selected-root regressions
