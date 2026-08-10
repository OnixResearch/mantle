# CLI NDJSON and process-status evidence

Question: Does Phase 4 expose a bounded NDJSON stream without changing the aggregate build interface or losing terminal root accounting?

Inspected evidence:

- `src/evaluation_stream_output.rs`
- `src/build_cmd.rs`
- `src/main.rs`
- `crates/crunch-pipeline/src/evaluation_stream.rs`
- `crates/crunch-evaluation-stream-core/src/{types,ledger,projection}.rs`
- `docs/evaluation-stream-contract.md`
- `scripts/check-evaluation-stream-contract.rs`
- `tests/integration.rs`
- Pueue tasks `16923`, `16941`, `16950`, `16959`, `16964`, `16969`, `16984`, `16987`, and `16991`

Approach registry:

- The accepted approach uses an optional bounded pipeline channel. The shell writes each event as one flushed NDJSON record.
- A post-run conversion was rejected because it cannot expose live completion order.
- Direct worker writes were rejected because workers must not own JSON, stdout, or flush behavior.
- Unbounded buffering was rejected because output memory and backpressure must stay explicit.

Results:

- `mantle build --evaluation-stream` emits `run-start`, source-order `root-discovered`, live `root-terminal`, and one canonical `run-summary` record.
- The bounded channel capacity is derived from the selected-root limit. Each serialized record is checked against `event_bytes_max` before write.
- Write, flush, closed-consumer, and missing-summary failures return the stream-failure process status. A closed consumer also requests pipeline cancellation.
- A partial CLI run preserved the successful root reference, emitted one failed sibling, produced a canonical partial summary, and returned process status 1.
- A successful CLI run emitted all four record kinds, ended in `run-summary`, and returned process status 0.
- The existing `mantle --json build` success test passed unchanged and retained `crunch-build-report-v1`.
- The stream mode rejects incompatible aggregate JSON, planning, fix, and remote-dispatch combinations.
- Diagnostic admission redacts absolute paths, credential-shaped tokens, and standalone hexadecimal digests before bounded truncation.
- The reference-boundary checker scans bounded first-party Rust and Cargo manifest inputs. It rejects non-comment `nix-eval-jobs` references outside the approved lifecycle and documentation boundary.

Validation:

- Pueue task `16950`: four evaluation-stream CLI tests passed, including success, partial, broken stdout, and JSON conflict. The aggregate JSON compatibility test also passed.
- Pueue task `16959`: 17 pure-core tests, 37 pipeline unit tests, and 19 pipeline integration tests passed. Four determinism probes remained explicitly ignored. Four stream-output unit tests passed. The contract self-test and fixture check passed with two positive and five negative fixtures.
- Pueue task `16941`: focused Clippy passed for the pure core, pipeline, and Mantle targets with dependency linting disabled and warnings denied.
- Pueue task `16964`: the pure core compiled for `wasm32-unknown-unknown`.
- Pueue task `16969`: the focused Tiger Style check passed for the pure core.
- Pueue task `16984`: the pipeline Tiger Style check reached only seven existing findings outside the changed evaluation-stream path. No new stream finding remained after the request-structure and assertion fixes.
- Pueue task `16987`: the Mantle Tiger Style check stopped on existing root-library findings before it checked the binary surface. Focused Clippy is the available binary-surface rail.
- Pueue task `16991`: the post-refactor pipeline stream test and focused pipeline Clippy passed.
- `git diff --check` passed.

Decision: Phase 4 tasks I15 through I20 are complete. The implementation provides bounded live records, canonical complete summaries, honest process status, output-failure handling, and aggregate compatibility.

Owner: `stream-independent-root-outcomes`.

Next action: Complete Phase 5 documentation, validation, traceability, sync, and archive tasks.
