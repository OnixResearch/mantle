# Tasks: enforce evaluator resource budgets

## Phase 1: Baseline and policy

- [x] [serial] I1 Run current `crunch-eval`, benchmark harness, selected-root, all-root, import, diagnostic, and cancellation tests before core changes. r[evaluation_performance.resource_observations]
- [x] [serial] I2 Record current wall metrics, explicit request-count semantics, absent peak-RSS evidence, and representative timeout and allocation blockers. r[evaluation_performance.metric_role_separation]
- [x] [serial] I3 Add a typed Nickel evaluation policy with named source, import, root, diagnostic, protocol, time, memory, worker, and teardown limits. r[evaluation_performance.budget_policy]
- [x] [parallel] I4 Add positive policy fixtures and negative zero, overflow, contradiction, unknown-field, unsupported-mode, and exceeded-limit fixtures. r[evaluation_performance.validation]

## Phase 2: Pure protocol and decision cores

- [x] [serial] I5 Define versioned worker request, response, diagnostic, observation, support, cancellation, and terminal outcome types. r[evaluation_performance.worker_protocol]
- [x] [serial] I6 Implement pure request admission, policy-mode selection, truncation, terminal classification, and report construction. r[evaluation_performance.budget_policy]
- [x] [serial] I7 Implement fixed-width bounded framing with rejection before disallowed allocation. r[evaluation_performance.worker_protocol]
- [x] [parallel] I8 Add property tests for deterministic reports, stable ordering, checked arithmetic, frame bounds, terminal-state exclusivity, and equivalent-fact replay. r[evaluation_performance.validation]

## Phase 3: Evaluator worker shell

- [x] [serial] I9 Add the hidden owned evaluator worker entry point and confined source and import handoff. r[evaluation_performance.worker_protocol]
- [x] [serial] I10 Add parent-side process launch, deadline, supported CPU and memory limit setup, peak observation, bounded stderr capture, cancellation, kill, and reap. r[evaluation_performance.enforced_teardown]
- [x] [serial] I11 Return fail-closed unsupported results when strict requested mechanisms are unavailable. Keep observe-only missing metrics explicit. r[evaluation_performance.budget_policy]
- [x] [parallel] I12 Add timeout, memory, panic, signal, malformed-frame, response-flood, stderr-flood, late-response, cancellation, kill, and failed-reap process fixtures. r[evaluation_performance.validation]

## Phase 4: Evaluation and benchmark integration

- [x] [serial] I13 Route strict evaluation through the worker while preserving the current path for observe-only parity and bounded rollback. r[evaluation_performance.rollout]
- [x] [serial] I14 Report wall, CPU, peak RSS, source/import/root counts, explicit force requests, supported evaluator observations, diagnostics, and terminal disposition by distinct roles. r[evaluation_performance.resource_observations]
- [x] [serial] I15 Extend benchmark bundles and comparison policy with cohort identity, metric support, named resource thresholds, and honest missing metrics. r[evaluation_performance.benchmark_gates]
- [x] [parallel] I16 Add selected-root, all-root, import, evaluator-error, compatible-cohort, mismatched-cohort, memory-regression, time-regression, and missing-metric fixtures. r[evaluation_performance.validation]

## Phase 5: Cutover and validation

- [x] [serial] I17 Dual-run bounded positive and negative fixtures and classify output, error, diagnostic, root, and request-observation drift before strict cutover. r[evaluation_performance.rollout]
- [x] [serial] I18 Document policy modes, supported platforms, metric roles, cancellation, benchmark use, rollback, and non-claims. r[evaluation_performance.metric_role_separation]
- [x] [serial] V1 Run `nix develop -c cargo test -p crunch-eval`, `nix develop -c cargo test -p mantle --test benchmark_harness`, and focused evaluator-worker process tests. r[evaluation_performance.validation]
- [x] [serial] V2 Run the smoke and full benchmark commands from `docs/benchmark-suite.md`, compare an accepted compatible baseline, and record wall and peak-RSS support evidence. r[evaluation_performance.benchmark_gates]
- [x] [serial] V3 Run focused formatting and Clippy with warnings denied, Nickel checks, machine-contract checks, timeout and process-leak checks, and `git diff --check`. r[evaluation_performance.validation]
- [x] [serial] V4 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .` plus proposal, design, and tasks gates for this change. Record exact outputs before archive. r[evaluation_performance.validation]
