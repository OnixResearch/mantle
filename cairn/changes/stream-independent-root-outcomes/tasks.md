# Tasks: stream independent root outcomes

## Phase 1: Baseline and contract

- [x] [serial] I1 Run current `crunch-eval`, `crunch-pipeline`, CLI evaluation, and machine-output tests before core changes. Record the current first-failure behavior. r[evaluation_streaming.complete_root_outcomes]
  Evidence: `evidence/baseline.md` records 164 passing focused tests, the machine-contract self-test, the existing inventory failure, and exact `first_failure` behavior.
- [x] [serial] I2 Record `NixOS/nix-eval-jobs` revision `a0cd02231c58974a6b5aaa3712069b071047162e`, GPL-3.0 license, selected concepts, rejected compatibility behavior, and non-claims. r[evaluation_streaming.reference_boundary]
  Evidence: `evidence/reference-boundary.md` records the reviewed revision, license, adapted concepts, rejected semantics, and non-claims.
- [x] [serial] I3 Define the stream schema, record kinds, terminal states, failure scopes, dispositions, named bounds, compatibility period, and process-status mapping. r[evaluation_streaming.versioned_event_stream] r[evaluation_streaming.partial_run_disposition]
  Evidence: `docs/evaluation-stream-contract.md`, ADR 0075, and the v1 JSON Schema define the complete Phase 1 contract.
- [x] [parallel] I4 Add positive schema fixtures and negative unknown-version, unknown-kind, duplicate-field, oversized-field, and malformed-record fixtures. r[evaluation_streaming.validation]
  Evidence: `evidence/contract-validation.md` records two admitted fixtures and five target-boundary rejections.

## Phase 2: Pure outcome and projection cores

- [x] [serial] I5 Define bounded selected-root, sequence, failure-scope, root-outcome, run-disposition, summary, and stream-record types. r[evaluation_streaming.complete_root_outcomes] r[evaluation_streaming.failure_scope]
  Evidence: `crunch-evaluation-stream-core` defines the validated `no_std` types and named limits.
- [x] [serial] I6 Implement pure root-set admission, terminal transition checks, duplicate rejection, and complete-summary accounting. r[evaluation_streaming.complete_root_outcomes]
  Evidence: `OutcomeLedger` rejects invalid transitions and creates a summary only after every selected root is terminal.
- [x] [serial] I7 Implement pure failure-scope classification and remaining-root classification for shared fatal errors and cancellation. r[evaluation_streaming.failure_scope] r[evaluation_streaming.cancellation_and_output]
  Evidence: focused tests cover root-scoped, shared-fatal, cancellation, and coordinator-failure decisions.
- [x] [serial] I8 Implement deterministic sequence assignment, canonical ordering, and domain-separated BLAKE3 identity preimages. r[evaluation_streaming.deterministic_identity_and_order]
  Evidence: known-answer and schedule-independence tests bind the documented root and run identity framing.
- [x] [serial] I9 Implement pure stream projection values, terminal disposition, and process-status selection without JSON or I/O helpers. r[evaluation_streaming.partial_run_disposition] r[evaluation_streaming.core_shell_boundary]
  Evidence: pure projection and process-status tests pass, and production code has no `std`, JSON, async, or I/O dependency.
- [x] [parallel] I10 Add property tests for terminal exclusivity, complete accounting, stable ordering, schedule independence, checked bounds, and equivalent-fact replay. r[evaluation_streaming.validation]
  Evidence: `evidence/pure-outcome-core.md` records five property tests that cover all six required properties.

## Phase 3: Pipeline and worker integration

- [x] [serial] I11 Replace `first_failure` dispatch control with typed root and shared outcomes while preserving bounded worker concurrency. r[evaluation_streaming.complete_root_outcomes] r[evaluation_streaming.failure_scope]
  Evidence: `evidence/pipeline-worker-integration.md` records the ledger coordinator, unchanged concurrency bound, and complete root accounting.
- [x] [serial] I12 Continue independent evaluation and build conversion after root-scoped errors. Stop safely after shared fatal errors. r[evaluation_streaming.complete_root_outcomes]
  Evidence: single-worker tests prove a later sibling succeeds after evaluation/deserialization and conversion failures; opaque evaluator and shared initialization failures stop dispatch.
- [x] [serial] I13 Integrate cancellation so new dispatch stops, owned work receives cancellation, late success cannot replace cancellation, and every root becomes terminal. r[evaluation_streaming.cancellation_and_output]
  Evidence: `EvaluationCancellation` and the cancellation race test prove stop, wake, terminalization, and late-success rejection.
- [x] [parallel] I14 Add pipeline fixtures for one malformed sibling, recursive evaluation, worker loss, shared initialization failure, cancellation races, and late results. r[evaluation_streaming.validation]
  Evidence: all required fixture classes pass in `crunch-pipeline`; see `evidence/pipeline-worker-integration.md`.

## Phase 4: Machine stream and compatibility shell

- [x] [serial] I15 Add an explicit CLI stream mode that emits bounded NDJSON on stdout and keeps human diagnostics on stderr. r[evaluation_streaming.versioned_event_stream]
- [x] [serial] I16 Emit live records in observed completion order and one canonical final summary in source-order sequence. r[evaluation_streaming.deterministic_identity_and_order]
- [x] [serial] I17 Preserve successful root references in partial runs while returning the admitted non-success process disposition. r[evaluation_streaming.partial_run_disposition]
- [x] [serial] I18 Add broken-pipe, flush-failure, output-cancellation, and missing-summary handling without false success. r[evaluation_streaming.cancellation_and_output]
- [x] [serial] I19 Keep aggregate output available for the declared migration period and add dependency gates for forbidden upstream code and semantics. r[evaluation_streaming.reference_boundary]
- [x] [parallel] I20 Add CLI golden fixtures for all record kinds, mixed outcomes, order variation, redaction, bounds, process status, and aggregate compatibility. r[evaluation_streaming.validation]

## Phase 5: Documentation and completion evidence

- [x] [serial] I21 Document stream records, identity inputs, ordering, failure scope, cancellation, partial-result handling, compatibility, and non-claims. r[evaluation_streaming.versioned_event_stream] r[evaluation_streaming.reference_boundary]
- [ ] [serial] V1 Rerun every Phase 1 baseline command and all focused positive and negative tests. r[evaluation_streaming.validation]
- [ ] [serial] V2 Run formatting, focused Clippy with warnings denied, machine-contract checks, architecture checks, and `git diff --check`. r[evaluation_streaming.core_shell_boundary]
- [ ] [serial] V3 Run Cairn validation, proposal, design, and tasks gates plus Tracey coverage. Record exact output before sync and archive. r[evaluation_streaming.validation]
- [ ] [serial] V4 Sync the accepted specification, archive the completed change with evidence, and rerun post-archive validation. r[evaluation_streaming.complete_root_outcomes] r[evaluation_streaming.validation]
