# Signal shell integration evidence

Question: Does explicit evaluation-stream mode convert process signals into bounded cancellation without false success?

Inspected evidence:

- `src/build_cmd.rs`
- `tests/integration.rs`
- `docs/evaluation-stream-contract.md`
- Pueue tasks `17189`, `17202`, `17210`, `17225`, `17239`, `17245`, and `17247`

Approach registry result:

- Mode-local Tokio listeners are validated.
- Global main-level handlers remain rejected because they would change other commands.
- A new signal-handler dependency remains rejected because Tokio already owns the required runtime integration.
- Default process termination remains rejected because it cannot preserve terminal root accounting.

Results:

- Unix stream mode registers `SIGINT` and `SIGTERM` listeners before pipeline launch.
- Other targets compile against the portable Tokio Ctrl-C listener.
- The first signal requests `EvaluationCancellation` and leaves bounded record draining active.
- A listener failure also requests cancellation before the shell returns its error.
- A repeated signal returns status 130 immediately and cannot report success.
- Real subprocess tests read `run-start` before sending a signal. They use 512 roots to retain channel backpressure and a bounded child wait.
- `SIGINT` and `SIGTERM` each produced one final cancelled summary with 512 terminal roots and status 130.
- The repeated `SIGINT` plus `SIGTERM` test returned status 130 without a success summary.
- Existing success, partial-result, broken-output, and JSON-conflict stream tests still pass.

Validation:

- Eighteen stream-core tests passed.
- One focused shell-policy test passed.
- Seven focused stream CLI tests passed without skips under the documented sandbox environment.
- Focused Clippy passed with warnings denied.
- The focused stream-core Tiger Style check passed.
- The stream-contract self-test and fixture check passed.
- `git diff --check` passed.

Decision: I3 through I7 are complete. Signal effects stay in the stream shell, while observation-count policy stays pure.

Owner: `handle-evaluation-stream-signals`.

Next action: Run lifecycle completion gates, sync the accepted requirement, and archive the change.
