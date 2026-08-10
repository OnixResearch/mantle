# Signal cancellation baseline

Question: Does evaluation-stream mode currently translate operator process signals into cooperative cancellation?

Inspected evidence:

- `src/build_cmd.rs::run_build_with_stream_output`
- `crates/crunch-pipeline::EvaluationCancellation`
- Product-source search for `tokio::signal`, `SIGINT`, `SIGTERM`, and `ctrl_c`
- Pueue task `17163`

Results:

- The stream shell creates an `EvaluationCancellation` handle for output failures.
- Its event loop selects only stream records and the pipeline result.
- No product stream path registers `SIGINT`, `SIGTERM`, or portable Ctrl-C listeners.
- Therefore process signals cannot reach the existing cooperative cancellation handle.
- The focused baseline passed four evaluation-stream CLI tests. Two build-dependent cases reported their existing environment skip.

Decision: The follow-up must add mode-local signal observation before it can claim operator-driven terminal summaries.

Owner: `handle-evaluation-stream-signals`.

Next action: Add and test the pure first-versus-repeated signal policy before adding process effects.
