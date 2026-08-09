# Stream independent root outcomes

## Why

Mantle can discover several evaluation roots and force them with bounded parallel workers. The pipeline currently records one `first_failure` and stops dispatching new roots after that failure.

One invalid root can therefore prevent independent roots from producing useful evaluation and build results. Machine consumers also lack a stable record stream for complete per-root outcomes.

Mantle needs complete root accounting without treating partial evaluation as full success.

## What Changes

- Continue independent root evaluation after one root returns a scoped evaluation error.
- Emit exactly one terminal outcome for every selected root.
- Distinguish root-scoped errors from shared fatal errors and operator cancellation.
- Add a versioned, bounded NDJSON event stream for evaluation and pipeline progress.
- Assign deterministic root identities and source-order sequence values before parallel work starts.
- Emit a canonical terminal summary with `success`, `partial`, `failed`, or `cancelled` disposition.
- Preserve successful root results when another root fails.
- Add positive and negative fixtures for errors, cancellation, broken streams, ordering, bounds, and worker loss.

## Dependencies

`enforce-evaluator-resource-budgets` remains authoritative for worker isolation, resource limits, process teardown, and resource observations. This change owns root outcome semantics and public stream projection.

`cache-substitution` remains authoritative for cache admission facts and structured cache diagnostics. This change can carry those facts without redefining them.

The design adapts selected concepts from `NixOS/nix-eval-jobs` revision `a0cd02231c58974a6b5aaa3712069b071047162e`. It adds no runtime or source dependency on that GPL-3.0 project.

## Non-Goals

- Adding Hydra aggregate or constituent semantics.
- Adding `recurseForDerivations`, arbitrary `--apply`, or evaluator callback expressions.
- Replacing Mantle roots with Nix attribute traversal rules.
- Treating partial evaluation as build correctness, reproducibility, or release readiness.
- Moving cache, store, build, deployment, or CI scheduling authority into the stream schema.

## Impact

- **Affected specs:** new `evaluation-streaming` capability.
- **Planned files:** `crunch-eval`, `crunch-pipeline`, CLI projection code, machine schemas, fixtures, and documentation.
- **Compatibility:** existing aggregate output remains available during a bounded migration period.
- **Testing:** baseline tests, pure outcome tests, pipeline tests, CLI stream fixtures, machine-contract checks, and Cairn gates.
