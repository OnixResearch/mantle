# Stream-independent root outcome baseline

Question: What behavior and test state exist before stream-contract or pipeline changes?

Inspected evidence:

- `crates/crunch-eval/src/session.rs`
- `crates/crunch-pipeline/src/lib.rs`
- `crates/crunch-pipeline/tests/integration_build.rs`
- `tests/integration.rs`
- `tests/evaluator_budget_cli.rs`
- Pueue tasks `16101`, `16102`, `16103`, `16105`, `16107`, and `16118`

## Test results

1. `cargo test -p crunch-eval --lib --tests`
   - Result: 80 passed, zero failed.
2. `cargo test -p crunch-pipeline --lib --tests`
   - Library result: 29 passed, zero failed.
   - Integration result: 19 passed, zero failed, four ignored.
3. `cargo test -p mantle --test integration eval_`
   - Result: 22 passed, zero failed, 47 filtered out.
4. `cargo test -p mantle --test evaluator_budget_cli`
   - Result: 14 passed, zero failed.
5. `cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test`
   - Result: `machine schema contract self-test: PASS`.
6. `cargo -Zscript scripts/check-machine-schema-contracts.rs`
   - Baseline result: failed before this change.
   - The checker reports existing unclassified root JSON producers.
   - Examples include `src/evaluator_budget.rs`, `src/operator_contract.rs`, and StageX sources.

## Current `first_failure` behavior

- `stream_roots_into_worker` stores one `Option<EvalFailure>` named `first_failure`.
- The dispatcher starts at most `max_jobs` root workers before it observes a result.
- The first root error sets `first_failure`.
- After that error, the dispatcher starts no new root workers.
- A successful result that arrives after `first_failure` is set is ignored.
- Later root errors are also ignored.
- Undispatched roots receive no result entry.
- The pipeline exports only the first evaluation error through `failed` and `root_labels`.
- Roots converted before the first error can still reach build work.
- The focused policy test accepts at most one dispatched success before the `bad` root error.

Decision: Preserve this baseline as negative evidence. Replace it only after the stream contract and pure outcome rules exist.

Owner: `stream-independent-root-outcomes`.

Next action: Define and validate the v1 stream contract without changing pipeline dispatch.
