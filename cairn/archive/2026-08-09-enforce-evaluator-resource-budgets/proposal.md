# Enforce evaluator resource budgets

## Why

Mantle benchmarks Nickel evaluation, selected-root latency, all-root forcing, and explicit root requests. The benchmark bundle records wall time and root-request counts, but it does not record peak resident memory or enforce evaluation time and memory policy.

Evaluation currently runs inside the Mantle process. An evaluator hang, runaway allocation, import explosion, or diagnostic explosion can therefore consume the operator process before Mantle can emit a bounded result. Explicit root-request counts also describe Mantle API calls, not every internal Nickel thunk or deep-export operation.

Mantle needs observable and enforceable evaluation budgets without treating performance evidence as evaluator correctness.

## What Changes

- Add a typed Nickel evaluation policy with named source, import, root, diagnostic, wall-time, CPU-time, resident-memory, response, and worker limits.
- Add a versioned bounded evaluator-worker protocol so strict evaluation can run in an owned child process.
- Keep request admission, budget decisions, outcome classification, and report construction in pure cores.
- Keep process launch, input transfer, clocks, resource observation, cancellation, kill, reap, and stderr capture in a thin shell.
- Report wall time, peak RSS when supported, CPU time when supported, source/import counts, selected roots, explicit force requests, diagnostics, and terminal disposition.
- Enforce deadlines and supported memory limits, with fail-closed unsupported diagnostics for strict policies that the host cannot enforce.
- Extend benchmark comparison with cohort-bound resource budgets and honest missing metrics.
- Add positive, negative, timeout, memory, crash, truncation, protocol, cancellation, and regression fixtures.

## Dependencies

- `add-evidence-driven-resource-policy` remains authoritative for remote build-attempt observations, machine-class selection, OOM escalation, and quota accounting. This change owns client-side Nickel evaluator admission, isolation, teardown, and metrics only.
- `support-portable-remote-client` can consume supported evaluator modes, but it does not broaden a platform's enforceable resource mechanisms.

## Non-Goals

- Implementing a second Nickel evaluator or claiming evaluator semantic equivalence.
- Treating explicit root-request counts as complete internal thunk observations.
- Using successful budget compliance as proof of build correctness or reproducibility.
- Adding unrestricted Import From Derivation or evaluator-controlled process execution.

## Impact

- **Affected specs:** new `evaluation-performance` capability
- **Planned files:** evaluation policy, worker protocol core, evaluator shell, `crunch-eval` adapters, benchmark schemas and tools, diagnostics, fixtures, and docs
- **Compatibility:** observe-only mode can preserve current in-process behavior during rollout; strict mode requires supported worker isolation and explicit policy
- **Testing:** protocol bounds, resource outcomes, selected-root behavior, worker loss, unsupported hosts, benchmark comparison, and Cairn gates
