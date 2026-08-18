## Phase 1: Inventory and baseline

- [ ] [serial] I1 Inventory every evaluation seam and bind it to its source path, accepted spec requirement ID, or ADR identity. r[distributed_evaluation.feasibility_assessment]
  - Record `crunch-eval::session` isolated workers, the ADR 0074 strict worker protocol, `crunch-eval-budget-core`, evaluation-stream workers, `crunch-evaluation-stream-core`, `mantle-portable-client-core`, source staging, and the remote-build data plane.
  - Record the accepted `evaluation-performance`, `evaluation-streaming`, `remote-builds`, and `realization-routing` facts that the assessment must not disturb.
- [ ] [parallel] I2 Record current positive and negative evaluation and remote-dispatch behavior before any assessment. r[distributed_evaluation.feasibility_assessment]
  - Capture local evaluation parity, strict worker isolation, streaming outcomes, and remote concrete-input dispatch behavior under explicit baselines.

## Phase 2: Feasibility analysis and decision core

- [ ] [serial] I3 Add the pure assessment core over normalized inventory, route, and probe facts. r[distributed_evaluation.feasibility_assessment]
  - Select one outcome from `candidate`, `rejected`, or `blocked` per route with deterministic reasons.
  - Use named limits, checked arithmetic, explicit assertions, and stable BLAKE3 identities.
- [ ] [serial] I4 Add the bounded eval round-trip probe over one transport. r[distributed_evaluation.feasibility_assessment]
  - Render an isolated eval-worker request, send it over a single stdio or socket transport, evaluate a small fixture, and verify the returned response.
  - Run the same fixture through the in-process path and compare results and diagnostics.
- [ ] [parallel] I5 Add positive and negative fixtures for all outcomes and malformed assessment facts. r[distributed_evaluation.feasibility_assessment]
  - Cover missing inventory, hidden host-local reads, undeclared imports, ambient stdlib dependence, malformed framing, oversized messages, response disagreement, and contradictory reports.

## Phase 3: Evidence and decision

- [ ] [serial] I6 Run the probe and produce the deterministic feasibility report. r[distributed_evaluation.feasibility_assessment]
  - Bind the exact evaluation baseline, probe transport, fixture, evaluator, and evidence identities.
  - Preserve the probe transcripts and the final report in lifecycle evidence.
- [ ] [serial] I7 Record the oracle checkpoint and ADR with explicit non-claims. r[distributed_evaluation.feasibility_assessment]
  - Write an oracle checkpoint with question, inspected evidence, decision, owner, and next action.
  - Write an ADR that preserves the outcome and all non-claims.
  - Leave evaluation, remote-build dispatch, scheduler, provider, and publication state unchanged.

## Phase 4: Verification

- [ ] [serial] V1 Run positive and negative self-tests for the assessment core and the round-trip probe. r[distributed_evaluation.feasibility_assessment]
  - Prove all three outcomes and reject malformed, oversized, incomplete, and contradictory assessment facts.
- [ ] [serial] V2 Run repository and lifecycle gates before archive. r[distributed_evaluation.feasibility_assessment]
  - Run `git diff --check` and relevant format checks.
  - Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`.
  - Run all proposal, design, and tasks gates for `explore-distributed-evaluation`.
  - Run Cairn traceability coverage and the smallest relevant Nix check.
