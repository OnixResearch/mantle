# Evaluation resource budgets

Mantle can run Nickel evaluation in an owned worker process. The parent process applies a checked policy, transfers one bounded request, and reaps the worker.

## Run strict evaluation

Use the checked default policy:

```sh
mantle eval examples/hello.ncl \
  --budget-policy config/evaluation/default-policy.json \
  --budget-report target/evaluation-report.json
```

The policy and report options must occur together. The JSON policy is generated from `config/evaluation/default-policy.ncl`.

Use `--root NAME` to request one or more top-level roots. Use `--all-roots` to discover and force all top-level roots. These options require a budget policy.

## Policy modes

`enforce` uses an owned worker. On Linux, Mantle uses these mechanisms:

- A parent monotonic deadline for wall time.
- `RLIMIT_CPU` for CPU time.
- `RLIMIT_AS` for the worker address space.
- Landlock read rules for the admitted import roots.
- `getrusage(RUSAGE_SELF)` for worker CPU time and peak RSS observations.

An address-space limit is not an RSS limit. The report names it `address_space_limit` and uses the mechanism `rlimit-as-not-rss`.

Strict evaluation fails before worker launch if a required host mechanism is not available. The parent and worker also count the bounded import-root surface. A changed surface fails before evaluation. The actual imported-module count stays unavailable because Nickel does not expose it.

`observe-only` keeps evaluation in the Mantle process. It is the rollback path. It reports operation-scoped CPU time and peak RSS as unavailable because the process can contain unrelated work.

## Reports

The report schema is `mantle-evaluation-budget-report-v1`. It binds the BLAKE3 policy and request references.

The report separates these fact roles:

- Admitted source, import, and selected-root request counts.
- Explicit top-level root force requests.
- Evaluator observations that Nickel exposes.
- Parent deadline and teardown facts.
- CPU and memory enforcement facts.
- CPU time and peak RSS observations.

`explicit_top_level_root_force_count` counts Mantle API requests. It is not a Nickel thunk count. `actual_nonselected_evaluation_count` stays unavailable until Nickel exposes this fact.

A missing observation has `status = "unavailable"`, no value, and a stable reason. Mantle does not use zero as a substitute.

## Protocol and teardown

The worker protocol uses a versioned request and response. Each message has an eight-byte little-endian length header. Mantle checks the declared length before allocation and rejects trailing bytes.

The parent drains bounded stdout and stderr concurrently. A deadline or cancellation sends termination first. If the worker does not stop during the configured grace period, the parent kills and reaps it.

A timeout or cancellation is terminal. A late successful response cannot replace that result.

## Benchmark comparison

Benchmark bundles now contain a resource cohort. The cohort binds the host class, target, evaluator, toolchain, policy, fixture set, repeat count, warm state, and measurement support.

The current in-process benchmark suite records CPU time and peak RSS as unavailable. It does not estimate them. Comparison applies thresholds only when both resource cohorts match. Named metric thresholds can override the default absolute and percentage thresholds.

## Limits and non-claims

Budget compliance reports bounded observations for one request. It does not prove evaluator correctness, reproducibility, build correctness, package correctness, or release eligibility.
