# Distributed-evaluation feasibility: seam inventory and baseline

Change: `explore-distributed-evaluation`
Tasks: I1, I2

## Scope

This evidence records the evaluation architecture seams and the accepted
behavioral baseline that the assessment must not disturb. It is a mapping from
the assessment requirement
`r[distributed_evaluation.feasibility_assessment]` to concrete code, spec, and
ADR identities.

## Seam inventory

Every recorded seam is a real surface in the current tree. Each seam is bound
to a source path, accepted spec, or ADR. The shell consumes the same inventory
from `tests/fixtures/distributed-evaluation/declared-facts.json`.

| Seam | Identity |
|---|---|
| embedded-evaluator | `#crates/crunch-eval/src/lib.rs` — embedded Nickel evaluator with `evaluate_and_deserialize` and streamed root export |
| isolated-worker-sessions | `#crates/crunch-eval/src/session.rs` — `IsolatedWorkerInput` shares only source text, import paths, source name, shape, and labels across per-root worker sessions |
| strict-worker-protocol | `adr/0074-enforce-evaluation-budgets-with-an-owned-worker.md` — hidden same-binary worker, one fixed-width length-bounded frame, separate request and policy BLAKE3 identities, Landlock import-root rules |
| evaluation-stream-workers | `#crates/crunch-pipeline/src/evaluation_stream.rs` — `EvalWorkerLaunch` and `spawn_eval_worker` behind an outcome ledger |
| evaluation-budget-core | `#crates/crunch-eval-budget-core` — pure policy admission, framing checks, truncation, terminal classification |
| portable-client-boundary | `#crates/mantle-portable-client-core` — portable remote-client planning core |
| source-staging | `cairn/specs/store-transports/spec.md` — content-addressed castore and source-bundle staging |
| remote-build-data-plane | `docs/remote-transfer.md` — bounded resumable transfer, receiver-driven chunk demand, verified digests |

## Behavioral baseline (I2)

The assessment must not alter any of these accepted behaviors:

- Evaluation runs on the client. `cairn/specs/remote-builds/spec.md` requires
  remote builders to execute only concrete inputs and to never evaluate
  Nickel source.
- Derivation realization is the distributed unit of work.
  `openspec/specs/distributed-builds/spec.md` forbids scheduling arbitrary
  sub-actions.
- Realization routing stays provider-neutral and default-local.
  `cairn/specs/realization-routing/spec.md`.
- Evaluation budgets, worker framing, and rollout parity are governed by
  `cairn/specs/evaluation-performance/spec.md`.
- Streaming root outcomes are governed by `cairn/specs/evaluation-streaming/spec.md`.
- Streaming evaluation-to-build overlap and dynamic-goal admission are
  in-process behavior recorded in `adr/0001-lazy-goals-vs-eager-dag.md`.
- Evaluator suspension and resumption for import-from-derivation remain
  deferred in `adr/0002-dynamic-derivations.md`.

## Baseline checks

- Strict evaluation runs through `mantle eval` with a budget policy and
  report (`docs/evaluation-resource-budgets.md`).
- Isolated worker sessions reproduce per-root results without sharing a
  mutable Nickel `Context` (`IsolatedWorkerInput::force_root`).
- Note: `open_named_str` opens a session from transferred source and
  re-derives shape and labels deterministically. This is the seam the probe
  exercises.

## Claims

This inventory proves that the seams exist and are bound to identities. It
does not prove the worker boundary is transportable, streaming can survive a
network hop, or either candidate route is product-adopted. Those claims are
decided by the probe and classifier in the assessment report.
