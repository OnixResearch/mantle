# Distributed-evaluation feasibility: probe run and report

Change: `explore-distributed-evaluation`
Tasks: I3, I4, I5, I6, V1

## Scope

This evidence records the bounded eval round-trip probe, the pure assessment
core, and the deterministic report. The probe exercises the exact seam under
assessment: whether a fresh evaluation session built only from declared
request facts reproduces the client result.

## Method

The shell `examples/distributed_eval_assess.rs` performs a real process round
trip:

1. It opens an `EvaluationSession` from the fixture source, declared import
   paths, and source name. It forces the root in-process (client baseline).
2. It serializes those same declared facts into a framed request and sends it
   over a stdio pipe to a worker process of the same binary
   (`--probe-worker`).
3. The worker calls `crunch_eval::session::EvaluationSession::open_named_str`
   and forces the same root label. Discovery is deterministic over the
   transported facts, so any hidden host-local read surfaces as a mismatch.
4. The client compares source BLAKE3 identities and canonical result JSON.

The framing mirrors the ADR 0074 eval-budget protocol: an eight-byte
little-endian length header, bounded frames, and rejection of oversized data
before allocation.

Artifacts:

- `src/distributed_eval_assessment.rs` — pure assessment core (24 unit tests,
  positive and negative).
- `examples/distributed_eval_assess.rs` — probe shell with `--self-test`,
  `--probe-worker`, and `--probe-run`.
- `tests/fixtures/distributed-evaluation/probe-target.ncl` — probe fixture.
- `tests/fixtures/distributed-evaluation/declared-facts.json` — bound
  inventory and declared route facts.

## Positive round trip

Command:

```text
cargo run --quiet --example distributed_eval_assess -- --probe-run \
  --fixture tests/fixtures/distributed-evaluation/probe-target.ncl \
  --root greeting \
  --facts tests/fixtures/distributed-evaluation/declared-facts.json \
  --out /tmp/distributed-eval-assessment-evidence
```

Probe evidence (`probe-evidence.json`):

- `request_source_blake3` = `3f28de419a35adbc6b807c76d901a60ad24ca81169eb252893d5d10a708505a0`
- `response_source_blake3` = `3f28de419a35adbc6b807c76d901a60ad24ca81169eb252893d5d10a708505a0`
- `client_result_json` = `"hello-from-client"`
- `worker_result_json` = `"hello-from-client"`
- `response_matches` = `true`
- `worker_error_observed` = `false`
- `framed_transport_used` = `true`

The worker reproduced the client result byte-for-byte from declared facts
alone. For this fixture the eval-worker boundary carries no hidden host-local
state.

## Negative round trip

Command:

```text
cargo run --quiet --example distributed_eval_assess -- --probe-run \
  --fixture tests/fixtures/distributed-evaluation/probe-target.ncl \
  --root does-not-exist \
  --facts tests/fixtures/distributed-evaluation/declared-facts.json \
  --out /tmp/distributed-eval-assessment-evidence-negative
```

Probe evidence (`probe-evidence-negative.json`):

- `response_matches` = `false`
- `worker_error_observed` = `true`
- `worker_error_class` = `eval`
- `worker_error_message` = `no root with label 'does-not-exist'; available roots: greeting, message`

Changed declared facts changed the outcome deterministically. No hidden state
filled the gap.

## Assessment report

`assessment-report.json` (report BLAKE3 of the stored file:
`0ea7fc739e4300d3af20440a9fff6b299f5a1438020f31c774ddda1c9ce967a9`):

- `eval-service`: `blocked`
  - missing cross-boundary streaming-overlap evidence
  - missing cross-boundary dynamic-goal evidence
  - missing evaluator suspension evidence
- `evaluate-once`: `candidate`
  - passes all assessment gates

The eval-service route is `blocked`, not `rejected`. The probe proved the
boundary is transportable and the inventory is complete. The missing evidence
is the cross-boundary survival of streaming evaluation, dynamic-goal
admission, and evaluator suspension. Those are in-process facts today and are
not demonstrated across a network transport.

## Self-test and core tests

- `distributed_eval_assess --self-test` passed (`self-test-ok`), covering
  worker evaluation, framing round trip, oversized-frame rejection, and the
  three classifier outcomes.
- `cargo test --bin mantle distributed_eval_assessment` passed:
  `test result: ok. 24 passed; 0 failed`.

## Claims

This evidence proves the probe result, the classifier behavior, and the
recorded reasons. It does not prove distributed evaluation in the product,
cross-boundary streaming or dynamic goals, evaluator equivalence, remote-build
correctness, or release eligibility.
