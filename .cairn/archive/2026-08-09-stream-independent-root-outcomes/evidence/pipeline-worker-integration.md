# Pipeline and worker integration evidence

Question: Does Phase 3 give every admitted selected root one terminal outcome while independent root-scoped failures continue and shared failures or cancellation stop dispatch?

Inspected evidence:

- `crates/crunch-eval/src/lib.rs`
- `crates/crunch-eval/src/session.rs`
- `crates/crunch-pipeline/src/evaluation_stream.rs`
- `crates/crunch-pipeline/src/lib.rs`
- `crates/crunch-pipeline/tests/integration_build.rs`
- Pueue tasks `16601`, `16604`, `16615`, `16632`, and `16641`

Architecture:

- `EvalStreamState` is the imperative coordinator. `OutcomeLedger` remains the pure authority for root transitions, dispatch decisions, and final accounting.
- The coordinator retains the existing resolved worker limit. It asserts that the `JoinSet` never exceeds that limit.
- Each worker returns a typed success, scoped failure, cancellation, or worker-loss completion.
- `crunch-eval::FailureScopeHint` classifies only scopes visible at the adapter boundary. Deserialization failures are root-scoped. I/O is shared-fatal. Boundary failures are coordinator failures.
- The Nickel facade keeps evaluator variants opaque. The adapter therefore classifies opaque Nickel failures as shared-fatal. It does not promote an unclassified failure to root-scoped.
- `EvaluationCancellation` stops new dispatch, wakes the coordinator, terminalizes unresolved work, and prevents a late success from replacing cancellation.
- Pipeline result construction derives failures and labels from the complete canonical `RunSummary`. It no longer uses `first_failure`.
- NDJSON and CLI projection remain deferred to Phase 4.

Positive results:

- Pueue task `16601` passed 82 `crunch-eval` tests, 35 `crunch-pipeline` unit tests, and 19 `crunch-pipeline` integration tests. Four explicitly ignored determinism probes were not executed and are not claimed.
- A malformed first root and a later valid root pass with one evaluation worker. The result is `partial`, with one failed and one successful root.
- A conversion failure and a later valid root pass with one evaluation worker.
- Recursive record roots preserve both sibling outcomes.
- The end-to-end build test preserves a successful sibling when another selected root fails deserialization.
- Source BLAKE3 identity is stable for replay and changes when source bytes change.

Negative results:

- Worker loss stops dispatch, marks the owned root `worker-lost`, and marks undispatched roots `not-started`.
- Shared resolver initialization failure dispatches no root and terminalizes all admitted roots as shared-fatal `not-started`.
- An opaque evaluator failure stops later dispatch as shared-fatal.
- A cancellation race preserves an earlier success, rejects a late second success as `cancelled`, and marks the undispatched third root `not-started`.
- The evaluator scope test rejects unsafe promotion: labeled shared I/O remains shared-fatal, and an opaque Nickel error remains shared-fatal.

Validation:

- Pueue task `16604`: focused Clippy passed for `crunch-evaluation-stream-core`, `crunch-eval`, and `crunch-pipeline`, with all targets, no dependency linting, and warnings denied.
- Pueue task `16632`: Rust formatting, the stream-contract self-test, two positive fixtures, five negative fixtures, and `git diff --check` passed.
- Pueue task `16615`: the repository Tiger Style command was blocked before it reached `crunch-pipeline` by existing findings in `crunch-overlay-core`, `crunch-gc-core`, and `crunch-composition-core`. This is not claimed as a Phase 3 pass. Focused Clippy and the explicit coordinator invariants cover the changed surface until the repository-wide blocker is repaired.
- Pueue task `16641`: pinned Cairn revision `e5ee2a61d8561d8fb47f42012b5d23211f847e7e` reported `valid: true`; the tasks gate reported `PASS` with 14 completed and 11 pending tasks.

Decision: Phase 3 tasks I11 through I14 are complete. The pipeline now has complete, stream-independent selected-root accounting without NDJSON or CLI changes.

Owner: `stream-independent-root-outcomes`.

Next action: Implement the Phase 4 machine stream as a shell projection over the completed ledger summary and live completion observations.
