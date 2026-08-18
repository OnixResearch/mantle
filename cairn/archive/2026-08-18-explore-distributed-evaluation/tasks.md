## Phase 1: Inventory and baseline

- [x] [serial] I1 Inventory every evaluation seam and bind it to its source path, accepted spec requirement ID, or ADR identity. r[distributed_evaluation.feasibility_assessment]
  - Evidence: `evidence/inventory.md`; the same inventory ships as `tests/fixtures/distributed-evaluation/declared-facts.json`.
  - Recorded `crunch-eval::session` isolated workers, the ADR 0074 strict worker protocol, evaluation-stream workers, `crunch-eval-budget-core`, `crunch-evaluation-stream-core`, `mantle-portable-client-core`, source staging, and the remote-build data plane.
  - Recorded the accepted `evaluation-performance`, `evaluation-streaming`, `remote-builds`, and `realization-routing` facts the assessment must not disturb.
- [x] [parallel] I2 Record current positive and negative evaluation and remote-dispatch behavior before any assessment. r[distributed_evaluation.feasibility_assessment]
  - Evidence: `evidence/inventory.md` behavioral-baseline section; accepted remote-builds, realization-routing, evaluation-performance, and evaluation-streaming requirements cited by identity.
  - Baseline: references `cairn/specs/remote-builds/spec.md` (workers never evaluate), `openspec/specs/distributed-builds/spec.md` (derivation is the unit), `adr/0001` (streaming overlap, dynamic goals), `adr/0002` (suspension deferred).

## Phase 2: Feasibility analysis and decision core

- [x] [serial] I3 Add the pure assessment core over normalized inventory, route, and probe facts. r[distributed_evaluation.feasibility_assessment]
  - Evidence: `src/distributed_eval_assessment.rs`; `cargo test --bin mantle distributed_eval_assessment` passed 24/24 (positive and negative).
  - Selects one outcome from `candidate`, `rejected`, or `blocked` per route with deterministic reasons.
  - Uses named limits, checked arithmetic, explicit assertions, and stable BLAKE3 identities.
- [x] [serial] I4 Add the bounded eval round-trip probe over one transport. r[distributed_evaluation.feasibility_assessment]
  - Evidence: `examples/distributed_eval_assess.rs`; `evidence/probe-and-report.md`.
  - Renders an isolated eval-worker request, sends it over a framed stdio transport to a self-spawned worker, evaluates a small fixture, and verifies the response against the in-process baseline.
  - Mirrors the ADR 0074 framing: eight-byte little-endian length header, bounded frames, oversized-frame rejection.
- [x] [parallel] I5 Add positive and negative fixtures for all outcomes and malformed assessment facts. r[distributed_evaluation.feasibility_assessment]
  - Evidence: 24 core unit tests covering candidate, rejected, blocked, malformed digests, missing routes, duplicates, oversized sets, empty bindings, and determinism; shell self-test covers framing round trip, oversized-frame rejection, and worker evaluation.

## Phase 3: Evidence and decision

- [x] [serial] I6 Run the probe and produce the deterministic feasibility report. r[distributed_evaluation.feasibility_assessment]
  - Evidence: `evidence/probe-evidence.json` (positive, `response_matches=true`, source BLAKE3 `3f28de41...`), `evidence/probe-evidence-negative.json` (negative, clean fact-sensitive eval error), `evidence/assessment-report.json` (stored-file BLAKE3 `0ea7fc73...`).
  - Result: eval-service `blocked` (missing cross-boundary streaming, dynamic-goal, and suspension evidence); evaluate-once `candidate`.
  - Bound the exact fixture, declared facts, probe evidence, and report identities.
- [x] [serial] I7 Record the oracle checkpoint and ADR with explicit non-claims. r[distributed_evaluation.feasibility_assessment]
  - Evidence: `evidence/oracle-checkpoint.md`; `adr/0078-explore-distributed-evaluation.md` (indexed in `adr/README.md`).
  - No evaluation, remote-build dispatch, scheduler, provider, or publication state changed.

## Phase 4: Verification

- [x] [serial] V1 Run positive and negative self-tests for the assessment core and the round-trip probe. r[distributed_evaluation.feasibility_assessment]
  - Evidence: `cargo test --bin mantle distributed_eval_assessment` → `test result: ok. 24 passed; 0 failed`; `distributed_eval_assess --self-test` → `self-test-ok`.
  - Proved all three outcomes and rejected malformed, oversized, incomplete, and contradictory assessment facts.
- [x] [serial] V2 Run repository and lifecycle gates before archive. r[distributed_evaluation.feasibility_assessment]
  - Evidence: `nix run github:onixresearch/cairn/fb1a7403a7897f7fa161e0b3c5d86b4cf19e52f8#cairn -- validate --root . --strict` valid; proposal, design, and tasks gates PASS; Cairn traceability coverage after sync.
  - Ran `git diff --check` and format checks on touched files.
