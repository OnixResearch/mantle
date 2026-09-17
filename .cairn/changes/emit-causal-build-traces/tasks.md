# Tasks: Emit causal build traces

All tasks remain open. Creating this proposal is not producer acceptance.

## Phase 1: Contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current failure-triage path: build log, report, transcript, and remote trace context, plus one dependency-failure observation. r[mantle.operator_diagnostics.causal_trace_records]
- [ ] [serial] T1.2 Define the trace record schema, action identity, cause vocabulary, and bounds. r[mantle.operator_diagnostics.causal_trace_cause_validation]
- [ ] [serial] T1.3 Record the identity-edge, closed-vocabulary, and diagnose-not-promote decisions in an ADR. r[mantle.operator_diagnostics.causal_trace_bounds]

## Phase 2: Core and emission

- [ ] [serial] T2.1 Implement pure record validation, cause validation, chain walking, canonical ordering, and bounds checks. r[mantle.operator_diagnostics.causal_trace_cause_validation]
- [ ] [serial] T2.2 Emit records from scheduler actions: root requirement, dependency ready, dispatch, cache decision, retry, cancellation, cleanup, and external trigger. r[mantle.operator_diagnostics.causal_trace_records]
- [ ] [serial] T2.3 Apply bounds and redaction to emitted records under the existing diagnostic rules. r[mantle.operator_diagnostics.causal_trace_bounds]
- [ ] [serial] T2.4 Keep trace records outside receipts and rejected by evidence validators. r[mantle.operator_diagnostics.causal_trace_bounds]

## Phase 3: Fixtures

- [ ] [parallel] T3.1 Add positive fixtures: dependency-failure chain from root to leaf, cache-hit cause, retry cause, and a parallel-goal interleaving that still chains correctly. r[mantle.operator_diagnostics.causal_trace_records]
- [ ] [parallel] T3.2 Add negative fixtures: unknown cause, missing cause, dangling cause identity, record over bound, trace over action bound, and unredacted sensitive value. r[mantle.operator_diagnostics.causal_trace_cause_validation]
- [ ] [parallel] T3.3 Prove existing receipts and reports are unchanged when tracing is enabled. r[mantle.operator_diagnostics.causal_trace_bounds]

## Phase 4: Verification

- [ ] [serial] T4.1 Run the chain, cache, retry, and negative rails before and after the change. Preserve exact results. r[mantle.operator_diagnostics.causal_trace_cause_validation]
- [ ] [serial] T4.2 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[mantle.operator_diagnostics.causal_trace_records]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.operator_diagnostics.causal_trace_bounds]
